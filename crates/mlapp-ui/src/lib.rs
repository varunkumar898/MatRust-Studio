//! Typed UI Component Tree, Geometry Mapping, and XML Parser for App Designer.

#![deny(clippy::unwrap_used)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UiError {
    #[error("XML parsing error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("Property error: {0}")]
    Property(String),

    #[error("Component '{0}' not found in UI tree")]
    ComponentNotFound(String),

    #[error("Type mismatch for property '{prop}' on component '{comp}': expected {expected}, got {actual}")]
    TypeMismatch {
        comp: String,
        prop: String,
        expected: String,
        actual: String,
    },

    #[error("XML nesting depth limit exceeded ({0})")]
    MaxDepthExceeded(usize),
}

/// 2D rectangular bounds [x, y, width, height]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// Coordinate transformation between MATLAB and standard GUI renderers.
///
/// MATLAB App Designer origin:
/// - (x, y) = (0, 0) is at the BOTTOM-LEFT of the parent container.
///
/// Standard GUI Renderers (egui, Web, Tauri):
/// - (x, y) = (0, 0) is at the TOP-LEFT of the parent container.
///
/// Transformation:
/// - x_top_left = x_bottom_left
/// - y_top_left = parent_height - (y_bottom_left + element_height)
pub fn matlab_to_top_left_geometry(matlab_rect: Rect, parent_height: f64) -> Rect {
    let top_y = parent_height - (matlab_rect.y + matlab_rect.height);
    Rect {
        x: matlab_rect.x,
        y: top_y,
        width: matlab_rect.width,
        height: matlab_rect.height,
    }
}

/// Coordinate transformation for top-level UIFigure windows.
///
/// In MATLAB, a figure's `Position = [left, bottom, width, height]` is defined relative
/// to the screen display with origin (0, 0) at the bottom-left of the monitor.
/// When rendered within a window or canvas, the figure's client area root origin is (0, 0).
/// When positioned on a desktop display with a known screen height (e.g. 1080),
/// the screen position translates to top-left desktop space:
/// `screen_top_y = screen_height - (fig_rect.y + fig_rect.height)`.
pub fn figure_to_top_left_geometry(fig_rect: Rect, screen_height: Option<f64>) -> Rect {
    if let Some(sh) = screen_height {
        let top_y = sh - (fig_rect.y + fig_rect.height);
        Rect {
            x: fig_rect.x,
            y: if top_y < 0.0 { 0.0 } else { top_y },
            width: fig_rect.width,
            height: fig_rect.height,
        }
    } else {
        Rect {
            x: 0.0,
            y: 0.0,
            width: fig_rect.width,
            height: fig_rect.height,
        }
    }
}

/// Reverse transformation from top-left to MATLAB bottom-left geometry.
pub fn top_left_to_matlab_geometry(top_left_rect: Rect, parent_height: f64) -> Rect {
    let matlab_y = parent_height - (top_left_rect.y + top_left_rect.height);
    Rect {
        x: top_left_rect.x,
        y: matlab_y,
        width: top_left_rect.width,
        height: top_left_rect.height,
    }
}

/// Supported UI component kinds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentKind {
    UIFigure,
    Panel,
    TabGroup,
    Tab,
    GridLayout,
    Button,
    StateButton,
    Label,
    NumericEditField,
    TextEditField,
    TextArea,
    DropDown,
    ListBox,
    CheckBox,
    RadioButton,
    ButtonGroup,
    Slider,
    Spinner,
    Switch,
    Table,
    UIAxes,
    Menu,
    Image,
    Unknown(String),
}

/// Strongly-typed property values for components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PropertyValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Position(Rect),
    Color([f32; 4]),
    List(Vec<String>),
    Unknown(String),
}

/// Container layout mode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LayoutType {
    None,
    Grid { rows: usize, cols: usize },
    Flow,
}

/// Component node inside the hierarchical UI tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub kind: ComponentKind,
    pub name: String,
    pub parent: Option<String>,
    pub position: Rect,
    pub properties: HashMap<String, PropertyValue>,
    pub callbacks: HashMap<String, String>,
    pub children: Vec<Component>,
    pub layout: LayoutType,
}

impl Component {
    pub fn new(kind: ComponentKind, name: &str) -> Self {
        Self {
            kind,
            name: name.to_string(),
            parent: None,
            position: Rect::default(),
            properties: HashMap::new(),
            callbacks: HashMap::new(),
            children: Vec::new(),
            layout: LayoutType::None,
        }
    }

    /// Read property with type validation
    pub fn get_property(&self, key: &str) -> Option<&PropertyValue> {
        self.properties.get(key)
    }

    /// Write property with type validation
    pub fn set_property(&mut self, key: &str, val: PropertyValue) -> Result<(), UiError> {
        // Enforce validation if existing property has a type
        if let Some(existing) = self.properties.get(key) {
            match (existing, &val) {
                (PropertyValue::String(_), PropertyValue::String(_))
                | (PropertyValue::Number(_), PropertyValue::Number(_))
                | (PropertyValue::Boolean(_), PropertyValue::Boolean(_))
                | (PropertyValue::Position(_), PropertyValue::Position(_))
                | (PropertyValue::Color(_), PropertyValue::Color(_))
                | (PropertyValue::List(_), PropertyValue::List(_)) => {}
                _ => {
                    return Err(UiError::TypeMismatch {
                        comp: self.name.clone(),
                        prop: key.to_string(),
                        expected: format!("{:?}", existing),
                        actual: format!("{:?}", val),
                    });
                }
            }
        }
        self.properties.insert(key.to_string(), val);
        Ok(())
    }

    /// Recursively find a component by its name in the tree
    pub fn find_by_name(&self, target_name: &str) -> Option<&Component> {
        if self.name == target_name {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_by_name(target_name) {
                return Some(found);
            }
        }
        None
    }

    /// Recursively find a mutable reference to a component by its name
    pub fn find_by_name_mut(&mut self, target_name: &str) -> Option<&mut Component> {
        if self.name == target_name {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_by_name_mut(target_name) {
                return Some(found);
            }
        }
        None
    }
}

/// Iterative drop implementation to prevent stack overflow when dropping deep component trees.
impl Drop for Component {
    fn drop(&mut self) {
        let mut stack = std::mem::take(&mut self.children);
        while let Some(mut child) = stack.pop() {
            let next = std::mem::take(&mut child.children);
            stack.extend(next);
        }
    }
}

/// Parses MATLAB App Designer `appdesigner/app.xml` into a Component tree.
pub fn parse_app_xml(xml: &str) -> Result<Component, UiError> {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut root_fig = None;
    let mut stack: Vec<Component> = Vec::new();

    while let Ok(event) = reader.read_event() {
        match event {
            Event::Start(ref e) | Event::Empty(ref e) => {
                let is_empty = matches!(event, Event::Empty(_));
                let tag_bytes = e.name();
                let tag_name = String::from_utf8_lossy(tag_bytes.as_ref()).to_string();
                let local_name = tag_name
                    .split(':')
                    .next_back()
                    .unwrap_or(&tag_name)
                    .to_string();

                if local_name.eq_ignore_ascii_case("appdata")
                    || local_name.eq_ignore_ascii_case("model")
                {
                    continue;
                }

                let mut name = local_name.clone();
                let mut position = Rect::default();
                let mut props = HashMap::new();
                let mut callbacks = HashMap::new();

                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let val = String::from_utf8_lossy(&attr.value).to_string();

                    let lower_key = key.to_lowercase();
                    match lower_key.as_str() {
                        "name" | "tag" => name = val,
                        "position" => {
                            if let Some(r) = parse_position_string(&val) {
                                position = r;
                                props.insert("Position".to_string(), PropertyValue::Position(r));
                            }
                        }
                        "text" | "title" | "placeholder" => {
                            props.insert(key, PropertyValue::String(val));
                        }
                        "value" => {
                            if let Ok(num) = val.parse::<f64>() {
                                props.insert(key, PropertyValue::Number(num));
                            } else {
                                props.insert(key, PropertyValue::String(val));
                            }
                        }
                        "onpushed" | "buttonpushedfcn" | "valuechangedfcn" => {
                            callbacks.insert(lower_key, val);
                        }
                        _ => {
                            props.insert(key, PropertyValue::String(val));
                        }
                    }
                }

                let kind = match_component_kind(&local_name);
                let comp = Component {
                    kind,
                    name,
                    parent: stack.last().map(|p| p.name.clone()),
                    position,
                    properties: props,
                    callbacks,
                    children: Vec::new(),
                    layout: LayoutType::None,
                };

                if is_empty {
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(comp);
                    } else if root_fig.is_none() {
                        root_fig = Some(comp);
                    }
                } else {
                    const MAX_XML_DEPTH: usize = 1000;
                    if stack.len() >= MAX_XML_DEPTH {
                        return Err(UiError::MaxDepthExceeded(stack.len()));
                    }
                    stack.push(comp);
                }
            }
            Event::End(e) => {
                let tag_bytes = e.name();
                let tag_name = String::from_utf8_lossy(tag_bytes.as_ref()).to_string();
                let local_name = tag_name
                    .split(':')
                    .next_back()
                    .unwrap_or(&tag_name)
                    .to_string();

                if local_name.eq_ignore_ascii_case("appdata")
                    || local_name.eq_ignore_ascii_case("model")
                {
                    continue;
                }

                if let Some(finished) = stack.pop() {
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(finished);
                    } else if root_fig.is_none() {
                        root_fig = Some(finished);
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    root_fig
        .ok_or_else(|| UiError::ComponentNotFound("Root UIFigure not found in app.xml".to_string()))
}

fn parse_position_string(val: &str) -> Option<Rect> {
    let parts: Vec<f64> = val
        .split(|c: char| c.is_whitespace() || c == ',' || c == '[' || c == ']')
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<f64>().ok())
        .collect();

    if parts.len() >= 4 {
        Some(Rect::new(parts[0], parts[1], parts[2], parts[3]))
    } else {
        None
    }
}

fn match_component_kind(tag: &str) -> ComponentKind {
    match tag.to_lowercase().as_str() {
        "uifigure" | "figure" => ComponentKind::UIFigure,
        "panel" | "uipanel" => ComponentKind::Panel,
        "tabgroup" | "uitabgroup" => ComponentKind::TabGroup,
        "tab" | "uitab" => ComponentKind::Tab,
        "gridlayout" | "uigridlayout" => ComponentKind::GridLayout,
        "button" | "uibutton" => ComponentKind::Button,
        "statebutton" | "uistatebutton" => ComponentKind::StateButton,
        "label" | "uilabel" => ComponentKind::Label,
        "editfield" | "uieditfield" => ComponentKind::TextEditField,
        "numericeditfield" => ComponentKind::NumericEditField,
        "textarea" | "uitextarea" => ComponentKind::TextArea,
        "dropdown" | "uidropdown" => ComponentKind::DropDown,
        "listbox" | "uilistbox" => ComponentKind::ListBox,
        "checkbox" | "uicheckbox" => ComponentKind::CheckBox,
        "radiobutton" | "uiradiobutton" => ComponentKind::RadioButton,
        "buttongroup" | "uibuttongroup" => ComponentKind::ButtonGroup,
        "slider" | "uislider" => ComponentKind::Slider,
        "spinner" | "uispinner" => ComponentKind::Spinner,
        "switch" | "uiswitch" => ComponentKind::Switch,
        "table" | "uitable" => ComponentKind::Table,
        "uiaxes" | "axes" => ComponentKind::UIAxes,
        "menu" | "uimenu" => ComponentKind::Menu,
        "image" | "uiimage" => ComponentKind::Image,
        _ => ComponentKind::Unknown(tag.to_string()),
    }
}
