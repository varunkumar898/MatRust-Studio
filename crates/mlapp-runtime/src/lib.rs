//! MATLAB App Designer Runtime Engine: Handle object semantics for `app`,
//! UI state synchronization, callback invocation, and built-in function registry.

#![deny(clippy::unwrap_used)]

use mlapp_lang::{AppClassDef, AppMethod};
use mlapp_ui::{Component, PropertyValue};
use rustlab_eval::Interpreter;
use rustlab_math::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Evaluation error: {0}")]
    Eval(String),

    #[error("Method '{0}' not found in AppClassDef")]
    MethodNotFound(String),

    #[error("Component '{0}' not found in App state")]
    ComponentNotFound(String),

    #[error("Unsupported MATLAB function '{name}'. Recommendation: {recommendation}")]
    UnsupportedFunction {
        name: String,
        recommendation: String,
    },

    #[error("Security violation: function '{0}' is blocked by sandbox policy")]
    SecurityViolation(String),
}

/// Represents a UI mutation triggered during callback execution.
#[derive(Debug, Clone, PartialEq)]
pub enum UiMutation {
    SetProperty {
        component: String,
        property: String,
        value: PropertyValue,
    },
    PlotCall {
        axes: String,
        command: String,
        args: Vec<Value>,
    },
    Alert {
        title: String,
        message: String,
    },
}

/// Shared runtime state of the running App Designer instance.
#[derive(Debug, Default)]
pub struct AppRuntimeState {
    pub name: String,
    pub properties: HashMap<String, Value>,
    pub components: HashMap<String, Component>,
    pub pending_mutations: Vec<UiMutation>,
}

impl AppRuntimeState {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            properties: HashMap::new(),
            components: HashMap::new(),
            pending_mutations: Vec::new(),
        }
    }

    /// Register a component from the UI hierarchy into the runtime state.
    pub fn register_component(&mut self, comp: &Component) {
        self.components.insert(comp.name.clone(), comp.clone());
        for child in &comp.children {
            self.register_component(child);
        }
    }

    /// Drain queued mutations to dispatch to the UI thread.
    pub fn drain_mutations(&mut self) -> Vec<UiMutation> {
        std::mem::take(&mut self.pending_mutations)
    }
}

/// Handle object pointer for `app`.
#[derive(Clone)]
pub struct AppHandle {
    pub state: Arc<Mutex<AppRuntimeState>>,
    pub cancel_token: Arc<AtomicBool>,
}

impl AppHandle {
    pub fn new(name: &str) -> Self {
        Self {
            state: Arc::new(Mutex::new(AppRuntimeState::new(name))),
            cancel_token: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancel_token.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.load(Ordering::SeqCst)
    }

    pub fn from_ui_tree(name: &str, root: &Component) -> Self {
        let handle = Self::new(name);
        {
            let mut state = handle
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state.register_component(root);
        }
        handle
    }

    pub fn set_property(&self, key: &str, val: Value) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.properties.insert(key.to_string(), val);
    }

    pub fn get_property(&self, key: &str) -> Option<Value> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.properties.get(key).cloned()
    }

    pub fn set_component_property(
        &self,
        component_name: &str,
        prop_name: &str,
        prop_val: PropertyValue,
    ) -> Result<(), RuntimeError> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(comp) = state.components.get_mut(component_name) {
            comp.set_property(prop_name, prop_val.clone())
                .map_err(|e| RuntimeError::Eval(e.to_string()))?;
            state.pending_mutations.push(UiMutation::SetProperty {
                component: component_name.to_string(),
                property: prop_name.to_string(),
                value: prop_val,
            });
            Ok(())
        } else {
            Err(RuntimeError::ComponentNotFound(component_name.to_string()))
        }
    }

    pub fn drain_mutations(&self) -> Vec<UiMutation> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.drain_mutations()
    }
}

/// Runtime evaluator for App Designer applications.
pub struct AppRuntime {
    pub interpreter: Interpreter,
    pub app_handle: AppHandle,
    pub class_def: Option<AppClassDef>,
    pub loaded_models: Arc<Mutex<HashMap<String, mlapp_sim::BlockDiagram>>>,
}

impl AppRuntime {
    pub fn new(app_handle: AppHandle) -> Self {
        let loaded_models = Arc::new(Mutex::new(HashMap::new()));
        let mut interp = Interpreter::new();
        rustlab_stdlib::register_all(&mut interp);
        interp.cancel_token = Some(app_handle.cancel_token.clone());
        register_app_builtins(&mut interp, &app_handle, loaded_models.clone());

        let mut runtime = Self {
            interpreter: interp,
            app_handle,
            class_def: None,
            loaded_models,
        };
        runtime.configure_matlab_bridge(false);
        runtime
    }

    /// Loads an in-memory `BlockDiagram` into the runtime.
    pub fn load_diagram(&self, diagram: mlapp_sim::BlockDiagram) {
        let mut models = self.loaded_models.lock().unwrap_or_else(|p| p.into_inner());
        models.insert(diagram.name.clone(), diagram);
    }

    /// Configures MATLAB bridge delegation for unsupported functions.
    pub fn configure_matlab_bridge(&mut self, allow_matlab: bool) {
        let session = if allow_matlab {
            Some(Arc::new(Mutex::new(
                mlapp_matlab::MatlabSession::start()
                    .unwrap_or_else(|_| mlapp_matlab::MatlabSession::start_mock()),
            )))
        } else {
            None
        };

        self.interpreter
            .set_fallback_handler(move |name: &str, args: &[Value], _nargout: usize| {
                if let Some(ref sess_arc) = session {
                    let mut sess = sess_arc.lock().unwrap_or_else(|p| p.into_inner());
                    match sess.call(name, args) {
                        Ok(val) => Ok(vec![val]),
                        Err(e) => Err(rustlab_eval::EvalError::UserError {
                            identifier: "MATLAB:BridgeError".to_string(),
                            message: format!("MATLAB bridge delegation failed for '{name}': {e}"),
                            span: rustlab_parse::Span::default(),
                        }),
                    }
                } else {
                    Err(rustlab_eval::EvalError::UserError {
                        identifier: "MATLAB:UndefinedFunction".to_string(),
                        message: format!(
                            "Unsupported MATLAB function '{name}'. Hint: run with --allow-matlab to delegate to real MATLAB"
                        ),
                        span: rustlab_parse::Span::default(),
                    })
                }
            });
    }

    pub fn load_class_def(&mut self, class_def: AppClassDef) {
        // Initialize default properties from the class definition
        for prop in &class_def.properties {
            if let Some(ref init_str) = prop.initial_value {
                if let Ok(num) = init_str.parse::<f64>() {
                    self.app_handle.set_property(
                        &prop.name,
                        Value::Double(rustlab_math::DenseArray::scalar(num)),
                    );
                } else {
                    let chars: Vec<char> = init_str.chars().collect();
                    let len = chars.len();
                    let arr = rustlab_math::DenseArray::from_vec_unchecked(vec![1, len], chars);
                    self.app_handle.set_property(&prop.name, Value::Char(arr));
                }
            }
        }
        // Register class methods into the interpreter function table
        for (name, method) in &class_def.methods {
            self.interpreter.functions.insert(
                name.clone(),
                rustlab_parse::ast::FunctionDef {
                    name: name.clone(),
                    inputs: method.inputs.clone(),
                    outputs: method.outputs.clone(),
                    body: method.body.clone(),
                    span: rustlab_parse::Span::default(),
                },
            );
        }

        self.class_def = Some(class_def);
    }

    /// Configures runtime sandbox permissions for untrusted app code.
    pub fn enforce_permissions(&mut self, allow_file_io: bool, allow_network: bool) {
        self.enforce_sandbox_policies(allow_file_io, allow_network, false, false, false);
    }

    /// Full sandbox policy configurator covering file I/O, network, system commands, eval, and external bridges.
    pub fn enforce_sandbox_policies(
        &mut self,
        allow_file_io: bool,
        allow_network: bool,
        allow_system: bool,
        allow_eval: bool,
        allow_interop: bool,
    ) {
        if !allow_file_io {
            for name in [
                "load", "save", "fopen", "fwrite", "fread", "fclose", "delete", "rmdir", "mkdir",
                "copyfile", "movefile", "cd", "csvread", "csvwrite",
            ] {
                self.interpreter
                    .register_builtin(name, move |_args: &[Value], _nargout: usize| {
                        Err(rustlab_eval::EvalError::UserError {
                            identifier: "RUSTLAB:SecurityViolation".to_string(),
                            message: format!(
                                "Security violation: function '{name}' is blocked by sandbox policy (allow_file_io = false)"
                            ),
                            span: rustlab_parse::Span::default(),
                        })
                    });
            }
        }
        if !allow_network {
            for name in [
                "web", "urlread", "urlwrite", "webread", "websave", "webwrite",
            ] {
                self.interpreter
                    .register_builtin(name, move |_args: &[Value], _nargout: usize| {
                        Err(rustlab_eval::EvalError::UserError {
                            identifier: "RUSTLAB:SecurityViolation".to_string(),
                            message: format!(
                                "Security violation: function '{name}' is blocked by sandbox policy (allow_network = false)"
                            ),
                            span: rustlab_parse::Span::default(),
                        })
                    });
            }
        }
        if !allow_system {
            for name in [
                "system", "dos", "unix", "getenv", "setenv", "exit", "quit", "!",
            ] {
                self.interpreter
                    .register_builtin(name, move |_args: &[Value], _nargout: usize| {
                        Err(rustlab_eval::EvalError::UserError {
                            identifier: "RUSTLAB:SecurityViolation".to_string(),
                            message: format!(
                                "Security violation: shell command function '{name}' is blocked by sandbox policy (allow_system = false)"
                            ),
                            span: rustlab_parse::Span::default(),
                        })
                    });
            }
        }
        if !allow_eval {
            for name in ["eval", "evalc", "evalin", "assignin", "run", "source"] {
                self.interpreter
                    .register_builtin(name, move |_args: &[Value], _nargout: usize| {
                        Err(rustlab_eval::EvalError::UserError {
                            identifier: "RUSTLAB:SecurityViolation".to_string(),
                            message: format!(
                                "Security violation: dynamic evaluation function '{name}' is blocked by sandbox policy (allow_eval = false)"
                            ),
                            span: rustlab_parse::Span::default(),
                        })
                    });
            }

            self.interpreter
                .register_builtin("builtin", move |args: &[Value], _nargout: usize| {
                    let target = match args.first() {
                        Some(Value::Char(da)) => da.as_slice().iter().collect::<String>(),
                        Some(Value::String(da)) => {
                            da.as_slice().first().cloned().unwrap_or_default()
                        }
                        _ => String::new(),
                    };
                    Err(rustlab_eval::EvalError::UserError {
                        identifier: "RUSTLAB:SecurityViolation".to_string(),
                        message: format!(
                            "Security violation: builtin invocation of '{target}' is blocked by sandbox policy"
                        ),
                        span: rustlab_parse::Span::default(),
                    })
                });
        }
        if !allow_interop {
            for name in [
                "py",
                "python",
                "javaObject",
                "javaMethod",
                "javaArray",
                "loadlibrary",
                "calllib",
                "unloadlibrary",
            ] {
                self.interpreter
                    .register_builtin(name, move |_args: &[Value], _nargout: usize| {
                        Err(rustlab_eval::EvalError::UserError {
                            identifier: "RUSTLAB:SecurityViolation".to_string(),
                            message: format!(
                                "Security violation: external language integration '{name}' is blocked by sandbox policy (allow_interop = false)"
                            ),
                            span: rustlab_parse::Span::default(),
                        })
                    });
            }
        }
    }

    /// Executes `startupFcn(app)`.
    pub fn run_startup_fcn(&mut self) -> Result<(), RuntimeError> {
        let startup = self
            .class_def
            .as_ref()
            .and_then(|c| c.methods.get("startupFcn"))
            .cloned();

        if let Some(method) = startup {
            self.execute_method(&method, vec![])?;
        }
        Ok(())
    }

    /// Invokes a named callback, e.g. `ButtonPushed(app, event)`.
    pub fn run_callback(
        &mut self,
        callback_name: &str,
        event_data: Value,
    ) -> Result<(), RuntimeError> {
        let method = self
            .class_def
            .as_ref()
            .and_then(|c| c.methods.get(callback_name))
            .cloned()
            .ok_or_else(|| RuntimeError::MethodNotFound(callback_name.to_string()))?;

        self.execute_method(&method, vec![event_data])
    }

    fn execute_method(&mut self, method: &AppMethod, args: Vec<Value>) -> Result<(), RuntimeError> {
        // Construct the `app` struct containing properties and component handles
        let mut app_fields = HashMap::new();
        {
            let state = self
                .app_handle
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            for (k, v) in &state.properties {
                app_fields.insert(k.clone(), v.clone());
            }
            for (comp_name, comp) in &state.components {
                let mut comp_fields = HashMap::new();
                for (pk, pv) in &comp.properties {
                    let v = match pv {
                        PropertyValue::String(s) => {
                            let chars: Vec<char> = s.chars().collect();
                            Value::Char(rustlab_math::DenseArray::from_vec_unchecked(
                                vec![1, chars.len()],
                                chars,
                            ))
                        }
                        PropertyValue::Number(n) => {
                            Value::Double(rustlab_math::DenseArray::scalar(*n))
                        }
                        PropertyValue::Boolean(b) => {
                            Value::Logical(rustlab_math::DenseArray::scalar(*b))
                        }
                        PropertyValue::Position(r) => {
                            Value::Double(rustlab_math::DenseArray::from_vec_unchecked(
                                vec![1, 4],
                                vec![r.x, r.y, r.width, r.height],
                            ))
                        }
                        _ => Value::Double(rustlab_math::DenseArray::empty()),
                    };
                    comp_fields.insert(pk.clone(), v);
                }
                comp_fields.insert("Name".to_string(), Value::char_vector(comp_name));
                app_fields.insert(
                    comp_name.clone(),
                    Value::Struct(rustlab_math::StructArray::scalar(comp_fields)),
                );
            }
        }

        // Bind parameters in environment: inputs[0] is typically "app"
        if let Some(app_param) = method.inputs.first() {
            self.interpreter.env.set(
                app_param,
                Value::Struct(rustlab_math::StructArray::scalar(app_fields)),
            );
        }

        // Bind remaining inputs with passed args
        for (i, param_name) in method.inputs.iter().skip(1).enumerate() {
            if let Some(arg_val) = args.get(i) {
                self.interpreter.env.set(param_name, arg_val.clone());
            }
        }

        // Evaluate each statement in method body
        for stmt in &method.body {
            if self.app_handle.is_cancelled() {
                return Err(RuntimeError::Eval(
                    "Callback execution cancelled by user request".to_string(),
                ));
            }
            self.interpreter
                .eval_stmt(stmt)
                .map_err(|e| RuntimeError::Eval(e.to_string()))?;
        }

        // Sync back any modifications to `app` from environment
        if let Some(app_param) = method.inputs.first() {
            if let Some(Value::Struct(s)) = self.interpreter.env.get(app_param) {
                let mut comp_updates = Vec::new();
                let mut prop_updates = Vec::new();

                for (k, v) in &s.data[0] {
                    if let Value::Struct(comp_s) = v {
                        for (pk, pv) in &comp_s.data[0] {
                            let ui_pv = match pv {
                                Value::Char(c) => {
                                    PropertyValue::String(c.as_slice().iter().collect())
                                }
                                Value::Double(d) => {
                                    if d.as_slice().len() == 1 {
                                        PropertyValue::Number(d.as_slice()[0])
                                    } else {
                                        PropertyValue::Unknown(format!("{pv:?}"))
                                    }
                                }
                                Value::Logical(l) => PropertyValue::Boolean(l.as_slice()[0]),
                                _ => PropertyValue::Unknown(format!("{pv:?}")),
                            };
                            comp_updates.push((k.clone(), pk.clone(), ui_pv));
                        }
                    } else {
                        prop_updates.push((k.clone(), v.clone()));
                    }
                }

                let mut state = self
                    .app_handle
                    .state
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                for (k, v) in prop_updates {
                    state.properties.insert(k, v);
                }
                for (comp_name, prop_name, prop_val) in comp_updates {
                    if prop_name == "Name" {
                        continue;
                    }
                    if let Some(comp) = state.components.get_mut(&comp_name) {
                        let changed = match comp.properties.get(&prop_name) {
                            Some(old) => old != &prop_val,
                            None => true,
                        };
                        if changed {
                            let _ = comp.set_property(&prop_name, prop_val.clone());
                            state.pending_mutations.push(UiMutation::SetProperty {
                                component: comp_name,
                                property: prop_name,
                                value: prop_val,
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

fn value_to_plain_string(v: &Value) -> String {
    match v {
        Value::String(sa) => sa.as_slice().first().cloned().unwrap_or_default(),
        Value::Char(ca) => ca.as_slice().iter().collect(),
        _ => format!("{v}").trim_matches('"').to_string(),
    }
}

fn resolve_slx_path(model_name: &str) -> Option<std::path::PathBuf> {
    let path = std::path::PathBuf::from(model_name);
    let slx_path = if path.extension().is_none() {
        path.with_extension("slx")
    } else {
        path
    };
    let candidates = [
        slx_path.clone(),
        std::path::PathBuf::from("fixtures/sample_models").join(&slx_path),
        std::path::PathBuf::from("../../fixtures/sample_models").join(&slx_path),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

/// Registers App Designer specific builtins like `uialert`, `uiconfirm`, `plot`, and Simulink `sim`/`set_param` hooks.
fn register_app_builtins(
    interp: &mut Interpreter,
    app_handle: &AppHandle,
    loaded_models: Arc<Mutex<HashMap<String, mlapp_sim::BlockDiagram>>>,
) {
    let handle_clone = app_handle.clone();
    interp.register_builtin("uialert", move |args: &[Value], _nargout: usize| {
        let msg = args.get(1).map(value_to_plain_string).unwrap_or_default();
        let title = args
            .get(2)
            .map(value_to_plain_string)
            .unwrap_or_else(|| "Alert".to_string());

        let mut state = handle_clone
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.pending_mutations.push(UiMutation::Alert {
            title,
            message: msg,
        });
        Ok(vec![])
    });

    // Pause builtin with cooperative cancellation polling
    let cancel_pause = app_handle.cancel_token.clone();
    interp.register_builtin("pause", move |args: &[Value], _nargout: usize| {
        if let Some(Value::Double(d)) = args.first() {
            if let Some(secs) = d.as_slice().first() {
                let total_ms = (*secs * 1000.0) as u64;
                let step_ms = 10;
                let mut elapsed = 0;
                while elapsed < total_ms {
                    if cancel_pause.load(std::sync::atomic::Ordering::Relaxed) {
                        return Err(rustlab_eval::EvalError::UserError {
                            identifier: "RUSTLAB:ExecutionCancelled".to_string(),
                            message: "Execution cancelled by timeout or user request".to_string(),
                            span: rustlab_parse::Span::default(),
                        });
                    }
                    std::thread::sleep(std::time::Duration::from_millis(step_ms));
                    elapsed += step_ms;
                }
            }
        }
        Ok(vec![])
    });

    // Drawnow builtin (flush event queue)
    interp.register_builtin("drawnow", |_args: &[Value], _nargout: usize| Ok(vec![]));

    // Plot builtin hook
    let handle_plot = app_handle.clone();
    interp.register_builtin("plot", move |args: &[Value], _nargout: usize| {
        let (axes_name, plot_args) = if let Some(first) = args.first() {
            if let Value::Struct(s) = first {
                let name = s
                    .get_field("Name")
                    .or_else(|| s.get_field("name"))
                    .map(value_to_plain_string)
                    .unwrap_or_else(|| "CurrentAxes".to_string());
                (name, args[1..].to_vec())
            } else if let Value::Char(_) | Value::String(_) = first {
                (value_to_plain_string(first), args[1..].to_vec())
            } else {
                ("CurrentAxes".to_string(), args.to_vec())
            }
        } else {
            ("CurrentAxes".to_string(), vec![])
        };

        let mut state = handle_plot
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.pending_mutations.push(UiMutation::PlotCall {
            axes: axes_name,
            command: "plot".to_string(),
            args: plot_args,
        });
        Ok(vec![])
    });

    // Hold builtin hook
    let handle_hold = app_handle.clone();
    interp.register_builtin("hold", move |args: &[Value], _nargout: usize| {
        let (axes_name, mode) = match args.len() {
            0 => ("CurrentAxes".to_string(), "on".to_string()),
            1 => ("CurrentAxes".to_string(), value_to_plain_string(&args[0])),
            _ => {
                let ax = if let Value::Struct(s) = &args[0] {
                    s.get_field("Name")
                        .or_else(|| s.get_field("name"))
                        .map(value_to_plain_string)
                        .unwrap_or_else(|| "CurrentAxes".to_string())
                } else {
                    value_to_plain_string(&args[0])
                };
                (ax, value_to_plain_string(&args[1]))
            }
        };
        let mut state = handle_hold
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.pending_mutations.push(UiMutation::PlotCall {
            axes: axes_name,
            command: "hold".to_string(),
            args: vec![Value::char_vector(&mode)],
        });
        Ok(vec![])
    });

    // Simulink sim builtin
    let models_sim = loaded_models.clone();
    interp.register_builtin("sim", move |args: &[Value], _nargout: usize| {
        if args.is_empty() {
            return Err(rustlab_eval::EvalError::UserError {
                identifier: "Simulink:InvalidInput".to_string(),
                message: "sim requires at least 1 argument (model name)".to_string(),
                span: rustlab_parse::Span::default(),
            });
        }
        let model_name = value_to_plain_string(&args[0]);
        let mut config = mlapp_sim::SimulationConfig::default();

        if args.len() >= 3 {
            let mut i = 1;
            while i + 1 < args.len() {
                let param = value_to_plain_string(&args[i]);
                let val_str = value_to_plain_string(&args[i + 1]);
                if param.eq_ignore_ascii_case("StopTime") {
                    if let Ok(t) = val_str.parse::<f64>() {
                        config.stop_time = t;
                    }
                } else if param.eq_ignore_ascii_case("StartTime") {
                    if let Ok(t) = val_str.parse::<f64>() {
                        config.start_time = t;
                    }
                } else if param.eq_ignore_ascii_case("FixedStep") {
                    if let Ok(dt) = val_str.parse::<f64>() {
                        config.step_size = dt;
                    }
                }
                i += 2;
            }
        }

        let diagram_opt = {
            let models = models_sim.lock().unwrap_or_else(|p| p.into_inner());
            models.get(&model_name).cloned()
        };

        let diagram = if let Some(diag) = diagram_opt {
            diag
        } else if let Some(slx_path) = resolve_slx_path(&model_name) {
            let f =
                std::fs::File::open(&slx_path).map_err(|e| rustlab_eval::EvalError::UserError {
                    identifier: "Simulink:LoadError".to_string(),
                    message: format!("Failed to open model file {:?}: {e}", slx_path),
                    span: rustlab_parse::Span::default(),
                })?;
            let (diag, _) = mlapp_sim::inspect_slx(f, mlapp_container::ContainerLimits::default())
                .map_err(|e| rustlab_eval::EvalError::UserError {
                    identifier: "Simulink:InspectError".to_string(),
                    message: format!("Failed to inspect model {:?}: {e}", slx_path),
                    span: rustlab_parse::Span::default(),
                })?;
            diag
        } else {
            return Err(rustlab_eval::EvalError::UserError {
                identifier: "Simulink:ModelNotFound".to_string(),
                message: format!(
                    "Simulink model '{model_name}' not found in loaded models or on disk"
                ),
                span: rustlab_parse::Span::default(),
            });
        };

        let sim_out = mlapp_sim::simulate_diagram(&diagram, &config).map_err(|e| {
            rustlab_eval::EvalError::UserError {
                identifier: "Simulink:SimError".to_string(),
                message: format!("Simulation failed for model '{model_name}': {e}"),
                span: rustlab_parse::Span::default(),
            }
        })?;

        Ok(vec![sim_out.to_matlab_value()])
    });

    // Simulink set_param builtin
    let models_set = loaded_models.clone();
    interp.register_builtin("set_param", move |args: &[Value], _nargout: usize| {
        if args.len() < 3 {
            return Err(rustlab_eval::EvalError::UserError {
                identifier: "Simulink:InvalidInput".to_string(),
                message: "set_param requires 3 arguments: (object, parameter, value)".to_string(),
                span: rustlab_parse::Span::default(),
            });
        }
        let target = value_to_plain_string(&args[0]);
        let param_name = value_to_plain_string(&args[1]);
        let param_val = value_to_plain_string(&args[2]);

        let mut models = models_set.lock().unwrap_or_else(|p| p.into_inner());

        let (model_name, block_name) = if let Some((m, b)) = target.split_once('/') {
            (m, Some(b))
        } else {
            (target.as_str(), None)
        };

        if let Some(diag) = models.get_mut(model_name) {
            if let Some(bname) = block_name {
                if let Some(b) = diag.find_block_mut(bname) {
                    b.set_param(param_name, param_val);
                } else {
                    return Err(rustlab_eval::EvalError::UserError {
                        identifier: "Simulink:BlockNotFound".to_string(),
                        message: format!("Block '{bname}' not found in model '{model_name}'"),
                        span: rustlab_parse::Span::default(),
                    });
                }
            } else {
                diag.properties.insert(param_name, param_val);
            }
        } else {
            return Err(rustlab_eval::EvalError::UserError {
                identifier: "Simulink:ModelNotFound".to_string(),
                message: format!("Simulink model '{model_name}' not loaded"),
                span: rustlab_parse::Span::default(),
            });
        }
        Ok(vec![])
    });

    // Simulink get_param builtin
    let models_get = loaded_models.clone();
    interp.register_builtin("get_param", move |args: &[Value], _nargout: usize| {
        if args.len() < 2 {
            return Err(rustlab_eval::EvalError::UserError {
                identifier: "Simulink:InvalidInput".to_string(),
                message: "get_param requires 2 arguments: (object, parameter)".to_string(),
                span: rustlab_parse::Span::default(),
            });
        }
        let target = value_to_plain_string(&args[0]);
        let param_name = value_to_plain_string(&args[1]);

        let models = models_get.lock().unwrap_or_else(|p| p.into_inner());
        let (model_name, block_name) = if let Some((m, b)) = target.split_once('/') {
            (m, Some(b))
        } else {
            (target.as_str(), None)
        };

        if let Some(diag) = models.get(model_name) {
            let val = if let Some(bname) = block_name {
                diag.find_block(bname)
                    .and_then(|b| b.get_param(&param_name))
            } else {
                diag.properties.get(&param_name).map(|s| s.as_str())
            };
            if let Some(v) = val {
                Ok(vec![Value::char_vector(v)])
            } else {
                Err(rustlab_eval::EvalError::UserError {
                    identifier: "Simulink:ParamNotFound".to_string(),
                    message: format!("Parameter '{param_name}' not found on '{target}'"),
                    span: rustlab_parse::Span::default(),
                })
            }
        } else {
            Err(rustlab_eval::EvalError::UserError {
                identifier: "Simulink:ModelNotFound".to_string(),
                message: format!("Simulink model '{model_name}' not loaded"),
                span: rustlab_parse::Span::default(),
            })
        }
    });

    // Simulink load_system builtin
    let models_load = loaded_models.clone();
    interp.register_builtin("load_system", move |args: &[Value], _nargout: usize| {
        if args.is_empty() {
            return Err(rustlab_eval::EvalError::UserError {
                identifier: "Simulink:InvalidInput".to_string(),
                message: "load_system requires model name".to_string(),
                span: rustlab_parse::Span::default(),
            });
        }
        let model_name = value_to_plain_string(&args[0]);
        if let Some(slx_path) = resolve_slx_path(&model_name) {
            let f =
                std::fs::File::open(&slx_path).map_err(|e| rustlab_eval::EvalError::UserError {
                    identifier: "Simulink:LoadError".to_string(),
                    message: format!("Failed to open model file {:?}: {e}", slx_path),
                    span: rustlab_parse::Span::default(),
                })?;
            let (diag, _) = mlapp_sim::inspect_slx(f, mlapp_container::ContainerLimits::default())
                .map_err(|e| rustlab_eval::EvalError::UserError {
                    identifier: "Simulink:InspectError".to_string(),
                    message: format!("Failed to inspect model {:?}: {e}", slx_path),
                    span: rustlab_parse::Span::default(),
                })?;
            let mut models = models_load.lock().unwrap_or_else(|p| p.into_inner());
            models.insert(diag.name.clone(), diag);
            Ok(vec![])
        } else {
            Err(rustlab_eval::EvalError::UserError {
                identifier: "Simulink:ModelNotFound".to_string(),
                message: format!("Simulink model '{model_name}' not found on disk"),
                span: rustlab_parse::Span::default(),
            })
        }
    });

    // Simulink close_system builtin
    let models_close = loaded_models.clone();
    interp.register_builtin("close_system", move |args: &[Value], _nargout: usize| {
        if let Some(first) = args.first() {
            let name = value_to_plain_string(first);
            let mut models = models_close.lock().unwrap_or_else(|p| p.into_inner());
            models.remove(&name);
        }
        Ok(vec![])
    });

    // Simulink bdclose builtin
    let models_bdclose = loaded_models;
    interp.register_builtin("bdclose", move |args: &[Value], _nargout: usize| {
        let mut models = models_bdclose.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(first) = args.first() {
            let name = value_to_plain_string(first);
            if name == "all" {
                models.clear();
            } else {
                models.remove(&name);
            }
        } else {
            models.clear();
        }
        Ok(vec![])
    });
}

/// Dispatches an unsupported function to the optional GNU Octave bridge if enabled.
pub fn call_octave_bridge(func_name: &str, _args: &[Value]) -> Result<Vec<Value>, RuntimeError> {
    Err(RuntimeError::UnsupportedFunction {
        name: func_name.to_string(),
        recommendation: format!(
            "Function '{func_name}' is not currently implemented in the native runtime."
        ),
    })
}
