// Written justification for unsafe code in mlapp-py:
// PyO3 macros and CPython C-API FFI wrappers internally require unsafe code blocks to interact
// with the Python runtime, reference counts, and GIL. No manual pointer arithmetic or unsanitized
// unsafe blocks are introduced beyond standard idiomatic PyO3 extension module definitions.

use mlapp_container::AppContainer;
use mlapp_host::{AppSession, HostConfig, UiEvent};
use mlapp_ui::PropertyValue;
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::AsPyPointer;
use rustlab_eval::Interpreter;
use rustlab_math::Value;
use std::cell::RefCell;
use std::io::Cursor;
use std::path::Path;

create_exception!(matrust, MlappError, PyException);
create_exception!(matrust, PackageError, MlappError);
create_exception!(matrust, SessionError, MlappError);

#[derive(Debug, thiserror::Error)]
pub enum MatRustError {
    #[error("{0}")]
    Package(String),
    #[error("{0}")]
    Session(String),
    #[error("{0}")]
    Py(#[from] PyErr),
}

impl From<MatRustError> for PyErr {
    fn from(err: MatRustError) -> PyErr {
        match err {
            MatRustError::Package(s) => PyErr::new::<PackageError, _>(s),
            MatRustError::Session(s) => PyErr::new::<SessionError, _>(s),
            MatRustError::Py(e) => e,
        }
    }
}

fn value_to_py(py: Python<'_>, val: &Value) -> PyResult<PyObject> {
    match val {
        Value::Double(arr) => {
            let slice = arr.as_slice();
            let shape = arr.shape();
            if shape == [1, 1] || slice.len() == 1 {
                if let Some(&first) = slice.first() {
                    return Ok(first.into_py(py));
                }
            }
            if shape.len() == 2 && shape[0] > 1 && shape[1] > 1 {
                let rows = shape[0];
                let cols = shape[1];
                let outer = PyList::empty_bound(py);
                for r in 0..rows {
                    let inner = PyList::empty_bound(py);
                    for c in 0..cols {
                        // column-major index: r + c * rows
                        let idx = r + c * rows;
                        inner.append(slice[idx])?;
                    }
                    outer.append(inner)?;
                }
                return Ok(outer.into_py(py));
            }
            let list = PyList::new_bound(py, slice);
            Ok(list.into_py(py))
        }
        Value::Single(arr) => {
            let list = PyList::new_bound(py, arr.as_slice());
            Ok(list.into_py(py))
        }
        Value::Logical(arr) => {
            let slice = arr.as_slice();
            let shape = arr.shape();
            if shape == [1, 1] || slice.len() == 1 {
                if let Some(&first) = slice.first() {
                    return Ok(first.into_py(py));
                }
            }
            let list = PyList::new_bound(py, slice);
            Ok(list.into_py(py))
        }
        Value::Char(arr) => {
            let s: String = arr.as_slice().iter().collect();
            Ok(s.into_py(py))
        }
        Value::Cell(arr) => {
            let list = PyList::empty_bound(py);
            for item in arr.as_slice() {
                let py_item = value_to_py(py, item)?;
                list.append(py_item)?;
            }
            Ok(list.into_py(py))
        }
        Value::Struct(st) => {
            let dict = PyDict::new_bound(py);
            if let Some(first_record) = st.data.first() {
                for (k, v) in first_record {
                    let py_val = value_to_py(py, v)?;
                    dict.set_item(k, py_val)?;
                }
            }
            Ok(dict.into_py(py))
        }
        Value::Int8(arr) => Ok(PyList::new_bound(py, arr.as_slice()).into_py(py)),
        Value::Int16(arr) => Ok(PyList::new_bound(py, arr.as_slice()).into_py(py)),
        Value::Int32(arr) => Ok(PyList::new_bound(py, arr.as_slice()).into_py(py)),
        Value::Int64(arr) => Ok(PyList::new_bound(py, arr.as_slice()).into_py(py)),
        Value::UInt8(arr) => Ok(PyList::new_bound(py, arr.as_slice()).into_py(py)),
        Value::UInt16(arr) => Ok(PyList::new_bound(py, arr.as_slice()).into_py(py)),
        Value::UInt32(arr) => Ok(PyList::new_bound(py, arr.as_slice()).into_py(py)),
        Value::UInt64(arr) => Ok(PyList::new_bound(py, arr.as_slice()).into_py(py)),
        Value::Sparse(s) => {
            let dict = PyDict::new_bound(py);
            dict.set_item("rows", s.rows())?;
            dict.set_item("cols", s.cols())?;
            dict.set_item("nnz", s.nnz())?;
            Ok(dict.into_py(py))
        }
        Value::FunctionHandle(h) => Ok(format!("{h:?}").into_py(py)),
        Value::ComplexDouble(arr) => {
            let slice = arr.as_slice();
            let complex_ctor = py.import_bound("builtins")?.getattr("complex")?;
            if arr.shape() == [1, 1] || slice.len() == 1 {
                if let Some(&first) = slice.first() {
                    let py_c = complex_ctor.call1((first.re, first.im))?;
                    return Ok(py_c.into_py(py));
                }
            }
            let list = PyList::empty_bound(py);
            for c in slice {
                let py_c = complex_ctor.call1((c.re, c.im))?;
                list.append(py_c)?;
            }
            Ok(list.into_py(py))
        }
        Value::String(arr) => {
            let slice = arr.as_slice();
            if arr.shape() == [1, 1] || slice.len() == 1 {
                if let Some(first) = slice.first() {
                    return Ok(first.clone().into_py(py));
                }
            }
            let list = PyList::new_bound(py, slice);
            Ok(list.into_py(py))
        }
    }
}

fn property_value_to_py(py: Python<'_>, pv: &PropertyValue) -> PyResult<PyObject> {
    match pv {
        PropertyValue::String(s) => Ok(s.clone().into_py(py)),
        PropertyValue::Number(n) => Ok(n.into_py(py)),
        PropertyValue::Boolean(b) => Ok(b.into_py(py)),
        PropertyValue::Position(rect) => {
            let list = PyList::new_bound(py, [rect.x, rect.y, rect.width, rect.height]);
            Ok(list.into_py(py))
        }
        PropertyValue::Color(c) => {
            let list = PyList::new_bound(py, c);
            Ok(list.into_py(py))
        }
        PropertyValue::List(items) => {
            let list = PyList::new_bound(py, items);
            Ok(list.into_py(py))
        }
        PropertyValue::Unknown(u) => Ok(u.clone().into_py(py)),
    }
}

fn py_to_property_value(obj: &Bound<'_, PyAny>) -> Result<PropertyValue, MatRustError> {
    if let Ok(b) = obj.extract::<bool>() {
        Ok(PropertyValue::Boolean(b))
    } else if let Ok(n) = obj.extract::<f64>() {
        Ok(PropertyValue::Number(n))
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(PropertyValue::String(s))
    } else if let Ok(list) = obj.downcast::<PyList>() {
        let mut strings = Vec::with_capacity(list.len());
        for item in list.iter() {
            let s = item.extract::<String>()?;
            strings.push(s);
        }
        Ok(PropertyValue::List(strings))
    } else {
        Err(MatRustError::Session(format!(
            "Unsupported Python type for PropertyValue: {obj:?}"
        )))
    }
}

/// Represents an inspected or unpacked MATLAB App Designer (.mlapp) container.
#[pyclass(name = "Package")]
pub struct PyPackage {
    path: String,
    raw_bytes: Vec<u8>,
}

#[pymethods]
impl PyPackage {
    /// Inspect the package and return a structured dictionary containing manifest,
    /// detected MATLAB release, total files, and component report.
    pub fn inspect(&self, py: Python<'_>) -> Result<PyObject, MatRustError> {
        let mut container = AppContainer::open(Cursor::new(&self.raw_bytes))
            .map_err(|e| MatRustError::Package(format!("Container error: {e}")))?;

        let report = container
            .inspect()
            .map_err(|e| MatRustError::Package(format!("Inspection error: {e}")))?;

        let json_str = serde_json::to_string(&report)
            .map_err(|e| MatRustError::Package(format!("Serialization error: {e}")))?;

        let json_mod = py.import_bound("json")?;
        let dict = json_mod.call_method1("loads", (json_str,))?;
        Ok(dict.into_py(py))
    }

    /// Safely extracts all files from the package into `dest_dir` with zip-slip mitigation.
    pub fn extract_all(&self, dest_dir: &str) -> Result<(), MatRustError> {
        let mut container = AppContainer::open(Cursor::new(&self.raw_bytes))
            .map_err(|e| MatRustError::Package(format!("Container error: {e}")))?;

        container
            .extract_all(Path::new(dest_dir))
            .map_err(|e| MatRustError::Package(format!("Extraction error: {e}")))?;

        Ok(())
    }

    /// Spawns a background AppSession worker and returns an interactive Session handle.
    pub fn create_session(&self) -> Result<PySession, MatRustError> {
        let session =
            AppSession::launch(Cursor::new(self.raw_bytes.clone()), HostConfig::default())
                .map_err(|e| MatRustError::Session(format!("Session launch error: {e}")))?;

        let mut interp = Interpreter::new();
        rustlab_stdlib::register_all(&mut interp);

        Ok(PySession {
            session: RefCell::new(Some(session)),
            interp: RefCell::new(interp),
        })
    }

    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    #[pyo3(signature = (_exc_type=None, _exc_value=None, _traceback=None))]
    fn __exit__(
        &self,
        _exc_type: Option<&Bound<'_, PyAny>>,
        _exc_value: Option<&Bound<'_, PyAny>>,
        _traceback: Option<&Bound<'_, PyAny>>,
    ) -> bool {
        false
    }

    fn __repr__(&self) -> String {
        format!(
            "<matrust.Package path='{}' size={} bytes>",
            self.path,
            self.raw_bytes.len()
        )
    }
}

/// Interactive session handle connected to an App Designer instance or interpreter.
#[pyclass(name = "Session", unsendable)]
pub struct PySession {
    session: RefCell<Option<AppSession>>,
    interp: RefCell<Interpreter>,
}

#[pymethods]
impl PySession {
    /// Send a UI event dictionary (e.g. `{"type": "button_click", "component": "btn"}`).
    pub fn send_event(&self, event: &Bound<'_, PyDict>) -> Result<(), MatRustError> {
        let session_borrow = self.session.borrow();
        let session = session_borrow
            .as_ref()
            .ok_or_else(|| MatRustError::Session("Session has already shut down".to_string()))?;

        let event_type: String = event
            .get_item("type")?
            .and_then(|v| v.extract::<String>().ok())
            .unwrap_or_else(|| "button_click".to_string());

        let component_name: String = event
            .get_item("component")?
            .or_else(|| event.get_item("component_name").ok().flatten())
            .and_then(|v| v.extract::<String>().ok())
            .ok_or_else(|| MatRustError::Session("Event missing 'component' key".to_string()))?;

        let ui_event = match event_type.to_ascii_lowercase().as_str() {
            "button_click" | "click" => UiEvent::ButtonClick { component_name },
            "value_changed" | "change" => {
                let val_obj = event.get_item("value")?.ok_or_else(|| {
                    MatRustError::Session("ValueChanged event missing 'value' key".to_string())
                })?;
                let pv = py_to_property_value(&val_obj)?;
                UiEvent::ValueChanged {
                    component_name,
                    new_val: pv,
                }
            }
            other => {
                return Err(MatRustError::Session(format!(
                    "Unknown event type: '{other}'. Expected 'button_click' or 'value_changed'."
                )));
            }
        };

        session
            .send_event(ui_event)
            .map_err(|e| MatRustError::Session(format!("Failed to send event: {e}")))?;

        Ok(())
    }

    /// Evaluates a MATLAB/RustLab expression or script statement, returning Python native types.
    pub fn eval(&self, py: Python<'_>, statement: &str) -> Result<PyObject, MatRustError> {
        let mut interp = self.interp.borrow_mut();

        match interp.eval_script(statement) {
            Ok(Some(val)) => Ok(value_to_py(py, &val)?),
            Ok(None) => Ok(py.None()),
            Err(e) => Err(MatRustError::Session(format!("Evaluation error: {e}"))),
        }
    }

    /// Retrieves a component property value from the active session.
    pub fn get_property(
        &self,
        py: Python<'_>,
        component: &str,
        prop: &str,
    ) -> Result<PyObject, MatRustError> {
        let session_borrow = self.session.borrow();
        let session = session_borrow
            .as_ref()
            .ok_or_else(|| MatRustError::Session("Session has already shut down".to_string()))?;

        if let Ok(state) = session.app_handle.state.lock() {
            if let Some(comp) = state.components.get(component) {
                if let Some(pv) = comp.get_property(prop) {
                    return Ok(property_value_to_py(py, pv)?);
                }
            }
        }

        if let Some(val) = session
            .app_handle
            .get_property(&format!("{component}.{prop}"))
        {
            return Ok(value_to_py(py, &val)?);
        }

        if let Some(val) = session.app_handle.get_property(prop) {
            return Ok(value_to_py(py, &val)?);
        }

        Ok(py.None())
    }

    /// Sets a component property value in the active session.
    pub fn set_property(
        &self,
        component: &str,
        prop: &str,
        value: &Bound<'_, PyAny>,
    ) -> Result<(), MatRustError> {
        let session_borrow = self.session.borrow();
        let session = session_borrow
            .as_ref()
            .ok_or_else(|| MatRustError::Session("Session has already shut down".to_string()))?;

        let pv = py_to_property_value(value)?;
        session
            .app_handle
            .set_component_property(component, prop, pv)
            .map_err(|e| MatRustError::Session(format!("Set property failed: {e}")))?;

        Ok(())
    }

    /// Gracefully terminates the session worker thread.
    pub fn shutdown(&self) -> Result<(), MatRustError> {
        if let Some(mut session) = self.session.borrow_mut().take() {
            session.shutdown();
        }
        Ok(())
    }

    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    #[pyo3(signature = (_exc_type=None, _exc_value=None, _traceback=None))]
    fn __exit__(
        &self,
        _exc_type: Option<&Bound<'_, PyAny>>,
        _exc_value: Option<&Bound<'_, PyAny>>,
        _traceback: Option<&Bound<'_, PyAny>>,
    ) -> Result<bool, MatRustError> {
        self.shutdown()?;
        Ok(false)
    }

    fn __repr__(&self) -> String {
        let is_active = self.session.borrow().is_some();
        format!("<matrust.Session active={is_active}>")
    }
}

/// Opens an App Designer package (.mlapp or .mlappinstall) file from disk.
#[pyfunction]
pub fn open_package(path: &str) -> Result<PyPackage, MatRustError> {
    let bytes = std::fs::read(path)
        .map_err(|e| MatRustError::Package(format!("Failed to read file '{path}': {e}")))?;

    Ok(PyPackage {
        path: path.to_string(),
        raw_bytes: bytes,
    })
}

/// The core `_matrust` CPython extension module.
#[pymodule]
fn _matrust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(open_package, m)?)?;
    m.add_class::<PyPackage>()?;
    m.add_class::<PySession>()?;
    m.add("MlappError", m.py().get_type_bound::<MlappError>())?;
    m.add("PackageError", m.py().get_type_bound::<PackageError>())?;
    m.add("SessionError", m.py().get_type_bound::<SessionError>())?;
    Ok(())
}
