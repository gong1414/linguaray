//! Task A4 (P1-6): assert the capability set contains every required permission.
//! This is an integration test, not a grep — it parses the JSON and validates the
//! structure so a missing/misnamed permission fails loudly.
use std::collections::HashSet;
use std::fs;

fn permission_set(path: &str) -> HashSet<String> {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read {path}: {e}"));
    let v: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("parse {path}: {e}"));
    v["permissions"]
        .as_array()
        .unwrap_or_else(|| panic!("{path} has no permissions array"))
        .iter()
        .map(|p| p.as_str().unwrap_or("").to_string())
        .collect()
}

#[test]
fn input_window_authorizes_session_and_provider_list() {
    let perms = permission_set("capabilities/input.json");
    for required in ["allow-translate-default", "allow-translate-session", "allow-provider-list"] {
        assert!(
            perms.contains(required),
            "input.json missing {required}; has: {:?}",
            perms
        );
    }
}

#[test]
fn popup_window_authorizes_selection_clipboard_and_settings() {
    let perms = permission_set("capabilities/popup.json");
    for required in [
        "allow-provider-list",
        "allow-provider-get-active-selection",
        "allow-translate-selection-ipc",
        "allow-open-settings-window",
        "clipboard-manager:allow-write-text",
    ] {
        assert!(
            perms.contains(required),
            "popup.json missing {required}; has: {:?}",
            perms
        );
    }
}

#[test]
fn main_window_authorizes_every_new_command() {
    let perms = permission_set("capabilities/main.json");
    for required in [
        "allow-translate-session",
        "allow-translate-selection-ipc",
        "allow-provider-get-active-selection",
        "allow-open-settings-window",
    ] {
        assert!(
            perms.contains(required),
            "main.json missing {required}; has: {:?}",
            perms
        );
    }
}
