use assert_cmd::Command;
use predicates::prelude::*;

fn get_fixture_path(name: &str) -> String {
    let candidate_paths = [
        format!("fixtures/sample_apps/{name}"),
        format!("../../fixtures/sample_apps/{name}"),
        format!("crates/mlapp-container/fixtures/sample_apps/{name}"),
    ];
    for p in candidate_paths {
        if std::path::Path::new(&p).exists() {
            return p;
        }
    }
    format!("fixtures/sample_apps/{name}")
}

fn get_model_fixture_path(name: &str) -> String {
    let candidate_paths = [
        format!("fixtures/sample_models/{name}"),
        format!("../../fixtures/sample_models/{name}"),
    ];
    for p in candidate_paths {
        if std::path::Path::new(&p).exists() {
            return p;
        }
    }
    format!("fixtures/sample_models/{name}")
}

#[test]
fn test_cli_inspect_command() {
    let fixture = get_fixture_path("r2021a_sample.mlapp");
    let mut cmd = Command::cargo_bin("mlapp-cli").expect("binary mlapp-cli exists");

    cmd.arg("inspect")
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains("FixtureApp_R2021a"))
        .stdout(predicate::str::contains("R2021a"))
        .stdout(predicate::str::contains("appdesigner/app.xml"));
}

#[test]
fn test_cli_check_compat_command() {
    let fixture = get_fixture_path("r2021a_sample.mlapp");
    let mut cmd = Command::cargo_bin("mlapp-cli").expect("binary mlapp-cli exists");

    cmd.arg("check-compat")
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"compatibility_score\": 100"))
        .stdout(predicate::str::contains("\"status\": \"Runs\""));
}

#[test]
fn test_cli_check_compat_hazardous_and_unanalyzable() {
    let fixture = get_fixture_path("hazardous_sample.mlapp");
    let mut cmd = Command::cargo_bin("mlapp-cli").expect("binary mlapp-cli exists");

    cmd.arg("check-compat")
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"Unsupported\""))
        .stdout(predicate::str::contains("system"))
        .stdout(predicate::str::contains("eval"))
        .stdout(predicate::str::contains("unanalyzable dynamic call"));
}

#[test]
fn test_cli_extract_command() {
    let fixture = get_fixture_path("r2021a_sample.mlapp");
    let temp_dir = std::env::temp_dir().join("mlapp_cli_extract_test");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let mut cmd = Command::cargo_bin("mlapp-cli").expect("binary mlapp-cli exists");
    cmd.arg("extract")
        .arg(&fixture)
        .arg("--out-dir")
        .arg(&temp_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Extraction complete"));

    assert!(temp_dir.join("appdesigner/app.xml").exists());
    assert!(temp_dir.join("matlab/document.xml").exists());
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_cli_run_command() {
    let fixture = get_fixture_path("r2021a_sample.mlapp");
    let mut cmd = Command::cargo_bin("mlapp-cli").expect("binary mlapp-cli exists");

    cmd.arg("run")
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains("Launching app package"))
        .stdout(predicate::str::contains("App Name: FixtureApp_R2021a"))
        .stdout(predicate::str::contains("App session closed successfully."));
}

#[test]
fn test_cli_run_gui_flag() {
    // The flag must exist and be discoverable. Opening a real window is a
    // manual/desktop check (docs/GUI_MANUAL_CHECK.md); CI is headless.
    let mut cmd = Command::cargo_bin("mlapp-cli").expect("binary mlapp-cli exists");

    cmd.arg("run")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--gui"))
        .stdout(predicate::str::contains("Launch GUI window"));
}

#[test]
fn test_cli_invalid_path_fails() {
    let mut cmd = Command::cargo_bin("mlapp-cli").expect("binary mlapp-cli exists");

    cmd.arg("inspect")
        .arg("non_existent_file_path.mlapp")
        .assert()
        .failure();
}

#[test]
fn test_cli_run_autotest_flag() {
    let mut cmd = Command::cargo_bin("mlapp-cli").expect("binary mlapp-cli exists");

    cmd.arg("run")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--autotest"))
        .stdout(predicate::str::contains("Path to JSON autotest script"));
}

#[test]
fn test_cli_run_autotest_headless_execution() {
    let fixture = get_fixture_path("r2021a_sample.mlapp");

    let script_json = r#"[
        { "type": "wait_ms", "duration_ms": 20 },
        { "type": "set_value", "component": "ThresholdField", "value": "99" },
        { "type": "expect_property", "component": "ThresholdField", "property": "Value", "expected": "99" },
        { "type": "close" }
    ]"#;

    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join("cli_test_autotest.json");
    std::fs::write(&script_path, script_json).expect("write temp autotest script");

    let mut cmd = Command::cargo_bin("mlapp-cli").expect("binary mlapp-cli exists");

    cmd.arg("run")
        .arg(&fixture)
        .arg("--autotest")
        .arg(&script_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Headless Autotest Mode"))
        .stdout(predicate::str::contains(
            "Headless autotest script passed successfully.",
        ));

    let _ = std::fs::remove_file(script_path);
}

#[test]
fn test_cli_run_allow_matlab_flag() {
    let mut cmd = Command::cargo_bin("mlapp-cli").expect("binary mlapp-cli exists");

    cmd.arg("run")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--allow-matlab"))
        .stdout(predicate::str::contains(
            "Allow delegation of unsupported functions to a real MATLAB installation",
        ));
}

#[test]
fn test_cli_slx_inspect_command() {
    let model_path = get_model_fixture_path("simple_feedback.slx");
    let mut cmd = Command::cargo_bin("mlapp-cli").expect("binary mlapp-cli exists");

    cmd.arg("slx-inspect")
        .arg(&model_path)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "=== Simulink Model Inspection Report ===",
        ))
        .stdout(predicate::str::contains("Model Name: simple_feedback"))
        .stdout(predicate::str::contains("Fully Supported by Solver: true"))
        .stdout(predicate::str::contains("Total Blocks: 5"));
}

#[test]
fn test_cli_slx_inspect_json_command() {
    let model_path = get_model_fixture_path("simple_feedback.slx");
    let mut cmd = Command::cargo_bin("mlapp-cli").expect("binary mlapp-cli exists");

    cmd.arg("slx-inspect")
        .arg(&model_path)
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"model_name\": \"simple_feedback\"",
        ))
        .stdout(predicate::str::contains("\"total_blocks\": 5"))
        .stdout(predicate::str::contains("\"is_solver_supported\": true"));
}

#[test]
fn test_cli_check_command_clean_and_json() {
    // Create a temporary clean block diagram JSON
    let temp_dir = std::env::temp_dir();
    let clean_path = temp_dir.join("clean_model_m9.json");
    let clean_json = r#"{
        "name": "clean_model",
        "schema_version": 3,
        "blocks": [
            { "sid": "1", "name": "Constant", "block_type": "Constant", "parameters": { "Value": "10" } }
        ],
        "lines": []
    }"#;
    std::fs::write(&clean_path, clean_json).expect("write clean_path");

    let mut cmd = Command::cargo_bin("mlapp-cli").expect("binary mlapp-cli exists");
    cmd.arg("check")
        .arg(&clean_path)
        .assert()
        .code(0)
        .stdout(predicate::str::contains("clean"));

    // Test with --json
    let mut cmd_json = Command::cargo_bin("mlapp-cli").expect("binary mlapp-cli exists");
    cmd_json
        .arg("check")
        .arg(&clean_path)
        .arg("--json")
        .assert()
        .code(0)
        .stdout(predicate::str::contains("[]"));

    let _ = std::fs::remove_file(clean_path);
}

#[test]
fn test_cli_check_command_error_and_warning() {
    let temp_dir = std::env::temp_dir();

    // Diagram with unconnected port in schema 4 -> Error (exit code 2)
    let err_path = temp_dir.join("error_model_m9.json");
    let err_json = r#"{
        "name": "error_model",
        "schema_version": 4,
        "blocks": [
            { "sid": "1", "name": "Gain", "block_type": "Gain", "parameters": { "Gain": "2.0" } }
        ],
        "lines": []
    }"#;
    std::fs::write(&err_path, err_json).expect("write err_path");

    let mut cmd_err = Command::cargo_bin("mlapp-cli").expect("binary mlapp-cli exists");
    cmd_err
        .arg("check")
        .arg(&err_path)
        .assert()
        .code(2)
        .stdout(predicate::str::contains("E-PORT"))
        .stdout(predicate::str::contains("Error"));

    let _ = std::fs::remove_file(err_path);

    // Diagram with unconnected port in schema 3 -> Warning (exit code 1)
    let warn_path = temp_dir.join("warn_model_m9.json");
    let warn_json = r#"{
        "name": "warn_model",
        "schema_version": 3,
        "blocks": [
            { "sid": "1", "name": "Gain", "block_type": "Gain", "parameters": { "Gain": "2.0" } }
        ],
        "lines": []
    }"#;
    std::fs::write(&warn_path, warn_json).expect("write warn_path");

    let mut cmd_warn = Command::cargo_bin("mlapp-cli").expect("binary mlapp-cli exists");
    cmd_warn
        .arg("check")
        .arg(&warn_path)
        .assert()
        .code(1)
        .stdout(predicate::str::contains("W-PORT"))
        .stdout(predicate::str::contains("Warning"));

    let _ = std::fs::remove_file(warn_path);
}

#[test]
fn test_cli_codegen_command() {
    let temp_dir = std::env::temp_dir().join("mlapp_cli_codegen_test");
    let _ = std::fs::create_dir_all(&temp_dir);
    let model_path = temp_dir.join("clean_model.json");
    let out_dir = temp_dir.join("out");

    let model_json = r#"{
        "name": "gain_sys",
        "schema_version": 4,
        "blocks": [
            { "sid": "1", "name": "Constant", "block_type": "Constant", "parameters": { "Value": "3.14" } },
            { "sid": "2", "name": "Gain", "block_type": "Gain", "parameters": { "Gain": "2.0" } }
        ],
        "lines": [
            { "src": { "block_sid": "1", "port_index": 0 }, "dst": { "block_sid": "2", "port_index": 0 } }
        ]
    }"#;
    std::fs::write(&model_path, model_json).expect("write model_path");

    let mut cmd = Command::cargo_bin("mlapp-cli").expect("binary mlapp-cli exists");
    cmd.arg("codegen")
        .arg(&model_path)
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--name")
        .arg("gain_sys")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Successfully generated C99 code for model 'gain_sys'",
        ));

    assert!(out_dir.join("gain_sys.h").exists());
    assert!(out_dir.join("gain_sys.c").exists());
    assert!(out_dir.join("gain_sys_main.c").exists());

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_cli_sweep_command() {
    let temp_dir = std::env::temp_dir().join("mlapp_cli_sweep_test");
    let _ = std::fs::create_dir_all(&temp_dir);
    let model_path = temp_dir.join("sweep_model.json");

    let model_json = r#"{
        "name": "sweep_model",
        "schema_version": 4,
        "blocks": [
            { "sid": "1", "name": "Constant", "block_type": "Constant", "parameters": { "Value": "1.0" } },
            { "sid": "2", "name": "Gain", "block_type": "Gain", "parameters": { "Gain": "1.0" } },
            { "sid": "3", "name": "Scope", "block_type": "Scope" }
        ],
        "lines": [
            { "src": { "block_sid": "1", "port_index": 0 }, "dst": { "block_sid": "2", "port_index": 0 } },
            { "src": { "block_sid": "2", "port_index": 0 }, "dst": { "block_sid": "3", "port_index": 0 } }
        ]
    }"#;
    std::fs::write(&model_path, model_json).expect("write model_path");

    let mut cmd = Command::cargo_bin("mlapp-cli").expect("binary mlapp-cli exists");
    cmd.arg("sweep")
        .arg(&model_path)
        .arg("--block")
        .arg("2")
        .arg("--param")
        .arg("Gain")
        .arg("--values")
        .arg("1.0,2.0,4.0")
        .assert()
        .success()
        .stdout(predicate::str::contains("Parameter Sweep Results (3 runs)"))
        .stdout(predicate::str::contains("Scope="));

    let _ = std::fs::remove_dir_all(&temp_dir);
}
