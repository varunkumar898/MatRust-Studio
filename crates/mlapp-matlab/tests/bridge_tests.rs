use mlapp_matlab::{
    find_matlab_binary, is_matlab_available, MatlabErrorInfo, MatlabRequest, MatlabResponse,
    MatlabSession, MatlabValue,
};
use rustlab_math::{DenseArray, StructArray, Value};
use std::collections::HashMap;

#[test]
fn test_matlab_detection_functions() {
    let available = is_matlab_available();
    let bin = find_matlab_binary();
    assert_eq!(available, bin.is_some());
}

#[test]
fn test_protocol_serialization_roundtrip() {
    let req = MatlabRequest::Call {
        fn_name: "det".to_string(),
        args: vec![MatlabValue::Double {
            dims: vec![2, 2],
            data: vec![1.0, 3.0, 2.0, 4.0],
        }],
    };
    let json_str = serde_json::to_string(&req).expect("Failed to serialize request");
    assert!(json_str.contains("\"cmd\":\"call\""));
    assert!(json_str.contains("\"fn_name\":\"det\""));

    let parsed_req: MatlabRequest =
        serde_json::from_str(&json_str).expect("Failed to parse request");
    match parsed_req {
        MatlabRequest::Call { fn_name, args } => {
            assert_eq!(fn_name, "det");
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected Call variant"),
    }
}

#[test]
fn test_protocol_error_response_handling() {
    let resp = MatlabResponse {
        ok: false,
        result: None,
        error: Some(MatlabErrorInfo {
            identifier: "MATLAB:UndefinedFunction".to_string(),
            message: "Undefined function 'foo' for input arguments of type 'double'.".to_string(),
        }),
    };
    let json_str = serde_json::to_string(&resp).expect("Failed to serialize response");
    let parsed: MatlabResponse = serde_json::from_str(&json_str).expect("Failed to parse response");
    assert!(!parsed.ok);
    assert_eq!(parsed.error.unwrap().identifier, "MATLAB:UndefinedFunction");
}

#[test]
fn test_matlab_value_roundtrip_all_types() {
    // 1. Double
    let double_val =
        Value::Double(DenseArray::from_vec(vec![2, 2], vec![1.0, 2.0, 3.0, 4.0]).unwrap());
    let m_double = MatlabValue::from_value(&double_val).unwrap();
    let roundtrip_double = m_double.to_value().unwrap();
    assert_eq!(double_val, roundtrip_double);

    // 2. Char
    let char_val = Value::char_vector("hello matlab");
    let m_char = MatlabValue::from_value(&char_val).unwrap();
    let roundtrip_char = m_char.to_value().unwrap();
    assert_eq!(char_val, roundtrip_char);

    // 3. Logical
    let bool_val =
        Value::Logical(DenseArray::from_vec(vec![1, 3], vec![true, false, true]).unwrap());
    let m_bool = MatlabValue::from_value(&bool_val).unwrap();
    let roundtrip_bool = m_bool.to_value().unwrap();
    assert_eq!(bool_val, roundtrip_bool);

    // 4. Struct
    let mut map = HashMap::new();
    map.insert("num".to_string(), Value::scalar(42.0));
    map.insert("text".to_string(), Value::char_vector("field"));
    let struct_val = Value::Struct(StructArray::scalar(map));
    let m_struct = MatlabValue::from_value(&struct_val).unwrap();
    let roundtrip_struct = m_struct.to_value().unwrap();
    match roundtrip_struct {
        Value::Struct(st) => {
            assert_eq!(st.get_field("num"), Some(&Value::scalar(42.0)));
            assert_eq!(st.get_field("text"), Some(&Value::char_vector("field")));
        }
        _ => panic!("Expected Struct variant"),
    }
}

#[test]
fn test_mock_session_eval_and_call() {
    let mut session = MatlabSession::start_mock();

    // Eval
    let res = session.eval("x = 42").expect("Mock eval failed");
    assert_eq!(res.value, Value::scalar(42.0));

    // Call SVD
    let matrix = Value::Double(DenseArray::from_vec(vec![2, 2], vec![1.0, 3.0, 2.0, 4.0]).unwrap());
    let svd_res = session
        .call("svd", std::slice::from_ref(&matrix))
        .expect("Mock svd call failed");
    match svd_res {
        Value::Double(arr) => {
            assert_eq!(arr.shape(), &[1, 2]);
            let s = arr.as_slice();
            assert!((s[0] - 5.4649857).abs() < 1e-4);
            assert!((s[1] - 0.3659661).abs() < 1e-4);
        }
        _ => panic!("Expected Double vector from svd"),
    }

    // Call det
    let det_res = session
        .call("det", &[matrix])
        .expect("Mock det call failed");
    assert_eq!(det_res, Value::scalar(-2.0));
}
