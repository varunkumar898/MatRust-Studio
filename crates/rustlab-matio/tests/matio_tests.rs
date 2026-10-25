use rustlab_math::{Complex64, DenseArray, Value};
use rustlab_matio::{load_mat, read_mat, save_mat, write_mat};
use std::collections::HashMap;

#[test]
fn test_uncompressed_roundtrip() {
    let mut vars = HashMap::new();
    let da_a = DenseArray::from_vec(vec![2, 2], vec![1.0, 3.0, 2.0, 4.0]).unwrap();
    vars.insert("A".to_string(), Value::Double(da_a));

    let da_b = DenseArray::col_vector(vec![5.0, 6.0]);
    vars.insert("b".to_string(), Value::Double(da_b));

    // Serialize uncompressed
    let bytes = write_mat(&vars, false).unwrap();
    assert!(bytes.len() >= 128);

    // Deserialize
    let loaded = read_mat(&bytes).unwrap();
    assert_eq!(loaded.len(), 2);

    let loaded_a = loaded.get("A").unwrap();
    assert_eq!(loaded_a.shape(), &[2, 2]);
    let da_loaded_a = loaded_a.to_double_array().unwrap();
    assert_eq!(da_loaded_a.as_slice(), &[1.0, 3.0, 2.0, 4.0]);

    let loaded_b = loaded.get("b").unwrap();
    assert_eq!(loaded_b.shape(), &[2, 1]);
    let da_loaded_b = loaded_b.to_double_array().unwrap();
    assert_eq!(da_loaded_b.as_slice(), &[5.0, 6.0]);
}

#[test]
fn test_compressed_roundtrip() {
    let mut vars = HashMap::new();
    let da = DenseArray::from_vec(vec![3, 3], (1..=9).map(|x| x as f64).collect()).unwrap();
    vars.insert("matrix".to_string(), Value::Double(da));

    // Serialize with zlib compression (MATLAB v7 default)
    let bytes = write_mat(&vars, true).unwrap();
    assert!(bytes.len() >= 128);

    // Read back
    let loaded = read_mat(&bytes).unwrap();
    assert_eq!(loaded.len(), 1);

    let loaded_mat = loaded.get("matrix").unwrap();
    assert_eq!(loaded_mat.shape(), &[3, 3]);
    let da_loaded = loaded_mat.to_double_array().unwrap();
    assert_eq!(
        da_loaded.as_slice(),
        &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]
    );
}

#[test]
fn test_complex_numbers_roundtrip() {
    let mut vars = HashMap::new();
    let c1 = Complex64::new(1.5, 2.5);
    let c2 = Complex64::new(-3.0, 4.0);
    let da_c = DenseArray::row_vector(vec![c1, c2]);
    vars.insert("Z".to_string(), Value::ComplexDouble(da_c));

    let bytes = write_mat(&vars, true).unwrap();
    let loaded = read_mat(&bytes).unwrap();

    let val = loaded.get("Z").unwrap();
    if let Value::ComplexDouble(da) = val {
        assert_eq!(da.shape(), &[1, 2]);
        assert_eq!(da.as_slice()[0], c1);
        assert_eq!(da.as_slice()[1], c2);
    } else {
        panic!("Expected ComplexDouble value");
    }
}

#[test]
fn test_char_and_logical_roundtrip() {
    let mut vars = HashMap::new();
    vars.insert("str".to_string(), Value::char_vector("RustLab v5"));

    let bools = vec![true, false, true, false];
    let da_bool = DenseArray::from_vec(vec![2, 2], bools).unwrap();
    vars.insert("mask".to_string(), Value::Logical(da_bool));

    let bytes = write_mat(&vars, false).unwrap();
    let loaded = read_mat(&bytes).unwrap();

    let str_val = loaded.get("str").unwrap();
    assert_eq!(str_val, &Value::char_vector("RustLab v5"));

    let mask_val = loaded.get("mask").unwrap();
    if let Value::Logical(da) = mask_val {
        assert_eq!(da.shape(), &[2, 2]);
        assert_eq!(da.as_slice(), &[true, false, true, false]);
    } else {
        panic!("Expected Logical value");
    }
}

#[test]
fn test_file_save_and_load() {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("test_rustlab_save.mat");

    let mut vars = HashMap::new();
    let da = DenseArray::scalar(std::f64::consts::PI);
    vars.insert("pi_val".to_string(), Value::Double(da));

    save_mat(&file_path, &vars, true).unwrap();
    assert!(file_path.exists());

    let loaded = load_mat(&file_path).unwrap();
    let val = loaded.get("pi_val").unwrap();
    assert!((val.as_scalar_f64().unwrap() - std::f64::consts::PI).abs() < 1e-12);

    let _ = std::fs::remove_file(file_path);
}

#[test]
fn test_numeric_classes_roundtrip() {
    let mut vars = HashMap::new();
    vars.insert(
        "s32".to_string(),
        Value::Single(DenseArray::row_vector(vec![1.5f32, 2.5f32])),
    );
    vars.insert(
        "i8".to_string(),
        Value::Int8(DenseArray::col_vector(vec![-128i8, 0, 127])),
    );
    vars.insert(
        "u8".to_string(),
        Value::UInt8(DenseArray::row_vector(vec![0u8, 128, 255])),
    );
    vars.insert(
        "i16".to_string(),
        Value::Int16(DenseArray::row_vector(vec![-32768i16, 32767])),
    );
    vars.insert(
        "u16".to_string(),
        Value::UInt16(DenseArray::row_vector(vec![0u16, 65535])),
    );
    vars.insert(
        "i32".to_string(),
        Value::Int32(DenseArray::row_vector(vec![-2147483648i32, 2147483647])),
    );
    vars.insert(
        "u32".to_string(),
        Value::UInt32(DenseArray::row_vector(vec![0u32, 4294967295])),
    );
    vars.insert(
        "i64".to_string(),
        Value::Int64(DenseArray::row_vector(vec![
            -9223372036854775808i64,
            9223372036854775807,
        ])),
    );
    vars.insert(
        "u64".to_string(),
        Value::UInt64(DenseArray::row_vector(vec![0u64, 18446744073709551615])),
    );

    let bytes = write_mat(&vars, true).unwrap();
    let loaded = read_mat(&bytes).unwrap();

    assert_eq!(loaded.get("s32").unwrap(), vars.get("s32").unwrap());
    assert_eq!(loaded.get("i8").unwrap(), vars.get("i8").unwrap());
    assert_eq!(loaded.get("u8").unwrap(), vars.get("u8").unwrap());
    assert_eq!(loaded.get("i16").unwrap(), vars.get("i16").unwrap());
    assert_eq!(loaded.get("u16").unwrap(), vars.get("u16").unwrap());
    assert_eq!(loaded.get("i32").unwrap(), vars.get("i32").unwrap());
    assert_eq!(loaded.get("u32").unwrap(), vars.get("u32").unwrap());
    assert_eq!(loaded.get("i64").unwrap(), vars.get("i64").unwrap());
    assert_eq!(loaded.get("u64").unwrap(), vars.get("u64").unwrap());
}

#[test]
fn test_matio_image_sparse_and_empty_roundtrip() {
    use rustlab_math::CscMatrix;
    let mut vars = HashMap::new();

    // 1. Image-sized 3D array: 256x256x3
    let img_numel = 256 * 256 * 3;
    let img_data: Vec<u8> = (0..img_numel).map(|i| (i % 256) as u8).collect();
    let da_img = DenseArray::from_vec(vec![256, 256, 3], img_data).unwrap();
    vars.insert("img".to_string(), Value::UInt8(da_img));

    // 2. Sparse CSC matrix
    let sp =
        CscMatrix::from_triplets_1based(5, 5, &[1, 3, 5], &[2, 3, 4], &[10.0, 20.0, 30.0]).unwrap();
    vars.insert("S".to_string(), Value::Sparse(sp));

    // 3. Empty arrays: 0x0, 0x5, 3x0
    vars.insert(
        "empty_0x0".to_string(),
        Value::Double(DenseArray::from_vec(vec![0, 0], Vec::new()).unwrap()),
    );
    vars.insert(
        "empty_0x5".to_string(),
        Value::Double(DenseArray::from_vec(vec![0, 5], Vec::new()).unwrap()),
    );
    vars.insert(
        "empty_3x0".to_string(),
        Value::Double(DenseArray::from_vec(vec![3, 0], Vec::new()).unwrap()),
    );

    let bytes = write_mat(&vars, true).unwrap();
    let loaded = read_mat(&bytes).unwrap();

    let loaded_img = loaded.get("img").unwrap();
    assert_eq!(loaded_img.shape(), &[256, 256, 3]);
    assert_eq!(loaded_img, vars.get("img").unwrap());

    let loaded_s = loaded.get("S").unwrap();
    assert_eq!(loaded_s, vars.get("S").unwrap());

    let loaded_0x0 = loaded.get("empty_0x0").unwrap();
    assert_eq!(loaded_0x0.shape(), &[0, 0]);

    let loaded_0x5 = loaded.get("empty_0x5").unwrap();
    assert_eq!(loaded_0x5.shape(), &[0, 5]);

    let loaded_3x0 = loaded.get("empty_3x0").unwrap();
    assert_eq!(loaded_3x0.shape(), &[3, 0]);
}
