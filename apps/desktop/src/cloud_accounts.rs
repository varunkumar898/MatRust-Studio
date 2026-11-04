//! Cloud drive provider registry for the Cloud Drives dialog.
//!
//! Security model (deliberately simple): MATSTUDIO never asks for account
//! passwords and never stores credentials or tokens. Sign-in happens inside
//! the provider's own desktop app or in the user's browser (via the official
//! help links below). Once the provider syncs files to a local folder,
//! MATSTUDIO reads that folder like any other folder - no network code, no
//! account access, nothing that can leak a credential.
//!
//! Native in-app OAuth sign-in is deliberately NOT implemented: it would
//! require an owner-registered OAuth client id for each provider. That is
//! tracked in docs/BACKLOG.md (Cloud sign-in (OAuth) - deferred).

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// One supported cloud-drive provider.
pub struct CloudProvider {
    /// Human-readable name shown in the dialog.
    pub label: &'static str,
    /// Environment variables the provider's Windows client sets.
    pub env_keys: &'static [&'static str],
    /// Profile-relative sync folders the client creates by default.
    pub home_folders: &'static [&'static str],
    /// Official page where the user signs in or downloads the sync client.
    /// Opened in the user's browser, so credentials stay out of this app.
    pub official_url: &'static str,
}

pub const ONEDRIVE: CloudProvider = CloudProvider {
    label: "OneDrive (Microsoft)",
    env_keys: &["OneDrive", "OneDriveCommercial", "OneDriveConsumer"],
    home_folders: &["OneDrive"],
    official_url: "https://www.microsoft.com/microsoft-365/onedrive/onedrive-for-business",
};

pub const GOOGLE_DRIVE: CloudProvider = CloudProvider {
    label: "Google Drive",
    env_keys: &[],
    home_folders: &["Google Drive", "GoogleDrive\\My Drive", "GoogleDrive"],
    official_url: "https://www.google.com/drive/download/",
};

pub const DROPBOX: CloudProvider = CloudProvider {
    label: "Dropbox",
    env_keys: &["Dropbox"],
    home_folders: &["Dropbox"],
    official_url: "https://www.dropbox.com/install",
};

pub const ICLOUD: CloudProvider = CloudProvider {
    label: "iCloud Drive",
    env_keys: &[],
    home_folders: &["iCloudDrive", "iCloud Drive"],
    official_url: "https://www.icloud.com/",
};

/// Providers in dialog order.
pub const PROVIDERS: &[&CloudProvider] = &[&ONEDRIVE, &GOOGLE_DRIVE, &DROPBOX, &ICLOUD];

/// Find the provider's synced folder. `env` is the environment lookup so the
/// pure logic stays testable without mutating process environment variables.
/// A candidate counts only if it exists as a directory.
pub fn detect_sync_folder(
    provider: &CloudProvider,
    home: Option<&Path>,
    env: &dyn Fn(&str) -> Option<OsString>,
) -> Option<PathBuf> {
    for key in provider.env_keys {
        if let Some(value) = env(key) {
            let path = PathBuf::from(value);
            if path.is_dir() {
                return Some(path);
            }
        }
    }
    let home = home?;
    for folder in provider.home_folders {
        let path = home.join(folder);
        if path.is_dir() {
            return Some(path);
        }
    }
    None
}

/// The live detection used by the dialog.
pub fn live_sync_folder(provider: &CloudProvider) -> Option<PathBuf> {
    let env = |key: &str| std::env::var_os(key);
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from);
    detect_sync_folder(provider, home.as_deref(), &env)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn missing_env(_key: &str) -> Option<OsString> {
        None
    }

    fn temp_home(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "rustlab-cloud-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).expect("create temporary home");
        dir
    }

    #[test]
    fn detection_prefers_the_environment_variable() {
        let root = temp_home("env");
        let env_dir = root.join("onedrive-env");
        std::fs::create_dir_all(&env_dir).expect("create env folder");
        let home_dir = root.join("OneDrive");
        std::fs::create_dir_all(&home_dir).expect("create home folder");
        let found = detect_sync_folder(&ONEDRIVE, Some(&root), &|key: &str| {
            (key == "OneDrive").then(|| env_dir.clone().into_os_string())
        });
        assert_eq!(found, Some(env_dir));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn detection_falls_back_to_known_home_folders() {
        let root = temp_home("home");
        let drive = root.join("Google Drive");
        std::fs::create_dir_all(&drive).expect("create Google Drive folder");
        let found = detect_sync_folder(&GOOGLE_DRIVE, Some(&root), &missing_env);
        assert_eq!(found, Some(drive));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn detection_returns_none_when_nothing_exists() {
        let root = temp_home("empty");
        let found = detect_sync_folder(&DROPBOX, Some(&root), &missing_env);
        assert_eq!(found, None);
        let found = detect_sync_folder(&ICLOUD, Some(&root), &missing_env);
        assert_eq!(found, None);
        let _ = std::fs::remove_dir_all(&root);
    }
}
