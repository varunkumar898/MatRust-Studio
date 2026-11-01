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
