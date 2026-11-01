//! Host lifecycle, asynchronous worker thread, event loop, and callback dispatch.

#![deny(clippy::unwrap_used)]

use mlapp_container::{AppContainer, AppManifest, ContainerError};
use mlapp_lang::{extract_code_from_document_xml, parse_app_classdef, AppClassDef, LangError};
use mlapp_runtime::{AppHandle, AppRuntime, RuntimeError, UiMutation};
use mlapp_ui::{parse_app_xml, Component, PropertyValue, UiError};
use rustlab_math::{DenseArray, StructArray, Value};
use std::collections::HashMap;
use std::io::{Read, Seek};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::Mutex;
use std::thread::{spawn, JoinHandle};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HostError {
    #[error("Container error: {0}")]
    Container(#[from] ContainerError),

    #[error("UI error: {0}")]
    Ui(#[from] UiError),

    #[error("Language error: {0}")]
    Lang(#[from] LangError),

    #[error("Runtime error: {0}")]
    Runtime(#[from] RuntimeError),

    #[error("Channel error: worker thread disconnected")]
    ChannelDisconnected,

    #[error("Callback execution timed out ({0} ms)")]
    CallbackTimeout(u64),
}

/// Security permissions and configuration for running an app package.
#[derive(Debug, Clone)]
pub struct HostConfig {
    pub allow_file_io: bool,
    pub allow_network: bool,
    pub allow_system: bool,
    pub allow_eval: bool,
    pub allow_interop: bool,
    pub allow_matlab: bool,
    pub callback_timeout_ms: u64,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            allow_file_io: false,
            allow_network: false,
            allow_system: false,
            allow_eval: false,
            allow_interop: false,
            allow_matlab: false,
            callback_timeout_ms: 5000,
        }
    }
}

/// UI event sent from the UI/render loop to the worker thread.
#[derive(Debug, Clone)]
pub enum UiEvent {
    ButtonClick {
        component_name: String,
    },
    ValueChanged {
        component_name: String,
        new_val: PropertyValue,
    },
    Shutdown,
}

/// Notification sent from the worker thread back to the UI/render loop.
#[derive(Debug, Clone)]
pub enum HostNotification {
    Mutations(Vec<UiMutation>),
    Alert { title: String, message: String },
    Error(String),
    Terminated,
}

/// Active App Designer session running on a background worker thread.
pub struct AppSession {
    pub manifest: AppManifest,
    pub ui_tree: Component,
    pub class_def: AppClassDef,
    pub app_handle: AppHandle,
    event_tx: Sender<UiEvent>,
    notif_rx: Mutex<Receiver<HostNotification>>,
    worker_handle: Option<JoinHandle<()>>,
}

impl AppSession {
    /// Launches an app package from any readable/seekable byte stream.
    pub fn launch<R: Read + Seek + Send + 'static>(
        reader: R,
        config: HostConfig,
    ) -> Result<Self, HostError> {
        let mut container = AppContainer::open(reader)?;
        let report = container.inspect()?;
        let manifest = report.manifest;

        // 1. Parse UI tree
        let app_xml = container.read_entry_to_string("appdesigner/app.xml")?;
        let ui_tree = parse_app_xml(&app_xml)?;

        // 2. Parse MATLAB code
        let doc_xml = container.read_entry_to_string("matlab/document.xml")?;
        let code_str = extract_code_from_document_xml(&doc_xml)?;
        let class_def = parse_app_classdef(&code_str)?;

        // 3. Setup communication channels
        let (event_tx, event_rx) = channel::<UiEvent>();
        let (notif_tx, notif_rx) = channel::<HostNotification>();

        // 4. Initialize AppHandle (Send + Sync)
        let app_handle = AppHandle::from_ui_tree(&manifest.app_name, &ui_tree);

        // 5. Spawn background worker thread (creates AppRuntime locally on thread)
        let handle_worker = app_handle.clone();
        let class_def_worker = class_def.clone();
        let config_clone = config.clone();
        let worker_handle = spawn(move || {
            let mut runtime = AppRuntime::new(handle_worker);
            runtime.enforce_sandbox_policies(
                config_clone.allow_file_io,
                config_clone.allow_network,
                config_clone.allow_system,
                config_clone.allow_eval,
                config_clone.allow_interop,
            );
            runtime.configure_matlab_bridge(config_clone.allow_matlab);
            runtime.load_class_def(class_def_worker.clone());

            // Run startupFcn
            if let Err(e) = runtime.run_startup_fcn() {
                let _ = notif_tx.send(HostNotification::Error(format!("startupFcn failed: {e}")));
            }

            // Drain any initial mutations from startupFcn
            let initial_mutations = runtime.app_handle.drain_mutations();
            if !initial_mutations.is_empty() {
                let _ = notif_tx.send(HostNotification::Mutations(initial_mutations));
            }

            worker_loop(runtime, class_def_worker, event_rx, notif_tx, config);
        });

        Ok(Self {
            manifest,
            ui_tree,
            class_def,
            app_handle,
            event_tx,
            notif_rx: Mutex::new(notif_rx),
            worker_handle: Some(worker_handle),
        })
    }

    /// Dispatches a UI event to the background worker.
    pub fn send_event(&self, event: UiEvent) -> Result<(), HostError> {
        self.event_tx
            .send(event)
            .map_err(|_| HostError::ChannelDisconnected)
    }

    /// Polls for notifications (mutations, alerts, errors) from the worker.
    pub fn poll_notification(&self) -> Option<HostNotification> {
        let rx = self.notif_rx.lock().unwrap_or_else(|p| p.into_inner());
        match rx.try_recv() {
            Ok(notif) => Some(notif),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(HostNotification::Terminated),
        }
    }

    /// Gracefully stops the worker thread.
    pub fn shutdown(&mut self) {
        self.app_handle.cancel();
        let _ = self.event_tx.send(UiEvent::Shutdown);
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for AppSession {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// The worker thread event loop.
fn worker_loop(
    mut runtime: AppRuntime,
    class_def: AppClassDef,
    event_rx: Receiver<UiEvent>,
    notif_tx: Sender<HostNotification>,
    _config: HostConfig,
) {
    while let Ok(event) = event_rx.recv() {
        match event {
            UiEvent::Shutdown => {
                let _ = notif_tx.send(HostNotification::Terminated);
                break;
            }
            UiEvent::ButtonClick { component_name } => {
                // Find registered callback method
                let callback_name =
                    find_callback_name(&runtime, &component_name, "ButtonPushedFcn")
                        .or_else(|| Some(format!("{component_name}Pushed")));

                if let Some(cb_name) = callback_name {
                    if class_def.methods.contains_key(&cb_name) {
                        let event = make_event_data(
                            &component_name,
                            "ButtonPushed",
                            &PropertyValue::Number(0.0),
                        );
                        if let Err(e) = run_callback_with_timeout(
                            &mut runtime,
                            &cb_name,
                            event,
                            _config.callback_timeout_ms,
                        ) {
                            let _ = notif_tx.send(HostNotification::Error(format!(
                                "Callback '{cb_name}' failed: {e}"
                            )));
                        }
                    }
                }

                // Drain and send batched mutations
                let mutations = runtime.app_handle.drain_mutations();
                if !mutations.is_empty() {
                    let _ = notif_tx.send(HostNotification::Mutations(mutations));
                }
            }
            UiEvent::ValueChanged {
                component_name,
                new_val,
            } => {
                let property_result = runtime.app_handle.set_component_property(
                    &component_name,
                    "Value",
                    new_val.clone(),
                );
                if let Err(e) = property_result {
                    let _ = notif_tx.send(HostNotification::Error(format!(
                        "Could not set Value on component '{component_name}': {e}"
                    )));
                }

                let callback_name =
                    find_callback_name(&runtime, &component_name, "ValueChangedFcn")
                        .or_else(|| Some(format!("{component_name}ValueChanged")));

                if let Some(cb_name) = callback_name {
                    if class_def.methods.contains_key(&cb_name) {
                        // App Designer callbacks receive event data as a struct
                        // with at least Source, EventName and Value fields. The
                        // new value is available as event.Value.
                        let event = make_event_data(&component_name, "ValueChanged", &new_val);
                        if let Err(e) = run_callback_with_timeout(
                            &mut runtime,
                            &cb_name,
                            event,
                            _config.callback_timeout_ms,
                        ) {
                            let _ = notif_tx.send(HostNotification::Error(format!(
                                "Callback '{cb_name}' failed: {e}"
                            )));
                        }
                    }
                }

                let mutations = runtime.app_handle.drain_mutations();
                if !mutations.is_empty() {
                    let _ = notif_tx.send(HostNotification::Mutations(mutations));
                }
            }
        }
    }
}

fn property_value_to_value(value: &PropertyValue) -> Value {
    match value {
        PropertyValue::String(s) => Value::char_vector(s),
        PropertyValue::Number(n) => Value::Double(DenseArray::scalar(*n)),
        PropertyValue::Boolean(b) => Value::logical_scalar(*b),
        PropertyValue::Position(rect) => Value::Double(
            DenseArray::from_vec(vec![1, 4], vec![rect.x, rect.y, rect.width, rect.height])
                .unwrap_or_else(|_| DenseArray::scalar(0.0)),
        ),
        PropertyValue::Color(c) => Value::Double(
            DenseArray::from_vec(vec![1, 4], c.iter().map(|v| f64::from(*v)).collect())
                .unwrap_or_else(|_| DenseArray::scalar(0.0)),
        ),
        PropertyValue::List(items) => {
            let data = items
                .iter()
                .map(|item| Value::char_vector(item))
                .collect::<Vec<_>>();
            let shape = vec![1, data.len()];
            Value::Cell(
                DenseArray::from_vec(shape, data)
                    .unwrap_or_else(|_| DenseArray::scalar(Value::char_vector(""))),
            )
        }
        PropertyValue::Unknown(s) => Value::char_vector(s),
    }
}

fn make_event_data(source: &str, event_name: &str, value: &PropertyValue) -> Value {
    let mut fields = HashMap::new();
    fields.insert("Source".to_string(), Value::char_vector(source));
    fields.insert("EventName".to_string(), Value::char_vector(event_name));
    fields.insert("Value".to_string(), property_value_to_value(value));
    Value::Struct(StructArray::scalar(fields))
}

fn find_callback_name(runtime: &AppRuntime, comp_name: &str, event_key: &str) -> Option<String> {
    let state = runtime
        .app_handle
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    state
        .components
        .get(comp_name)
        .and_then(|c| c.callbacks.get(&event_key.to_lowercase()).cloned())
}

fn run_callback_with_timeout(
    runtime: &mut AppRuntime,
    cb_name: &str,
    arg: Value,
    timeout_ms: u64,
) -> Result<(), RuntimeError> {
    if timeout_ms == 0 {
        return runtime.run_callback(cb_name, arg);
    }

    let cancel_token = runtime.app_handle.cancel_token.clone();
    let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let done_clone = done.clone();

    let watchdog = std::thread::spawn(move || {
        let step = 10;
        let mut elapsed = 0;
        while elapsed < timeout_ms {
            if done_clone.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(step));
            elapsed += step;
        }
        if !done_clone.load(std::sync::atomic::Ordering::Relaxed) {
            cancel_token.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    });

    let res = runtime.run_callback(cb_name, arg);
    done.store(true, std::sync::atomic::Ordering::Relaxed);
    let _ = watchdog.join();
    res
}
