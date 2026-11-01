use rustlab_eval::Interpreter;
use rustlab_math::Value;
use std::fs;
use std::path::PathBuf;

fn find_conformance_dir() -> PathBuf {
    let candidates = [
        PathBuf::from("tests/conformance"),
        PathBuf::from("../../tests/conformance"),
        PathBuf::from("../tests/conformance"),
    ];
    for c in &candidates {
        if c.exists() && c.is_dir() {
            return c.clone();
        }
    }
    PathBuf::from("tests/conformance")
}

#[test]
fn test_conformance_scripts_execution() {
    let conf_dir = find_conformance_dir();
    assert!(
        conf_dir.exists(),
        "Conformance directory {:?} must exist",
        conf_dir
    );

    let mut entries: Vec<_> = fs::read_dir(&conf_dir)
        .expect("read conformance dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "m"))
        .collect();
    entries.sort();

    assert!(
        !entries.is_empty(),
        "Must find at least one conformance test script in {:?}",
        conf_dir
    );

    for m_path in entries {
        let file_stem = m_path.file_stem().unwrap().to_string_lossy().to_string();
        println!("Executing conformance test: {}", m_path.display());

        let source_raw = fs::read_to_string(&m_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", m_path.display()));
        let source = source_raw.trim_start_matches('\u{feff}');

        let mut interp = Interpreter::new();
        rustlab_stdlib::register_all(&mut interp);

        let res = interp.eval_script(source);
        assert!(
            res.is_ok(),
            "Conformance script {} failed to evaluate: {:?}",
            m_path.display(),
            res.err()
        );

        let expected_path = m_path.with_extension("expected");
        if expected_path.exists() {
            let expected_text_raw = fs::read_to_string(&expected_path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {e}", expected_path.display()));
            let expected_text = expected_text_raw.trim_start_matches('\u{feff}');
            let first_line = expected_text.lines().next().unwrap_or("").trim();
            let var_name = first_line.split('=').next().unwrap_or("").trim();

            match file_stem.as_str() {
                "cellfun_anonymous" => {
                    let val = interp
                        .env
                        .get("sq")
                        .or_else(|| interp.last_ans.clone())
                        .expect("variable sq or ans");
                    let double_arr = val.to_double_array().expect("double array");
                    assert_eq!(double_arr.as_slice(), &[1.0, 4.0, 9.0]);
                }
                "indexing_deletion" => {
                    let val = interp.env.get("res").expect("variable res");
                    let double_arr = val.to_double_array().expect("double array");
                    assert_eq!(double_arr.as_slice(), &[1.0, 3.0, 4.0]);
                }
                "comprehensive_callback" => {
                    let val = interp.env.get("out").expect("variable out");
                    match val {
                        Value::Char(da) => {
                            let s: String = da.as_slice().iter().collect();
                            assert_eq!(s, "Result=81");
                        }
                        Value::String(da) => {
                            assert_eq!(
                                da.as_slice().first().map(|s| s.as_str()),
                                Some("Result=81")
                            );
                        }
                        other => panic!("expected string output, got {other:?}"),
                    }
                }
                _ => {
                    if !var_name.is_empty() {
                        let actual_val = interp
                            .env
                            .get(var_name)
                            .or_else(|| interp.last_ans.clone())
                            .unwrap_or_else(|| {
                                panic!(
                                    "Expected variable '{}' in environment after running {}",
                                    var_name,
                                    m_path.display()
                                )
                            });

                        let expected_val_lines: Vec<&str> = expected_text
                            .lines()
                            .skip(1)
                            .map(|l| l.trim())
                            .filter(|l| !l.is_empty())
                            .collect();

                        if !expected_val_lines.is_empty() {
                            let val_str = expected_val_lines.join(" ");

                            match actual_val {
                                Value::Char(da) => {
                                    let s: String = da.as_slice().iter().collect();
                                    assert_eq!(
                                        s.trim(),
                                        val_str.trim(),
                                        "Char string mismatch in {}",
                                        m_path.display()
                                    );
                                }
                                Value::String(da) => {
                                    if let Some(first) = da.as_slice().first() {
                                        assert_eq!(
                                            first.trim(),
                                            val_str.trim(),
                                            "String mismatch in {}",
                                            m_path.display()
                                        );
                                    }
                                }
                                _ => {
                                    let parsed_numbers: Result<Vec<f64>, _> = val_str
                                        .split_whitespace()
                                        .map(|token| token.parse::<f64>())
                                        .collect();

                                    if let Ok(expected_floats) = parsed_numbers {
                                        if let Ok(arr) = actual_val.to_double_array() {
                                            assert_eq!(
                                                arr.numel(),
                                                expected_floats.len(),
                                                "Element count mismatch for '{}' in {}: expected {}, got {}",
                                                var_name,
                                                m_path.display(),
                                                expected_floats.len(),
                                                arr.numel()
                                            );
                                            for (i, (&actual, &expected)) in arr
                                                .as_slice()
                                                .iter()
                                                .zip(expected_floats.iter())
                                                .enumerate()
                                            {
                                                if expected.is_nan() {
                                                    assert!(
                                                        actual.is_nan(),
                                                        "Expected NaN at index {} in {}",
                                                        i,
                                                        m_path.display()
                                                    );
                                                } else if expected.is_infinite() {
                                                    assert_eq!(
                                                        actual.is_sign_positive(),
                                                        expected.is_sign_positive(),
                                                        "Sign mismatch at index {} in {}",
                                                        i,
                                                        m_path.display()
                                                    );
                                                    assert!(
                                                        actual.is_infinite(),
                                                        "Expected Inf at index {} in {}",
                                                        i,
                                                        m_path.display()
                                                    );
                                                } else {
                                                    let diff = (actual - expected).abs();
                                                    assert!(
                                                        diff <= 1e-9
                                                            || (expected != 0.0
                                                                && diff / expected.abs() <= 1e-9),
                                                        "Value mismatch at index {} in {}: expected {}, got {} (diff: {})",
                                                        i,
                                                        m_path.display(),
                                                        expected,
                                                        actual,
                                                        diff
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
