//! Detection of local MATLAB installations across operating systems.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Checks if a usable MATLAB executable is available on the system.
pub fn is_matlab_available() -> bool {
    find_matlab_binary().is_some()
}

/// Locates the `matlab` executable by searching:
/// 1. `MATLAB_ROOT` environment variable
/// 2. `PATH` environment variable
/// 3. Standard Windows registry and default installation directories
/// 4. Standard macOS `/Applications/MATLAB_*.app` bundles
/// 5. Standard Linux `/usr/local/MATLAB/*` and `/opt/MATLAB/*` paths
pub fn find_matlab_binary() -> Option<PathBuf> {
    // 1. Check explicit MATLAB_ROOT environment variable
    if let Some(root) = std::env::var_os("MATLAB_ROOT") {
        let root_path = PathBuf::from(root);
        if let Some(bin) = probe_matlab_in_dir(&root_path) {
            return Some(bin);
        }
    }

    // 2. Check PATH
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate_exe = dir.join(if cfg!(windows) {
                "matlab.exe"
            } else {
                "matlab"
            });
            if candidate_exe.is_file() {
                return Some(candidate_exe);
            }
            if cfg!(windows) {
                let candidate_bat = dir.join("matlab.bat");
                if candidate_bat.is_file() {
                    return Some(candidate_bat);
                }
            }
        }
    }

    // 3. Platform-specific fallback detection
    #[cfg(target_os = "windows")]
    {
        // Check Windows registry HKLM\SOFTWARE\MathWorks\MATLAB via reg query
        if let Some(bin) = probe_windows_registry() {
            return Some(bin);
        }

        // Check C:\Program Files\MATLAB
        let pf = Path::new("C:\\Program Files\\MATLAB");
        if pf.is_dir() {
            if let Ok(entries) = std::fs::read_dir(pf) {
                let mut release_dirs: Vec<PathBuf> = entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.is_dir())
                    .collect();
                // Sort descending so newest release (e.g. R2026b, R2025a) is tried first
                release_dirs.sort_by(|a, b| b.cmp(a));
                for rdir in release_dirs {
                    if let Some(bin) = probe_matlab_in_dir(&rdir) {
                        return Some(bin);
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let apps = Path::new("/Applications");
        if apps.is_dir() {
            if let Ok(entries) = std::fs::read_dir(apps) {
                let mut app_dirs: Vec<PathBuf> = entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| {
                        p.is_dir()
                            && p.file_name()
                                .and_then(|n| n.to_str())
                                .map(|s| s.starts_with("MATLAB_") && s.ends_with(".app"))
                                .unwrap_or(false)
                    })
                    .collect();
                app_dirs.sort_by(|a, b| b.cmp(a));
                for app in app_dirs {
                    let bin = app.join("bin").join("matlab");
                    if bin.is_file() {
                        return Some(bin);
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        for base in &["/usr/local/MATLAB", "/opt/MATLAB"] {
            let base_path = Path::new(base);
            if base_path.is_dir() {
                if let Ok(entries) = std::fs::read_dir(base_path) {
                    let mut dirs: Vec<PathBuf> = entries
                        .filter_map(|e| e.ok())
                        .map(|e| e.path())
                        .filter(|p| p.is_dir())
                        .collect();
                    dirs.sort_by(|a, b| b.cmp(a));
                    for rdir in dirs {
                        let bin = rdir.join("bin").join("matlab");
                        if bin.is_file() {
                            return Some(bin);
                        }
                    }
                }
            }
        }
    }

    None
}

/// Probes standard relative locations inside a MATLAB root directory.
fn probe_matlab_in_dir(root: &Path) -> Option<PathBuf> {
    let candidates = [
        root.join("bin").join("matlab.exe"),
        root.join("bin").join("win64").join("matlab.exe"),
        root.join("bin").join("matlab.bat"),
        root.join("bin").join("matlab"),
        root.join("matlab.exe"),
    ];

    for c in &candidates {
        if c.is_file() {
            return Some(c.clone());
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn probe_windows_registry() -> Option<PathBuf> {
    let output = Command::new("reg")
        .args(["query", r"HKLM\SOFTWARE\MathWorks\MATLAB", "/s"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.contains("REG_SZ") && (trimmed.contains("MATLAB") || trimmed.contains("bin")) {
            let parts: Vec<&str> = trimmed.split("REG_SZ").collect();
            if parts.len() >= 2 {
                let val = parts[1].trim();
                let p = PathBuf::from(val);
                if p.is_file() {
                    return Some(p);
                }
                if let Some(bin) = probe_matlab_in_dir(&p) {
                    return Some(bin);
                }
            }
        }
    }
    None
}
