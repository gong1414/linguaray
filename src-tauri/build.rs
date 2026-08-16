fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "translate",
            "translate_default",
            "translate_clipboard",
            "translate_session",
            "translate_selection_ipc",
            "key_status",
            "get_settings",
            "set_setting",
            "a11y_status",
            "screen_capture_status",
            "keystore_health",
            "archive_keystore",
            "reset_keystore",
            "get_data_readiness",
            "provider_list_presets",
            "provider_list",
            "provider_create",
            "provider_update",
            "provider_duplicate",
            "provider_delete",
            "provider_reorder",
            "provider_toggle",
            "provider_set_key",
            "provider_set_active",
            "provider_get_active_selection",
            "provider_confirm_and_set_active",
            "provider_get_models",
            "provider_test_connection",
            "archive_database",
            "open_settings_window",
            "shortcut_list",
            "shortcut_check_conflict",
            "shortcut_save",
            "shortcut_reset_defaults",
            "shortcut_recording_begin",
            "shortcut_recording_end",
            "history_privacy_status",
            "history_set_enabled",
            "history_set_retention",
            "history_clear_all",
            "history_search",
            "history_toggle_favorite",
            "history_delete_session",
            "history_export",
            "vocabulary_add",
            "vocabulary_list",
            "vocabulary_delete",
            "vocabulary_export_file",
            "vocabulary_export_anki",
            "dict_lookup",
            "dict_list_packages",
            "dict_install_package",
            "ocr_capture",
            "ocr_capture_region",
            "ocr_from_image",
            "ocr_recognize_bytes",
            "ocr_from_clipboard",
            "tts_list_voices",
            "tts_speak",
            "tts_stop",
            "external_api_enable",
            "external_api_status",
            "external_api_disable",
            "external_api_regenerate_token",
            "updater_check",
            "updater_download_install",
            "onboarding_status",
            "onboarding_next",
            "onboarding_complete",
            "provider_get_balance",
        ]),
    ))
    .expect("failed to run tauri build");

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rerun-if-changed=src/ocr/macos.m");
        println!("cargo:rerun-if-changed=src/tts/macos.m");
        cc::Build::new()
            .file("src/ocr/macos.m")
            .file("src/tts/macos.m")
            .flag("-fobjc-arc")
            .compile("linguaray_apple");
        println!("cargo:rustc-link-lib=framework=Vision");
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=CoreGraphics");
    }

    // rev-12 / Task A5: write the tray red-dot-overlay + dimmed-pulse PNGs to OUT_DIR.
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    build_tray_icons(&out_dir);
}

// ─── rev-12 / Task A5: generate the tray PNGs (build-time) ───────────────────
// TWO icons are written to OUT_DIR so the runtime embeds them via
// include_bytes!(concat!(env!("OUT_DIR"), "/tray-error-32.png")) and
// ".../tray-active-32.png"). Both are PROGRAMMATIC COMPOSITES over the repo's
// existing app default icon src-tauri/icons/32x32.png (NOT new design assets).
fn build_tray_icons(out_dir: &std::path::Path) {
    use image::{ImageBuffer, Rgba};
    const SIZE: u32 = 32;

    // Load the repo's existing app default icon as the base for BOTH variants.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let base_path = std::path::Path::new(&manifest_dir)
        .join("icons")
        .join("32x32.png");
    let base = image::open(&base_path)
        .expect("open src-tauri/icons/32x32.png (base icon for tray composites)")
        .to_rgba8();

    // ── tray-error-32.png: red-dot OVERLAY on the base (rev-12 P1-2) ─────────
    // Draw a ~10px-diameter dot at the top-right. Center ~(26, 6), radius 5.
    // Color #DC2626 = [220, 38, 38, 255] — frozen danger color (user-specified).
    let mut error_img = base.clone();
    let dot_center: (i32, i32) = (26, 6);
    let dot_radius: i32 = 5;
    let dot_color: [u8; 4] = [220, 38, 38, 255]; // #DC2626
    for y in 0..SIZE as i32 {
        for x in 0..SIZE as i32 {
            let dx = x - dot_center.0;
            let dy = y - dot_center.1;
            if dx * dx + dy * dy <= dot_radius * dot_radius {
                error_img.put_pixel(x as u32, y as u32, Rgba(dot_color));
            }
        }
    }
    let error_path = out_dir.join("tray-error-32.png");
    image::DynamicImage::ImageRgba8(error_img)
        .save(&error_path)
        .expect("write tray-error-32.png to OUT_DIR");

    // ── tray-active-32.png: dimmed variant for the pulse (rev-12 P1-1) ───────
    // Each pixel's RGB scaled to ~60% brightness; alpha unchanged. This is the
    // "dimmed" frame the pulse timer swaps in every 800ms (the visible pulse).
    let mut active_img: ImageBuffer<Rgba<u8>, Vec<u8>> = base.clone();
    for px in active_img.pixels_mut() {
        let channels = px.0;
        px.0 = [
            (channels[0] as u16 * 60 / 100) as u8,
            (channels[1] as u16 * 60 / 100) as u8,
            (channels[2] as u16 * 60 / 100) as u8,
            channels[3], // alpha unchanged
        ];
    }
    let active_path = out_dir.join("tray-active-32.png");
    image::DynamicImage::ImageRgba8(active_img)
        .save(&active_path)
        .expect("write tray-active-32.png to OUT_DIR");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=icons/32x32.png");
}
