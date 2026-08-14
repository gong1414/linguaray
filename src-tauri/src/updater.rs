//! Updater status mapping. Network check is optional; tests drive the mapper.

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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

/// Production check: no configured updater endpoint → up to date.
pub fn check_current() -> UpdateCheck {
    map_check(env!("CARGO_PKG_VERSION"), Ok(None))
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
}
