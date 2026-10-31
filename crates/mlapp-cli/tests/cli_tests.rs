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
    let fixture = get_fixture_path("r2021a_sample.mlapp");
    let mut cmd = Command::cargo_bin("mlapp-cli").expect("binary mlapp-cli exists");

    cmd.arg("run")
        .arg("--gui")
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "[GUI Mode]: Initializing GUI window",
        ))
        .stdout(predicate::str::contains("App Name: FixtureApp_R2021a"))
        .stdout(predicate::str::contains("App session closed successfully."));
}

#[test]
fn test_cli_invalid_path_fails() {
    let mut cmd = Command::cargo_bin("mlapp-cli").expect("binary mlapp-cli exists");

    cmd.arg("inspect")
        .arg("non_existent_file_path.mlapp")
        .assert()
        .failure();
}
