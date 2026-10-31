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

        let source = fs::read_to_string(&m_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", m_path.display()));

        let mut interp = Interpreter::new();
        rustlab_stdlib::register_all(&mut interp);

        let res = interp.eval_script(&source);
        assert!(
            res.is_ok(),
            "Conformance script {} failed to evaluate: {:?}",
            m_path.display(),
            res.err()
        );

        let expected_path = m_path.with_extension("expected");
        if expected_path.exists() {
            let expected_text = fs::read_to_string(&expected_path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {e}", expected_path.display()));
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
                        assert!(
                            interp.env.get(var_name).is_some() || interp.last_ans.is_some(),
                            "Expected variable '{}' in environment after running {}",
                            var_name,
                            m_path.display()
                        );
                    }
                }
            }
        }
    }
}
