use crate::save_screenshot_png;
use mlapp_host::{AppSession, HostNotification, UiEvent};
use mlapp_runtime::UiMutation;
use mlapp_ui::{Component, PropertyValue};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AutotestAction {
    WaitMs {
        duration_ms: u64,
    },
    Click {
        component: String,
    },
    SetValue {
        component: String,
        value: String,
    },
    ExpectProperty {
        component: String,
        property: String,
        expected: String,
    },
    Screenshot {
        path: PathBuf,
    },
    Close,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutotestScript {
    pub actions: Vec<AutotestAction>,
}

impl AutotestScript {
    pub fn from_json_str(s: &str) -> Result<Self, serde_json::Error> {
        if let Ok(script) = serde_json::from_str::<AutotestScript>(s) {
            Ok(script)
        } else {
            let actions = serde_json::from_str::<Vec<AutotestAction>>(s)?;
            Ok(Self { actions })
        }
    }

    pub fn from_file<P: AsRef<Path>>(
        path: P,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let text = std::fs::read_to_string(path)?;
        Ok(Self::from_json_str(&text)?)
    }
}

pub fn find_component<'a>(root: &'a Component, name: &str) -> Option<&'a Component> {
    let clean = name.strip_prefix("app.").unwrap_or(name);
    if let Some(c) = root.find_by_name(clean) {
        return Some(c);
    }
    if let Some(c) = root.find_by_name(name) {
        return Some(c);
    }
    if name == "CurrentAxes" {
        return root
            .children
            .iter()
            .find(|c| c.kind == mlapp_ui::ComponentKind::UIAxes);
    }
    None
}

pub fn find_component_mut<'a>(root: &'a mut Component, name: &str) -> Option<&'a mut Component> {
    let clean = name.strip_prefix("app.").unwrap_or(name);
    if root.find_by_name(clean).is_some() {
        return root.find_by_name_mut(clean);
    }
    if root.find_by_name(name).is_some() {
        return root.find_by_name_mut(name);
    }
    if name == "CurrentAxes" {
        return root
            .children
            .iter_mut()
            .find(|c| c.kind == mlapp_ui::ComponentKind::UIAxes);
    }
    None
}

pub fn matches_property_value(val: &PropertyValue, expected: &str) -> bool {
    match val {
        PropertyValue::String(s) => s == expected,
        PropertyValue::Unknown(s) => s == expected,
        PropertyValue::Number(n) => {
            if let Ok(exp_num) = expected.parse::<f64>() {
                (n - exp_num).abs() < 1e-6
            } else {
                format!("{n}") == expected
            }
        }
        PropertyValue::Boolean(b) => {
            if let Ok(exp_b) = expected.parse::<bool>() {
                b == &exp_b
            } else {
                format!("{b}") == expected
            }
        }
        _ => format!("{val:?}") == expected,
    }
}

pub fn parse_property_value(s: &str) -> PropertyValue {
    if let Ok(n) = s.parse::<f64>() {
        PropertyValue::Number(n)
    } else if let Ok(b) = s.parse::<bool>() {
        PropertyValue::Boolean(b)
    } else {
        PropertyValue::String(s.to_string())
    }
}

pub fn pump_session_mutations(session: &mut AppSession) {
    while let Some(notif) = session.poll_notification() {
        match notif {
            HostNotification::Mutations(mutations) => {
                for m in mutations {
                    match m {
                        UiMutation::SetProperty {
                            component,
                            property,
                            value,
                        } => {
                            if let Some(comp) = find_component_mut(&mut session.ui_tree, &component)
                            {
                                let _ = comp.set_property(&property, value);
                            }
                        }
                        UiMutation::PlotCall {
                            axes,
                            command,
                            args,
                        } => {
                            let num_points = args
                                .iter()
                                .filter_map(|v| v.to_double_array().ok())
                                .map(|a| a.numel())
                                .max()
                                .unwrap_or(0);
                            let axes_name = axes.strip_prefix("app.").unwrap_or(&axes);
                            if let Some(comp) = find_component_mut(&mut session.ui_tree, axes_name)
                            {
                                let _ = comp
                                    .set_property("HasPlottedData", PropertyValue::Boolean(true));
                                let _ = comp.set_property(
                                    "NumPoints",
                                    PropertyValue::Number(num_points as f64),
                                );
                                let _ = comp
                                    .set_property("PlotCommand", PropertyValue::String(command));
                            }
                        }
                        UiMutation::Alert { title, message } => {
                            eprintln!("[Autotest Alert] {title}: {message}");
                        }
                    }
                }
            }
            HostNotification::Alert { title, message } => {
                eprintln!("[Autotest Alert] {title}: {message}");
            }
            HostNotification::Error(e) => {
                eprintln!("[Autotest Worker Error]: {e}");
            }
            HostNotification::Terminated => {}
        }
    }
}

pub fn run_autotest_headless(
    session: &mut AppSession,
    script: &AutotestScript,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!(
        "[Autotest Headless] Starting execution of {} actions...",
        script.actions.len()
    );

    for (idx, action) in script.actions.iter().enumerate() {
        pump_session_mutations(session);

        match action {
            AutotestAction::WaitMs { duration_ms } => {
                let deadline = Instant::now() + Duration::from_millis(*duration_ms);
                while Instant::now() < deadline {
                    pump_session_mutations(session);
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
            AutotestAction::Click { component } => {
                if find_component(&session.ui_tree, component).is_none() {
                    return Err(format!("Component '{component}' not found in UI tree").into());
                }
                println!("[Autotest Headless] Action {idx}: Click '{component}'");
                session.send_event(UiEvent::ButtonClick {
                    component_name: component.clone(),
                })?;

                let settle = Instant::now() + Duration::from_millis(50);
                while Instant::now() < settle {
                    pump_session_mutations(session);
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
            AutotestAction::SetValue { component, value } => {
                let parsed = parse_property_value(value);
                if let Some(comp) = find_component_mut(&mut session.ui_tree, component) {
                    let _ = comp.set_property("Value", parsed.clone());
                }
                println!("[Autotest Headless] Action {idx}: SetValue '{component}' = '{value}'");
                session.send_event(UiEvent::ValueChanged {
                    component_name: component.clone(),
                    new_val: parsed,
                })?;

                let settle = Instant::now() + Duration::from_millis(50);
                while Instant::now() < settle {
                    pump_session_mutations(session);
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
            AutotestAction::ExpectProperty {
                component,
                property,
                expected,
            } => {
                println!(
                    "[Autotest Headless] Action {idx}: ExpectProperty '{component}.{property}' == '{expected}'"
                );
                let deadline = Instant::now() + Duration::from_millis(3000);
                let mut matched = false;

                while Instant::now() < deadline {
                    pump_session_mutations(session);
                    if let Some(comp) = find_component(&session.ui_tree, component) {
                        if let Some(actual) = comp.get_property(property) {
                            if matches_property_value(actual, expected) {
                                matched = true;
                                break;
                            }
                        }
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }

                if !matched {
                    let actual = find_component(&session.ui_tree, component)
                        .and_then(|c| c.get_property(property))
                        .map(|v| format!("{v:?}"))
                        .unwrap_or_else(|| "<none>".to_string());
                    return Err(format!(
                        "ExpectProperty mismatch for '{component}.{property}': expected '{expected}', got '{actual}'"
                    )
                    .into());
                }
            }
            AutotestAction::Screenshot { path } => {
                println!(
                    "[Autotest Headless] Action {idx}: Saving screenshot to {:?}",
                    path
                );
                save_screenshot_png(&session.ui_tree, path)?;
            }
            AutotestAction::Close => {
                println!("[Autotest Headless] Action {idx}: Close session");
                session.shutdown();
                return Ok(());
            }
        }
    }

    session.shutdown();
    println!("[Autotest Headless] All actions completed successfully.");
    Ok(())
}
