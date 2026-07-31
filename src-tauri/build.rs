fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(
                tauri_build::AppManifest::new()
                    .commands(&[
                        "translate", "translate_default", "translate_clipboard",
                        "list_engines", "set_key", "delete_key", "key_status",
                        "get_settings", "set_setting", "lookup_dictionary",
                        "a11y_status", "keystore_health", "archive_keystore", "reset_keystore",
                    ]),
            ),
    )
    .expect("failed to run tauri build");
}
