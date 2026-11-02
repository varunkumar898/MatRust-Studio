//! Integration tests for the GNU Octave subprocess bridge (`mlapp-octave`).

use mlapp_octave::{
    check_call_safety, find_octave_binary, is_octave_available, is_recognized_safe, OctaveConfig,
    OctaveError, OctaveErrorInfo, OctaveRequest, OctaveResponse, OctaveSession, OctaveValue,
    BLOCKED_FUNCTIONS,
};
use rustlab_math::{DenseArray, StructArray, Value};

#[test]
fn test_octave_detection_functions() {
    let available = is_octave_available();
    let bin_path = find_octave_binary();
    assert_eq!(
        available,
        bin_path.is_some(),
        "is_octave_available() must match find_octave_binary().is_some()"
    );
    if let Some(path) = bin_path {
        println!("Detected GNU Octave binary at: {}", path.display());
        assert!(path.is_file(), "Detected binary path must exist on disk");
    } else {
        println!("No GNU Octave executable found on this host system.");
    }
}

#[test]
fn test_octave_value_roundtrip_all_types() {
    // 1. Double scalar & matrix
    let d_scalar = Value::scalar(42.5);
    let oct_d = OctaveValue::from_value(&d_scalar).expect("from_value scalar");
    let back_d = oct_d.to_value().expect("to_value scalar");
    assert_eq!(d_scalar, back_d);

    let d_mat = Value::Double(DenseArray::from_vec(vec![2, 2], vec![1.0, 3.0, 2.0, 4.0]).unwrap());
    let oct_mat = OctaveValue::from_value(&d_mat).expect("from_value matrix");
    let back_mat = oct_mat.to_value().expect("to_value matrix");
    assert_eq!(d_mat, back_mat);

    // 2. Logical array
    let log_arr =
        Value::Logical(DenseArray::from_vec(vec![1, 3], vec![true, false, true]).unwrap());
    let oct_log = OctaveValue::from_value(&log_arr).expect("from_value logical");
    let back_log = oct_log.to_value().expect("to_value logical");
    assert_eq!(log_arr, back_log);

    // 3. Char vector
    let char_val = Value::char_vector("hello_octave");
    let oct_char = OctaveValue::from_value(&char_val).expect("from_value char");
    let back_char = oct_char.to_value().expect("to_value char");
    assert_eq!(char_val, back_char);

    // 4. Cell array
    let cell_val = Value::Cell(
        DenseArray::from_vec(
            vec![1, 2],
            vec![Value::scalar(100.0), Value::char_vector("test")],
        )
        .unwrap(),
    );
    let oct_cell = OctaveValue::from_value(&cell_val).expect("from_value cell");
    let back_cell = oct_cell.to_value().expect("to_value cell");
    assert_eq!(cell_val, back_cell);

    // 5. Struct array
    let mut st = StructArray::empty();
    st.set_field("alpha", Value::scalar(1.23));
    st.set_field("beta", Value::char_vector("nested"));
    let st_val = Value::Struct(st);
    let oct_st = OctaveValue::from_value(&st_val).expect("from_value struct");
    let back_st = oct_st.to_value().expect("to_value struct");
    assert_eq!(st_val, back_st);
}

#[test]
fn test_octave_literal_generation() {
    let scalar = OctaveValue::scalar(123.45);
    assert_eq!(scalar.to_octave_literal(), "123.45");

    let mat = OctaveValue::Double {
        dims: vec![2, 2],
        data: vec![1.0, 3.0, 2.0, 4.0],
    };
    assert_eq!(mat.to_octave_literal(), "reshape([1, 3, 2, 4], [2, 2])");

    let text = OctaveValue::Char {
        data: "it's working".to_string(),
    };
    assert_eq!(text.to_octave_literal(), "'it''s working'");

    let log = OctaveValue::Logical {
        dims: vec![1, 2],
        data: vec![true, false],
    };
    assert_eq!(log.to_octave_literal(), "logical(reshape([1, 0], [1, 2]))");
}

#[test]
fn test_octave_safety_allowlist_enforcement() {
    // 1. Blocked functions by default
    for &func in BLOCKED_FUNCTIONS {
        let res = check_call_safety(func, false);
        assert!(
            res.is_err(),
            "Blocked function '{func}' must be rejected by default policy"
        );
        let err_msg = res.unwrap_err();
        assert!(err_msg.contains("Security violation"));
        assert!(err_msg.contains(func));
    }

    // 2. Allow unsafe flag permits execution
    for &func in BLOCKED_FUNCTIONS {
        assert!(
            check_call_safety(func, true).is_ok(),
            "allow_unsafe must permit '{func}'"
        );
    }

    // 3. Known-safe builtins recognized
    assert!(is_recognized_safe("sin"));
    assert!(is_recognized_safe("svd"));
    assert!(is_recognized_safe("inv"));
    assert!(!is_recognized_safe("system"));
    assert!(!is_recognized_safe("custom_unknown_123"));
}

#[test]
fn test_octave_mock_session_eval_and_call() {
    let mut session = OctaveSession::start_mock();

    // Ping
    assert!(session.ping().is_ok());

    // Put and Get
    let test_val = Value::scalar(999.0);
    session.put("my_var", &test_val).expect("put variable");
    let retrieved = session.get("my_var").expect("get variable");
    assert_eq!(test_val, retrieved);

    // Eval
    let num_res = session.eval("42").expect("eval number");
    assert_eq!(num_res, Value::scalar(42.0));

    let str_res = session.eval("'octave_test'").expect("eval string");
    assert_eq!(str_res, Value::char_vector("octave_test"));

    // Call SVD on 2x2 matrix
    let mat = Value::Double(DenseArray::from_vec(vec![2, 2], vec![1.0, 3.0, 2.0, 4.0]).unwrap());
    let s_vals = session
        .call("svd", std::slice::from_ref(&mat))
        .expect("call svd");
    if let Value::Double(arr) = s_vals {
        assert_eq!(arr.shape(), &[1, 2]);
        assert!((arr.as_slice()[0] - 5.4649).abs() < 1e-3);
    } else {
        panic!("Expected Double vector result for SVD");
    }

    // Call Det on 2x2 matrix
    let det_val = session.call("det", &[mat]).expect("call det");
    assert_eq!(det_val, Value::scalar(-2.0));

    // Security violation on dangerous call
    let sec_err = session.call("system", &[Value::char_vector("whoami")]);
    assert!(sec_err.is_err());
    match sec_err.unwrap_err() {
        OctaveError::SecurityViolation(msg) => {
            assert!(msg.contains("system"));
        }
        other => panic!("Expected SecurityViolation, got: {other:?}"),
    }
}

#[test]
fn test_octave_protocol_serialization_roundtrip() {
    // Request round-trip
    let req = OctaveRequest::Call {
        fn_name: "eig".to_string(),
        args: vec![OctaveValue::scalar(10.0)],
    };
    let json = serde_json::to_string(&req).expect("serialize request");
    let back: OctaveRequest = serde_json::from_str(&json).expect("deserialize request");
    assert_eq!(req, back);

    // Response round-trip (Ok)
    let resp_ok = OctaveResponse {
        ok: true,
        result: Some(OctaveValue::scalar(20.0)),
        error: None,
    };
    let json_ok = serde_json::to_string(&resp_ok).expect("serialize response ok");
    let back_ok: OctaveResponse = serde_json::from_str(&json_ok).expect("deserialize response ok");
    assert_eq!(resp_ok, back_ok);

    // Response round-trip (Error)
    let resp_err = OctaveResponse {
        ok: false,
        result: None,
        error: Some(OctaveErrorInfo {
            identifier: "Octave:syntaxError".to_string(),
            message: "parse error near line 1".to_string(),
        }),
    };
    let json_err = serde_json::to_string(&resp_err).expect("serialize response err");
    let back_err: OctaveResponse =
        serde_json::from_str(&json_err).expect("deserialize response err");
    assert_eq!(resp_err, back_err);
}

#[test]
fn test_octave_session_lifecycle_and_shutdown() {
    let mut session = OctaveSession::start_mock_with_config(OctaveConfig {
        allow_unsafe: false,
        timeout_ms: 5000,
    });
    session.put("state", &Value::scalar(1.0)).expect("put");
    session.shutdown();
}

#[test]
fn test_real_octave_oracle_conformance_if_installed() {
    if !OctaveSession::is_available() {
        println!("[SKIPPED] GNU Octave (octave-cli) not installed on host runner.");
        return;
    }

    let bin = OctaveSession::find_binary().expect("found binary");
    println!("Running live Octave oracle test using: {}", bin.display());

    let mut session = OctaveSession::start().expect("start real Octave session");
    assert!(
        session.ping().is_ok(),
        "Real Octave worker must respond to ping"
    );

    let res = session.eval("2 + 2").expect("eval 2+2");
    assert_eq!(res, Value::scalar(4.0));
    session.shutdown();
}
