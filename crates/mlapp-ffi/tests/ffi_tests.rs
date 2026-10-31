use mlapp_ffi::{
    mlapp_package_check_compat_json, mlapp_package_check_compat_string, mlapp_package_free,
    mlapp_package_get_manifest_json, mlapp_package_get_manifest_string, mlapp_package_open_bytes,
    mlapp_session_launch, mlapp_session_send_button_click, mlapp_session_shutdown,
    mlapp_string_free,
};
use std::ffi::CStr;
use std::ptr;

#[test]
fn test_c_abi_package_and_session_flow_with_handle_ids() {
    let candidate_paths = [
        "fixtures/sample_apps/r2021a_sample.mlapp",
        "crates/mlapp-container/fixtures/sample_apps/r2021a_sample.mlapp",
        "../../fixtures/sample_apps/r2021a_sample.mlapp",
    ];

    let mut bytes = None;
    for path in candidate_paths {
        if let Ok(b) = std::fs::read(path) {
            bytes = Some(b);
            break;
        }
    }
    let bytes = bytes.expect("read sample app bytes");

    unsafe {
        let mut package_id: u64 = 0;
        let res = mlapp_package_open_bytes(bytes.as_ptr(), bytes.len(), &mut package_id);
        assert_eq!(res, 0);
        assert!(package_id > 0);

        // Get manifest JSON into buffer
        let mut buf = [0u8; 1024];
        let m_res =
            mlapp_package_get_manifest_json(package_id, buf.as_mut_ptr() as *mut _, buf.len());
        assert_eq!(m_res, 0);
        let manifest_str = CStr::from_ptr(buf.as_ptr() as *const _).to_str().unwrap();
        assert!(manifest_str.contains("FixtureApp_R2021a"));

        // Get manifest string directly and free
        let mut allocated_str: *mut std::os::raw::c_char = ptr::null_mut();
        let m_str_res = mlapp_package_get_manifest_string(package_id, &mut allocated_str);
        assert_eq!(m_str_res, 0);
        assert!(!allocated_str.is_null());
        let str_val = CStr::from_ptr(allocated_str).to_str().unwrap();
        assert!(str_val.contains("FixtureApp_R2021a"));
        mlapp_string_free(allocated_str);

        // Check compatibility JSON into buffer
        let mut compat_buf = [0u8; 2048];
        let c_res = mlapp_package_check_compat_json(
            package_id,
            compat_buf.as_mut_ptr() as *mut _,
            compat_buf.len(),
        );
        assert_eq!(c_res, 0);
        let compat_str = CStr::from_ptr(compat_buf.as_ptr() as *const _)
            .to_str()
            .unwrap();
        assert!(compat_str.contains("compatibility_score"));

        // Check compatibility string directly and free
        let mut allocated_compat: *mut std::os::raw::c_char = ptr::null_mut();
        let c_str_res = mlapp_package_check_compat_string(package_id, &mut allocated_compat);
        assert_eq!(c_str_res, 0);
        assert!(!allocated_compat.is_null());
        let compat_val = CStr::from_ptr(allocated_compat).to_str().unwrap();
        assert!(compat_val.contains("compatibility_score"));
        mlapp_string_free(allocated_compat);

        // Launch session
        let mut session_id: u64 = 0;
        let s_res = mlapp_session_launch(package_id, &mut session_id);
        assert_eq!(s_res, 0);
        assert!(session_id > 0);

        // Send click
        let comp_name = std::ffi::CString::new("StartBtn").unwrap();
        let click_res = mlapp_session_send_button_click(session_id, comp_name.as_ptr());
        assert_eq!(click_res, 0);

        // Shutdown session and free package
        assert_eq!(mlapp_session_shutdown(session_id), 0);
        assert_eq!(mlapp_package_free(package_id), 0);

        // Accessing freed IDs must return -1 safely (no UAF or crash)
        assert_eq!(
            mlapp_package_get_manifest_json(package_id, buf.as_mut_ptr() as *mut _, buf.len()),
            -1
        );
        assert_eq!(
            mlapp_session_send_button_click(session_id, comp_name.as_ptr()),
            -1
        );
    }
}

#[test]
fn test_c_abi_null_and_invalid_inputs_with_handle_ids() {
    unsafe {
        // Null pointers on open
        let mut out_pkg: u64 = 0;
        assert_eq!(mlapp_package_open_bytes(ptr::null(), 10, &mut out_pkg), -1);
        let dummy = [1u8, 2, 3];
        assert_eq!(
            mlapp_package_open_bytes(dummy.as_ptr(), 0, &mut out_pkg),
            -1
        );
        assert_eq!(
            mlapp_package_open_bytes(dummy.as_ptr(), 3, ptr::null_mut()),
            -1
        );

        // Null free functions (must safely do nothing, no panics)
        assert_eq!(mlapp_package_free(0), 0);
        mlapp_string_free(ptr::null_mut());
        assert_eq!(mlapp_session_shutdown(0), 0);

        // Invalid integer ID queries
        let mut buf = [0u8; 128];
        assert_eq!(
            mlapp_package_get_manifest_json(999999, buf.as_mut_ptr() as *mut _, 128),
            -1
        );
        let mut out_str: *mut std::os::raw::c_char = ptr::null_mut();
        assert_eq!(mlapp_package_get_manifest_string(999999, &mut out_str), -1);
        assert_eq!(
            mlapp_package_check_compat_json(999999, buf.as_mut_ptr() as *mut _, 128),
            -1
        );
        assert_eq!(mlapp_package_check_compat_string(999999, &mut out_str), -1);

        // Null session operations
        let mut out_session: u64 = 0;
        assert_eq!(mlapp_session_launch(0, &mut out_session), -1);
        assert_eq!(mlapp_session_launch(999999, &mut out_session), -1);
        assert_eq!(mlapp_session_send_button_click(0, ptr::null()), -1);
        assert_eq!(mlapp_session_send_button_click(999999, ptr::null()), -1);
    }
}
