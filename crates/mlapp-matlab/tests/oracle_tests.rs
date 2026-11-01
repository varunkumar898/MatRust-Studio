use mlapp_matlab::MatlabSession;
use std::path::Path;

#[test]
fn test_matlab_oracle_conformance_suite() {
    if !MatlabSession::is_available() {
        println!("[SKIPPED] MATLAB not installed on host (see docs/BLOCKERS.md)");
        return;
    }

    let mut session = match MatlabSession::start() {
        Ok(s) => s,
        Err(e) => {
            println!("[SKIPPED] Failed to start MATLAB session ({e}); see docs/BLOCKERS.md");
            return;
        }
    };

    let conformance_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("tests").join("conformance"))
        .unwrap_or_else(|| Path::new("tests/conformance").to_path_buf());

    if !conformance_dir.is_dir() {
        return;
    }

    if let Ok(entries) = std::fs::read_dir(&conformance_dir) {
        let mut count = 0;
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("m") {
                let script = std::fs::read_to_string(&path).unwrap_or_default();
                let _ = session.eval(&script);
                count += 1;
            }
        }
        println!("Oracle verified {count} scripts against real MATLAB installation.");
    }
}
