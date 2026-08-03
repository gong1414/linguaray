fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(
                tauri_build::AppManifest::new()
                    .commands(&[
                        "translate", "translate_default", "translate_clipboard",
                        "list_engines", "set_key", "delete_key", "key_status",
                        "get_settings", "set_setting",
                        "a11y_status", "keystore_health", "archive_keystore", "reset_keystore",
                        // S2a data-readiness + provider CRUD.
                        "get_data_readiness",
                        "provider_list", "provider_create", "provider_update",
                        "provider_duplicate", "provider_delete", "provider_reorder",
                        "provider_toggle", "provider_set_key", "provider_set_active",
                    ]),
            ),
    )
    .expect("failed to run tauri build");
}
