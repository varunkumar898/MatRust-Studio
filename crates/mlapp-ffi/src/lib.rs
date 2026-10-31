//! C ABI interface for embedding `mlapp-rs` into external host applications.
//!
//! Exposes integer-based handle IDs (`u64`) instead of raw pointers to guarantee memory safety,
//! eliminate dangling pointers, and prevent use-after-free or pointer forgery across the FFI boundary.

#![deny(clippy::unwrap_used)]

use mlapp_container::AppContainer;
use mlapp_host::{AppSession, HostConfig, UiEvent};
use mlapp_lang::{analyze_compatibility, extract_code_from_document_xml, parse_app_classdef};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::io::Cursor;
use std::os::raw::c_char;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{OnceLock, RwLock};

static NEXT_PACKAGE_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

fn packages_registry() -> &'static RwLock<HashMap<u64, Vec<u8>>> {
    static REGISTRY: OnceLock<RwLock<HashMap<u64, Vec<u8>>>> = OnceLock::new();
    REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

fn sessions_registry() -> &'static RwLock<HashMap<u64, AppSession>> {
    static REGISTRY: OnceLock<RwLock<HashMap<u64, AppSession>>> = OnceLock::new();
    REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Frees a C string allocated by `mlapp-ffi`.
///
/// # Safety
/// `s` must be a pointer returned by an allocation function in this crate or null.
#[no_mangle]
pub unsafe extern "C" fn mlapp_string_free(s: *mut c_char) {
    if !s.is_null() {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            drop(CString::from_raw(s));
        }));
    }
}

/// Opens a package from a byte buffer and registers it in the package handle registry.
/// Returns 0 on success with the assigned package ID written to `*out_package_id`, or -1 on error.
///
/// # Safety
/// - `data` must point to a valid readable byte buffer of length `len`.
/// - `out_package_id` must point to valid writable uint64 memory.
#[no_mangle]
pub unsafe extern "C" fn mlapp_package_open_bytes(
    data: *const u8,
    len: usize,
    out_package_id: *mut u64,
) -> i32 {
    let res = catch_unwind(AssertUnwindSafe(|| {
        if data.is_null() || out_package_id.is_null() || len == 0 {
            return -1;
        }

        let slice = std::slice::from_raw_parts(data, len);
        let bytes = slice.to_vec();

        // Verify container can be parsed
        if AppContainer::open(Cursor::new(bytes.clone())).is_err() {
            return -1;
        }

        let pkg_id = NEXT_PACKAGE_ID.fetch_add(1, Ordering::SeqCst);
        let registry = packages_registry();
        let mut lock = registry
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        lock.insert(pkg_id, bytes);

        *out_package_id = pkg_id;
        0
    }));
    res.unwrap_or(-1)
}

/// Frees/unregisters a package handle from the registry. Returns 0 on success, -1 if ID not found.
#[no_mangle]
pub extern "C" fn mlapp_package_free(package_id: u64) -> i32 {
    let res = catch_unwind(AssertUnwindSafe(|| {
        if package_id == 0 {
            return 0; // Null safe
        }
        let registry = packages_registry();
        let mut lock = registry
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if lock.remove(&package_id).is_some() {
            0
        } else {
            -1
        }
    }));
    res.unwrap_or(-1)
}

/// Retrieves the package manifest as a JSON string into `out_buf`.
///
/// # Safety
/// - `out_buf` must point to writable memory of at least `max_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn mlapp_package_get_manifest_json(
    package_id: u64,
    out_buf: *mut c_char,
    max_len: usize,
) -> i32 {
    let res = catch_unwind(AssertUnwindSafe(|| {
        if package_id == 0 || out_buf.is_null() || max_len == 0 {
            return -1;
        }

        let registry = packages_registry();
        let lock = registry
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let bytes = match lock.get(&package_id) {
            Some(b) => b.clone(),
            None => return -1,
        };

        let mut container = match AppContainer::open(Cursor::new(bytes)) {
            Ok(c) => c,
            Err(_) => return -1,
        };

        let report = match container.inspect() {
            Ok(r) => r,
            Err(_) => return -1,
        };

        let json = match serde_json::to_string(&report.manifest) {
            Ok(s) => s,
            Err(_) => return -1,
        };

        let c_str = match CString::new(json) {
            Ok(s) => s,
            Err(_) => return -1,
        };

        let c_bytes = c_str.as_bytes_with_nul();
        if c_bytes.len() > max_len {
            return -1;
        }

        ptr::copy_nonoverlapping(c_bytes.as_ptr() as *const c_char, out_buf, c_bytes.len());
        0
    }));
    res.unwrap_or(-1)
}

/// Retrieves the package manifest as a newly allocated C string.
/// The caller is responsible for freeing it via `mlapp_string_free`.
///
/// # Safety
/// - `out_str` must point to valid writable pointer memory.
#[no_mangle]
pub unsafe extern "C" fn mlapp_package_get_manifest_string(
    package_id: u64,
    out_str: *mut *mut c_char,
) -> i32 {
    let res = catch_unwind(AssertUnwindSafe(|| {
        if package_id == 0 || out_str.is_null() {
            return -1;
        }

        let registry = packages_registry();
        let lock = registry
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let bytes = match lock.get(&package_id) {
            Some(b) => b.clone(),
            None => return -1,
        };

        let mut container = match AppContainer::open(Cursor::new(bytes)) {
            Ok(c) => c,
            Err(_) => return -1,
        };

        let report = match container.inspect() {
            Ok(r) => r,
            Err(_) => return -1,
        };

        let json = match serde_json::to_string(&report.manifest) {
            Ok(s) => s,
            Err(_) => return -1,
        };

        let c_str = match CString::new(json) {
            Ok(s) => s,
            Err(_) => return -1,
        };

        *out_str = c_str.into_raw();
        0
    }));
    res.unwrap_or(-1)
}

/// Statically analyzes the package compatibility and writes the report as JSON into `out_buf`.
///
/// # Safety
/// - `out_buf` must point to writable memory of at least `max_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn mlapp_package_check_compat_json(
    package_id: u64,
    out_buf: *mut c_char,
    max_len: usize,
) -> i32 {
    let res = catch_unwind(AssertUnwindSafe(|| {
        if package_id == 0 || out_buf.is_null() || max_len == 0 {
            return -1;
        }

        let registry = packages_registry();
        let lock = registry
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let bytes = match lock.get(&package_id) {
            Some(b) => b.clone(),
            None => return -1,
        };

        let mut container = match AppContainer::open(Cursor::new(bytes)) {
            Ok(c) => c,
            Err(_) => return -1,
        };

        let doc_xml = match container.read_entry_to_string("matlab/document.xml") {
            Ok(s) => s,
            Err(_) => return -1,
        };

        let code = match extract_code_from_document_xml(&doc_xml) {
            Ok(s) => s,
            Err(_) => return -1,
        };

        let class_def = match parse_app_classdef(&code) {
            Ok(cd) => cd,
            Err(_) => return -1,
        };

        let report = analyze_compatibility(&class_def);
        let json = match serde_json::to_string(&report) {
            Ok(s) => s,
            Err(_) => return -1,
        };

        let c_str = match CString::new(json) {
            Ok(s) => s,
            Err(_) => return -1,
        };

        let c_bytes = c_str.as_bytes_with_nul();
        if c_bytes.len() > max_len {
            return -1;
        }

        ptr::copy_nonoverlapping(c_bytes.as_ptr() as *const c_char, out_buf, c_bytes.len());
        0
    }));
    res.unwrap_or(-1)
}

/// Statically analyzes the package compatibility and returns the report as a newly allocated C string.
/// The caller is responsible for freeing it via `mlapp_string_free`.
///
/// # Safety
/// - `out_str` must point to valid writable pointer memory.
#[no_mangle]
pub unsafe extern "C" fn mlapp_package_check_compat_string(
    package_id: u64,
    out_str: *mut *mut c_char,
) -> i32 {
    let res = catch_unwind(AssertUnwindSafe(|| {
        if package_id == 0 || out_str.is_null() {
            return -1;
        }

        let registry = packages_registry();
        let lock = registry
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let bytes = match lock.get(&package_id) {
            Some(b) => b.clone(),
            None => return -1,
        };

        let mut container = match AppContainer::open(Cursor::new(bytes)) {
            Ok(c) => c,
            Err(_) => return -1,
        };

        let doc_xml = match container.read_entry_to_string("matlab/document.xml") {
            Ok(s) => s,
            Err(_) => return -1,
        };

        let code = match extract_code_from_document_xml(&doc_xml) {
            Ok(s) => s,
            Err(_) => return -1,
        };

        let class_def = match parse_app_classdef(&code) {
            Ok(cd) => cd,
            Err(_) => return -1,
        };

        let report = analyze_compatibility(&class_def);
        let json = match serde_json::to_string(&report) {
            Ok(s) => s,
            Err(_) => return -1,
        };

        let c_str = match CString::new(json) {
            Ok(s) => s,
            Err(_) => return -1,
        };

        *out_str = c_str.into_raw();
        0
    }));
    res.unwrap_or(-1)
}

/// Launches an application session for the given package ID.
/// Returns 0 on success with the assigned session ID written to `*out_session_id`, or -1 on error.
///
/// # Safety
/// - `out_session_id` must point to valid writable uint64 memory.
#[no_mangle]
pub unsafe extern "C" fn mlapp_session_launch(package_id: u64, out_session_id: *mut u64) -> i32 {
    let res = catch_unwind(AssertUnwindSafe(|| {
        if package_id == 0 || out_session_id.is_null() {
            return -1;
        }

        let registry = packages_registry();
        let lock = registry
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let bytes = match lock.get(&package_id) {
            Some(b) => b.clone(),
            None => return -1,
        };

        let session = match AppSession::launch(Cursor::new(bytes), HostConfig::default()) {
            Ok(s) => s,
            Err(_) => return -1,
        };

        let session_id = NEXT_SESSION_ID.fetch_add(1, Ordering::SeqCst);
        let s_registry = sessions_registry();
        let mut s_lock = s_registry
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        s_lock.insert(session_id, session);

        *out_session_id = session_id;
        0
    }));
    res.unwrap_or(-1)
}

/// Dispatches a button click event to a component in the session. Returns 0 on success, -1 on error.
///
/// # Safety
/// - `component_name` must point to a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn mlapp_session_send_button_click(
    session_id: u64,
    component_name: *const c_char,
) -> i32 {
    let res = catch_unwind(AssertUnwindSafe(|| {
        if session_id == 0 || component_name.is_null() {
            return -1;
        }

        let name_str = match CStr::from_ptr(component_name).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return -1,
        };

        let s_registry = sessions_registry();
        let s_lock = s_registry
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let session = match s_lock.get(&session_id) {
            Some(s) => s,
            None => return -1,
        };

        let event = UiEvent::ButtonClick {
            component_name: name_str,
        };

        match session.send_event(event) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }));
    res.unwrap_or(-1)
}

/// Gracefully shuts down and unregisters an active session. Returns 0 on success, -1 if ID not found.
#[no_mangle]
pub extern "C" fn mlapp_session_shutdown(session_id: u64) -> i32 {
    let res = catch_unwind(AssertUnwindSafe(|| {
        if session_id == 0 {
            return 0; // Null safe
        }

        let s_registry = sessions_registry();
        let mut s_lock = s_registry
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(mut session) = s_lock.remove(&session_id) {
            session.shutdown();
            0
        } else {
            -1
        }
    }));
    res.unwrap_or(-1)
}
