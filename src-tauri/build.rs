fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(
                tauri_build::AppManifest::new()
                    .commands(&[
                        "translate", "translate_default", "translate_clipboard",
                        "translate_session",
                        "translate_selection_ipc",
                        "list_engines", "set_key", "delete_key", "key_status",
                        "get_settings", "set_setting",
                        "a11y_status", "keystore_health", "archive_keystore", "reset_keystore",
                        "get_data_readiness",
                        "provider_list", "provider_create", "provider_update",
                        "provider_duplicate", "provider_delete", "provider_reorder",
                        "provider_toggle", "provider_set_key", "provider_set_active",
                        "provider_get_active_selection",
                        "provider_confirm_and_set_active",
                        "provider_get_models", "provider_test_connection",
                        "archive_database",
                        "open_settings_window",
                    ]),
            ),
    )
    .expect("failed to run tauri build");
}
