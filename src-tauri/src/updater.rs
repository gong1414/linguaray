//! Updater status mapping + the Tauri updater adapter. `map_check` /
//! `progress_bucket` are pure and unit-tested; `check_remote` performs the
//! real network round-trip via tauri-plugin-updater.

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum UpdateCheck {
    UpToDate { version: String },
    Available { current: String, next: String, notes: String },
    Error { message: String },
}

/// Map a remote check onto tray + settings state.
pub fn map_check(current: &str, remote: Result<Option<RemoteUpdate>, String>) -> UpdateCheck {
    match remote {
        Ok(None) => UpdateCheck::UpToDate {
            version: current.to_string(),
        },
        Ok(Some(u)) if u.version == current => UpdateCheck::UpToDate {
            version: current.to_string(),
        },
        Ok(Some(u)) => UpdateCheck::Available {
            current: current.to_string(),
            next: u.version,
            notes: u.notes,
        },
        Err(message) => UpdateCheck::Error { message },
    }
}

#[derive(Debug, Clone)]
pub struct RemoteUpdate {
    pub version: String,
    pub notes: String,
}

/// Real network check via tauri-plugin-updater. Never panics and never returns
/// an Err: the startup check runs unattended, and every failure mode maps onto
/// `UpdateCheck::Error` (which the UI renders non-blockingly).
pub async fn check_remote(app: &tauri::AppHandle) -> UpdateCheck {
    use tauri_plugin_updater::UpdaterExt;
    let updater = match app.updater() {
        Ok(u) => u,
        Err(e) => {
            return UpdateCheck::Error {
                message: format!("updater unavailable: {e}"),
            }
        }
    };
    let remote = match tokio::time::timeout(
        std::time::Duration::from_secs(15),
        updater.check(),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => {
            return UpdateCheck::Error {
                message: "update check timed out".to_string(),
            }
        }
    };
    match remote {
        Ok(Some(u)) => UpdateCheck::Available {
            current: u.current_version.to_string(),
            next: u.version.to_string(),
            notes: u.body.clone().unwrap_or_default(),
        },
        Ok(None) => UpdateCheck::UpToDate {
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        Err(e) => UpdateCheck::Error { message: e.to_string() },
    }
}

/// Quantize download progress into an event bucket so the webview receives at
/// most one `updater-progress` event per whole percent (or per full MiB when
/// the total length is unknown). Returns u64 (not a smaller type) so callers
/// compare buckets without casting; totals of 0 are treated as unknown.
pub fn progress_bucket(downloaded: u64, total: Option<u64>) -> u64 {
    match total {
        Some(t) if t > 0 => (downloaded * 100) / t,
        _ => downloaded / (1024 * 1024),
    }
}

pub fn tray_should_show_update(check: &UpdateCheck) -> bool {
    matches!(check, UpdateCheck::Available { .. })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn available_when_remote_differs() {
        let got = map_check(
            "0.1.0",
            Ok(Some(RemoteUpdate {
                version: "0.2.0".into(),
                notes: "fixes".into(),
            })),
        );
        assert_eq!(
            got,
            UpdateCheck::Available {
                current: "0.1.0".into(),
                next: "0.2.0".into(),
                notes: "fixes".into(),
            }
        );
        assert!(tray_should_show_update(&got));
    }

    #[test]
    fn up_to_date_when_no_remote() {
        let got = map_check("0.1.0", Ok(None));
        assert_eq!(
            got,
            UpdateCheck::UpToDate {
                version: "0.1.0".into()
            }
        );
        assert!(!tray_should_show_update(&got));
    }

    #[test]
    fn progress_bucket_emits_at_most_once_per_percent() {
        let total = Some(10_000);
        // 0%..=100% buckets.
        assert_eq!(progress_bucket(0, total), 0);
        assert_eq!(progress_bucket(9_999, total), 99);
        assert_eq!(progress_bucket(10_000, total), 100);
        // Same percent bucket → identical value (caller skips the emit).
        assert_eq!(progress_bucket(5_000, total), progress_bucket(5_099, total));
    }

    #[test]
    fn progress_bucket_unknown_total_counts_mib() {
        assert_eq!(progress_bucket(0, None), 0);
        assert_eq!(progress_bucket(1024 * 1024 - 1, None), 0);
        assert_eq!(progress_bucket(1024 * 1024, None), 1);
        // A zero total is unusable — treated as unknown, never a division by 0.
        assert_eq!(progress_bucket(5_000_000, Some(0)), 4);
    }
}
