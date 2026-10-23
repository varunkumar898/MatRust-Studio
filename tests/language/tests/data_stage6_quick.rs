use matlab_runtime::run_source;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_csv(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "matrust_{}_{}_{}.csv",
        label,
        std::process::id(),
        nonce
    ))
}

#[test]
fn stage6_csv_roundtrip() {
    let path = temp_csv("csv_roundtrip");
    let path_text = path.to_string_lossy().to_string();

    let r = run_source(&format!("A = [1 2; 3 4]; csvwrite('{}', A);", path_text))
        .expect("csvwrite executes");
    let a = r.env.get("A").and_then(|v| v.as_matrix()).unwrap();
    assert_eq!(a.data, vec![1.0, 2.0, 3.0, 4.0]);

    let written = std::fs::read_to_string(&path).expect("csvwrite created a file");
    assert_eq!(written, "1,2\n3,4\n");

    let r2 = run_source(&format!("A2 = csvread('{}');", path_text)).expect("csvread executes");
    let a2 = r2
        .env
        .get("A2")
        .and_then(|v| v.as_matrix())
        .expect("A2 is a matrix");
    assert_eq!((a2.rows, a2.cols), (2, 2));
    assert_eq!(a2.data, vec![1.0, 2.0, 3.0, 4.0]);

    std::fs::remove_file(&path).expect("temporary CSV cleaned up");
}

#[test]
fn stage6_save_creates_named_file() {
    let path = temp_csv("save");
    let path_text = path.to_string_lossy().to_string();

    run_source(&format!("x = [10 20]; save('{}');", path_text)).expect("save executes");
    let written = std::fs::read_to_string(&path).expect("save created a file");
    assert_eq!(written, "x\n10,20\n");

    let loaded = run_source(&format!("data = load('{}');", path_text)).expect("load executes");
    let data = loaded
        .env
        .get("data")
        .and_then(|v| v.as_matrix())
        .expect("loaded data is a matrix");
    assert_eq!((data.rows, data.cols), (1, 2));
    assert_eq!(data.data, vec![10.0, 20.0]);

    std::fs::remove_file(&path).expect("temporary CSV cleaned up");
}
