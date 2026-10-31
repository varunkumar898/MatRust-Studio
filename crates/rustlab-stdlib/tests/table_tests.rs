#![cfg(feature = "data")]

//! Integration tests for table operations: readtable, writetable, head, tail, summary.

use rustlab_math::Value;
use std::fs::File;
use std::io::Write;

#[test]
fn test_readtable_and_dot_indexing() {
    let dir = tempfile::tempdir().unwrap();
    let csv_path = dir.path().join("sensor_readings.csv");
    {
        let mut f = File::create(&csv_path).unwrap();
        writeln!(f, "time,temperature,pressure").unwrap();
        writeln!(f, "0.0,21.5,101.3").unwrap();
        writeln!(f, "1.0,22.0,101.2").unwrap();
        writeln!(f, "2.0,23.5,101.5").unwrap();
    }

    let script = format!(
        "data = readtable('{}'); temp = data.temperature; m = mean(temp);",
        csv_path.display().to_string().replace('\\', "/")
    );
    let mut interp = rustlab_stdlib::new_interpreter();
    let _ = interp.eval_script(&script).unwrap();

    let temp_val = interp.env.get("temp").unwrap();
    match temp_val {
        Value::Double(d) => {
            assert_eq!(d.shape(), &[3, 1]);
            assert_eq!(d.as_slice(), &[21.5, 22.0, 23.5]);
        }
        _ => panic!("Expected double column vector"),
    }

    let m_val = interp.env.get("m").unwrap();
    let mean_f64 = m_val.as_scalar_f64().unwrap();
    // (21.5 + 22.0 + 23.5) / 3 = 67.0 / 3 = 22.3333333...
    assert!((mean_f64 - 22.333333333333332).abs() < 1e-12);
}

#[test]
fn test_head_and_tail_table() {
    let dir = tempfile::tempdir().unwrap();
    let csv_path = dir.path().join("data_rows.csv");
    {
        let mut f = File::create(&csv_path).unwrap();
        writeln!(f, "x,y").unwrap();
        for i in 1..=10 {
            writeln!(f, "{i},{}", i * 10).unwrap();
        }
    }

    let script = format!(
        "T = readtable('{}'); H = head(T, 3); Tl = tail(T, 2); h_y = H.y; tl_y = Tl.y;",
        csv_path.display().to_string().replace('\\', "/")
    );
    let mut interp = rustlab_stdlib::new_interpreter();
    interp.eval_script(&script).unwrap();

    let h_y = interp.env.get("h_y").unwrap();
    match h_y {
        Value::Double(d) => {
            assert_eq!(d.shape(), &[3, 1]);
            assert_eq!(d.as_slice(), &[10.0, 20.0, 30.0]);
        }
        _ => panic!("Expected double array"),
    }

    let tl_y = interp.env.get("tl_y").unwrap();
    match tl_y {
        Value::Double(d) => {
            assert_eq!(d.shape(), &[2, 1]);
            assert_eq!(d.as_slice(), &[90.0, 100.0]);
        }
        _ => panic!("Expected double array"),
    }
}

#[test]
fn test_summary_and_writetable_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let csv_path = dir.path().join("source.csv");
    let out_path = dir.path().join("exported.csv");
    {
        let mut f = File::create(&csv_path).unwrap();
        writeln!(f, "speed,voltage").unwrap();
        writeln!(f, "10.0,12.5").unwrap();
        writeln!(f, "20.0,14.5").unwrap();
    }

    let script = format!(
        "T = readtable('{}'); s = summary(T); writetable(T, '{}'); T2 = readtable('{}'); s2 = T2.speed;",
        csv_path.display().to_string().replace('\\', "/"),
        out_path.display().to_string().replace('\\', "/"),
        out_path.display().to_string().replace('\\', "/")
    );
    let mut interp = rustlab_stdlib::new_interpreter();
    interp.eval_script(&script).unwrap();

    let s2 = interp.env.get("s2").unwrap();
    match s2 {
        Value::Double(d) => {
            assert_eq!(d.shape(), &[2, 1]);
            assert_eq!(d.as_slice(), &[10.0, 20.0]);
        }
        _ => panic!("Expected double vector"),
    }

    // Missing values and quoted fields:
    let missing_path = dir.path().join("missing_quoted.csv");
    {
        let mut f = File::create(&missing_path).unwrap();
        writeln!(f, "\"patient_id\",\"heart_rate\",\"status\"").unwrap();
        writeln!(f, "\"P101\",72.0,\"Normal\"").unwrap();
        writeln!(f, "\"P102\",,\"Abnormal\"").unwrap(); // missing heart rate -> NaN
        writeln!(f, "\"P103\",NaN,\"Normal\"").unwrap(); // explicit NaN
    }

    let script_missing = format!(
        "T_m = readtable('{}'); hr = T_m.heart_rate; p_ids = T_m.patient_id;",
        missing_path.display().to_string().replace('\\', "/")
    );
    let mut interp2 = rustlab_stdlib::new_interpreter();
    interp2.eval_script(&script_missing).unwrap();

    let hr_val = interp2.env.get("hr").unwrap();
    match hr_val {
        Value::Double(d) => {
            assert_eq!(d.shape(), &[3, 1]);
            let s = d.as_slice();
            assert_eq!(s[0], 72.0);
            assert!(s[1].is_nan(), "Empty field must parse to NaN");
            assert!(s[2].is_nan(), "Explicit 'NaN' field must parse to NaN");
        }
        _ => panic!("Expected double column vector"),
    }

    let pid_val = interp2.env.get("p_ids").unwrap();
    match pid_val {
        Value::String(sa) => {
            assert_eq!(sa.shape(), &[3, 1]);
            assert_eq!(sa.as_slice(), &["P101", "P102", "P103"]);
        }
        _ => panic!("Expected string array for patient IDs"),
    }
}
