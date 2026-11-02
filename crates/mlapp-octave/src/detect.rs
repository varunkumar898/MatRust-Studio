//! Detection of local GNU Octave installations across operating systems.

use std::path::{Path, PathBuf};

/// Checks if a usable `octave-cli` executable is available on the host system.
pub fn is_octave_available() -> bool {
    find_octave_binary().is_some()
}

/// Locates the `octave-cli` executable by searching:
/// 1. `OCTAVE_BIN` explicit environment variable
/// 2. `OCTAVE_ROOT` environment variable
/// 3. System `PATH`
/// 4. Platform-specific default install directories
pub fn find_octave_binary() -> Option<PathBuf> {
    // 1. Direct explicit binary variable
    if let Some(bin_var) = std::env::var_os("OCTAVE_BIN") {
        let p = PathBuf::from(bin_var);
        if p.is_file() {
            return Some(p);
        }
    }

    // 2. OCTAVE_ROOT directory
    if let Some(root_var) = std::env::var_os("OCTAVE_ROOT") {
        let root = PathBuf::from(root_var);
        let candidates = [
            root.join("bin").join(if cfg!(windows) {
                "octave-cli.exe"
            } else {
                "octave-cli"
            }),
            root.join("mingw64").join("bin").join("octave-cli.exe"),
            root.join(if cfg!(windows) {
                "octave-cli.exe"
            } else {
                "octave-cli"
            }),
        ];
        for c in candidates {
            if c.is_file() {
                return Some(c);
            }
        }
    }

    // 3. Search PATH
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let names = if cfg!(windows) {
                vec!["octave-cli.exe", "octave.exe"]
            } else {
                vec!["octave-cli", "octave"]
            };
            for name in names {
                let candidate = dir.join(name);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }

    // 4. Platform-specific default directories
    #[cfg(target_os = "windows")]
    {
        let roots = ["C:\\Program Files\\GNU Octave", "C:\\Octave"];
        for r in roots {
            let p = Path::new(r);
            if p.is_dir() {
                if let Ok(entries) = std::fs::read_dir(p) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let bin1 = path.join("mingw64").join("bin").join("octave-cli.exe");
                            if bin1.is_file() {
                                return Some(bin1);
                            }
                            let bin2 = path.join("bin").join("octave-cli.exe");
                            if bin2.is_file() {
                                return Some(bin2);
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let standard_paths = [
            "/usr/bin/octave-cli",
            "/usr/local/bin/octave-cli",
            "/usr/bin/octave",
            "/snap/bin/octave",
        ];
        for sp in standard_paths {
            let p = Path::new(sp);
            if p.is_file() {
                return Some(p.to_path_buf());
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let standard_paths = [
            "/opt/homebrew/bin/octave-cli",
            "/usr/local/bin/octave-cli",
            "/Applications/Octave.app/Contents/Resources/usr/bin/octave-cli",
        ];
        for sp in standard_paths {
            let p = Path::new(sp);
            if p.is_file() {
                return Some(p.to_path_buf());
            }
        }
    }

    None
}
