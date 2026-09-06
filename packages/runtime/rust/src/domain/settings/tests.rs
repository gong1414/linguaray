use std::fs;

use serde_json::Value;

use super::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[test]
fn provider_entry_config_round_trip_preserves_identity_and_fields() {
    let entry = ProviderConfigEntry {
        id: "deepl-main".to_owned(),
        r#type: ProviderType::DeepL,
        fields: HashMap::from([
            ("apiKey".to_owned(), "test-key".to_owned()),
            ("defaultModel".to_owned(), "gpt-4o-mini".to_owned()),
        ]),
        created_at: Some(1_700_000_000),
        preset_id: Some("deepl-pro".to_owned()),
    };
    let config = provider_config_from_settings(&entry).unwrap();
    let back = provider_entry_from_config("deepl-main", &config).unwrap();
    // created_at is entry metadata that deliberately does not live in the
    // engine-side config; everything else must survive the round trip.
    assert_eq!(back.id, entry.id);
    assert_eq!(back.r#type, entry.r#type);
    assert_eq!(back.fields, entry.fields);
}

fn temp_settings_file() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos();
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir()
        .join(format!(
            "linguaray-settings-{}-{unique}-{sequence}",
            std::process::id()
        ))
        .join("settings.json")
}

#[test]
fn load_missing_file_returns_defaults() {
    let file_path = temp_settings_file();
    let settings = Settings::load(&file_path).expect("failed to load settings");

    assert!(settings.providers.is_empty());
    assert_eq!(settings.general, GeneralSettings::default());
    assert_eq!(settings.shortcuts, ShortcutSettings::default());
    assert_eq!(settings.appearance, AppearanceSettings::default());
    assert_eq!(settings.advanced, AdvancedSettings::default());
}

#[test]
fn load_settings_schema() {
    let path = temp_settings_file();
    fs::create_dir_all(path.parent().unwrap()).expect("failed to create temp dir");
    fs::write(
        &path,
        r#"{
"shortcuts": {
"toggleMiniTranslator": "Command+Shift+Space",
"extractTextFromScreenSelection": "Command+Shift+1",
"extractTextFromScreenCapture": "Command+Shift+2",
"captureOcr": "Command+Shift+4",
"extractTextFromClipboard": "Command+Shift+3",
"translateInputContent": "Option+Z"
  },
  "appearance": {
"language": "en",
"themeMode": "dark"
  },
  "general": {
"launchAtLogin": true,
"showInMenuBar": false
  },
  "advanced": {},
  "providers": {
"deepl-main": {
  "type": "deepl",
  "appKey": "test-key"
}
  },
  "lastUpdated": 1710000000000
}"#,
    )
    .expect("failed to write settings");

    let settings = Settings::load(&path).expect("failed to load settings");
    assert_eq!(settings.last_updated, 1710000000000);
    assert_eq!(
        settings.shortcuts.toggle_mini_translator,
        "Command+Shift+Space"
    );
    assert_eq!(
        settings.shortcuts.extract_text_from_screen_selection,
        "Command+Shift+1"
    );
    assert_eq!(
        settings.shortcuts.extract_text_from_screen_capture,
        "Command+Shift+2"
    );
    assert_eq!(settings.shortcuts.capture_ocr, "Command+Shift+4");
    assert_eq!(
        settings.shortcuts.extract_text_from_clipboard,
        "Command+Shift+3"
    );
    assert_eq!(settings.shortcuts.translate_input_content, "Option+Z");
    assert_eq!(settings.appearance.language, "en");
    assert_eq!(settings.appearance.theme_mode, "dark");
    // The fixture predates `theme`, so loading it must fall back to the
    // default family rather than failing.
    assert_eq!(settings.appearance.theme, "bright");
    assert!(settings.general.launch_at_login);
    assert!(!settings.general.show_in_menu_bar);
    assert_eq!(settings.providers.len(), 1);
    let provider = settings.providers.get("deepl-main").unwrap();
    assert_eq!(provider.id, "deepl-main");
    assert_eq!(provider.r#type, ProviderType::DeepL);
    let parsed = provider_config_from_settings(provider).unwrap();
    assert_eq!(parsed.provider_type.as_str(), "deepl");
    assert_eq!(
        parsed.options.get("appKey"),
        Some(&serde_yaml::Value::String("test-key".to_owned()))
    );
}

#[test]
fn load_skips_services_with_unknown_type() {
    // Regression test: a stale `tts` service entry (removed from the schema)
    // must not brick the whole settings load. Valid entries are kept and the
    // unknown one is dropped.
    let path = temp_settings_file();
    fs::create_dir_all(path.parent().unwrap()).expect("failed to create temp dir");
    fs::write(
        &path,
        r#"{
  "general": {
"defaultTtsService": "system+tts"
  },
  "providers": {
"system": { "type": "system" }
  },
  "services": {
"system+tts": {
  "fields": {},
  "id": "system+tts",
  "name": "System TTS",
  "providerId": "system",
  "type": "tts"
},
"system+translation": {
  "fields": {},
  "id": "system+translation",
  "name": "System Translation",
  "providerId": "system",
  "type": "translation"
}
  }
}"#,
    )
    .expect("failed to write settings");

    let settings = Settings::load(&path).expect("failed to load settings");

    // The stale `tts` entry is skipped, the valid one is preserved.
    assert!(!settings.services.contains_key("system+tts"));
    assert!(settings.services.contains_key("system+translation"));
    assert_eq!(settings.services.len(), 1);
    let translation = settings.services.get("system+translation").unwrap();
    assert_eq!(translation.provider_id, "system");
    assert_eq!(translation.r#type, ServiceType::Translation);
}

#[test]
fn load_shortcuts_accepts_legacy_extract_keys() {
    let path = temp_settings_file();
    fs::create_dir_all(path.parent().unwrap()).expect("failed to create temp dir");
    fs::write(
        &path,
        r#"{
  "shortcuts": {
"extractFromScreenSelection": "Command+Shift+1",
"extractFromScreenCapture": "Command+Shift+2",
"extractFromClipboard": "Command+Shift+3"
  }
}"#,
    )
    .expect("failed to write settings");

    let settings = Settings::load(&path).expect("failed to load settings");
    assert_eq!(
        settings.shortcuts.extract_text_from_screen_selection,
        "Command+Shift+1"
    );
    assert_eq!(
        settings.shortcuts.extract_text_from_screen_capture,
        "Command+Shift+2"
    );
    assert_eq!(
        settings.shortcuts.extract_text_from_clipboard,
        "Command+Shift+3"
    );
}

#[test]
fn load_shortcuts_uses_field_defaults_for_missing_keys() {
    let path = temp_settings_file();
    fs::create_dir_all(path.parent().unwrap()).expect("failed to create temp dir");
    fs::write(
        &path,
        r#"{
  "shortcuts": {
"toggleMiniTranslator": "Command+Shift+Space"
  }
}"#,
    )
    .expect("failed to write settings");

    let settings = Settings::load(&path).expect("failed to load settings");
    assert_eq!(
        settings.shortcuts.toggle_mini_translator,
        "Command+Shift+Space"
    );
    assert_eq!(
        settings.shortcuts.extract_text_from_screen_selection,
        "Option+Q"
    );
    assert_eq!(
        settings.shortcuts.extract_text_from_screen_capture,
        "Option+W"
    );
    assert_eq!(settings.shortcuts.capture_ocr, "Option+Shift+W");
    assert_eq!(settings.shortcuts.extract_text_from_clipboard, "Option+E");
    assert_eq!(settings.shortcuts.translate_input_content, "Option+Z");
}

#[test]
fn save_writes_settings_schema() {
    let path = temp_settings_file();
    fs::create_dir_all(path.parent().unwrap()).expect("failed to create temp dir");

    let mut settings = Settings::default();
    settings.shortcuts.toggle_mini_translator = "Command+Shift+Space".to_owned();
    settings.shortcuts.extract_text_from_screen_selection = "Command+Shift+1".to_owned();
    settings.shortcuts.extract_text_from_screen_capture = "Command+Shift+2".to_owned();
    settings.shortcuts.extract_text_from_clipboard = "Command+Shift+3".to_owned();
    settings.shortcuts.translate_input_content = "Option+Z".to_owned();
    settings.appearance.language = "en".to_owned();
    settings.appearance.theme_mode = "system".to_owned();
    settings.general.launch_at_login = true;
    settings.general.show_in_menu_bar = false;
    settings.providers.insert(
        "deepl-main".to_owned(),
        ProviderConfigEntry {
            id: "deepl-main".to_owned(),
            r#type: ProviderType::DeepL,
            fields: HashMap::from([("appKey".to_owned(), "test-key".to_owned())]),
            created_at: None,
            preset_id: Some("deepl-pro".to_owned()),
        },
    );
    settings.save(&path).expect("failed to save settings");

    let saved = fs::read_to_string(path).expect("failed to read saved settings");
    let json = serde_json::from_str::<Value>(&saved).expect("invalid saved json");
    assert_eq!(
        json.pointer("/shortcuts/toggleMiniTranslator").cloned(),
        Some(Value::String("Command+Shift+Space".to_owned()))
    );
    assert_eq!(
        json.pointer("/shortcuts/extractTextFromScreenSelection")
            .cloned(),
        Some(Value::String("Command+Shift+1".to_owned()))
    );
    assert_eq!(
        json.pointer("/shortcuts/extractTextFromScreenCapture")
            .cloned(),
        Some(Value::String("Command+Shift+2".to_owned()))
    );
    assert_eq!(
        json.pointer("/shortcuts/extractTextFromClipboard").cloned(),
        Some(Value::String("Command+Shift+3".to_owned()))
    );
    assert_eq!(
        json.pointer("/shortcuts/translateInputContent").cloned(),
        Some(Value::String("Option+Z".to_owned()))
    );
    assert_eq!(
        json.pointer("/appearance/language").cloned(),
        Some(Value::String("en".to_owned()))
    );
    assert_eq!(
        json.pointer("/appearance/themeMode").cloned(),
        Some(Value::String("system".to_owned()))
    );
    assert_eq!(
        json.pointer("/general/launchAtLogin").cloned(),
        Some(Value::Bool(true))
    );
    assert_eq!(
        json.pointer("/general/showInMenuBar").cloned(),
        Some(Value::Bool(false))
    );
    assert_eq!(
        json.pointer("/providers/deepl-main/type").cloned(),
        Some(Value::String("deepl".to_owned()))
    );
    assert_eq!(
        json.pointer("/providers/deepl-main/appKey").cloned(),
        Some(Value::String("test-key".to_owned()))
    );
    assert!(json.pointer("/providers/deepl-main/id").is_none());
    assert_eq!(
        json.pointer("/providers/deepl-main/presetId").cloned(),
        Some(Value::String("deepl-pro".to_owned()))
    );
    assert_eq!(json.get("lastUpdated").and_then(Value::as_u64), Some(0));
}

#[test]
fn engine_config_is_flattened() {
    let settings = Settings::default();
    let json = serde_json::from_str::<Value>(&settings.to_pretty_json().unwrap())
        .expect("invalid settings json");

    assert!(json.get("engine").is_none());
    assert!(json.get("general").is_some());
    assert!(json.get("shortcuts").is_some());
    assert!(json.get("providers").is_none());
    assert!(json.get("appearance").is_some());
    assert!(json.get("advanced").is_some());
    assert!(json.get("lastUpdated").is_some());
}

#[test]
fn macos_seed_uses_google_web_and_keeps_system_available() {
    let mut settings = Settings::default();
    assert!(apply_catalog_seed_for(&mut settings, true));
    assert_eq!(settings.catalog_seed_revision, 3);
    assert!(settings.providers.contains_key("ecdict"));
    assert!(settings.providers.contains_key("system"));
    assert!(settings.providers.contains_key("google-web"));
    assert_eq!(
        settings.providers["google-web"].preset_id.as_deref(),
        Some("google-web")
    );
    assert_eq!(
        settings.general.default_translation_service,
        "google-web+translation"
    );
    assert_eq!(
        settings.general.default_directory_service,
        "ecdict+dictionary"
    );
    assert_eq!(settings.general.default_ocr_service, "system+ocr");
    assert_eq!(
        settings.services["google-web+translation"]
            .fields
            .get("enabled"),
        Some(&"true".to_owned())
    );
    assert_eq!(
        settings.services["system+translation"]
            .fields
            .get("enabled"),
        Some(&"false".to_owned())
    );
    assert_eq!(
        settings.general.translation_service_order,
        vec![
            "google-web+translation".to_owned(),
            "system+translation".to_owned()
        ]
    );
    assert!(!apply_catalog_seed_for(&mut settings, true));
}

#[test]
fn windows_seed_keeps_system_ocr_and_enables_google_web() {
    let mut settings = Settings::default();
    assert!(apply_catalog_seed_for(&mut settings, false));
    assert_eq!(settings.catalog_seed_revision, 3);
    assert!(settings.providers.contains_key("system"));
    assert!(settings.providers.contains_key("ecdict"));
    assert_eq!(
        settings.general.default_translation_service,
        "google-web+translation"
    );
    assert_eq!(
        settings.services["google-web+translation"]
            .fields
            .get("enabled"),
        Some(&"true".to_owned())
    );
    assert_eq!(settings.general.default_ocr_service, "system+ocr");
    assert!(!settings.services.contains_key("system+translation"));
    assert!(!settings.providers.contains_key("bing-web"));
}

#[test]
fn seed_preserves_existing_user_configuration() {
    let mut settings = Settings::default();
    settings.providers.insert(
        "deepl-main".to_owned(),
        ProviderConfigEntry {
            id: "deepl-main".to_owned(),
            r#type: ProviderType::DeepL,
            fields: HashMap::from([("authKey".to_owned(), "ref".to_owned())]),
            created_at: None,
            preset_id: Some("deepl-pro".to_owned()),
        },
    );
    settings.general.default_translation_service = "deepl-main+translation".to_owned();
    assert!(apply_catalog_seed_for(&mut settings, true));
    assert_eq!(settings.catalog_seed_revision, 3);
    assert_eq!(settings.providers.len(), 2);
    assert_eq!(
        settings.general.default_translation_service,
        "deepl-main+translation"
    );
    assert!(!settings.providers.contains_key("google-web"));
    assert!(settings.providers.contains_key("ecdict"));
}

#[test]
fn revision_two_adds_offline_dictionary_without_replacing_user_defaults() {
    let mut settings = Settings {
        catalog_seed_revision: 1,
        ..Settings::default()
    };
    settings.providers.insert(
        "deepl-main".to_owned(),
        ProviderConfigEntry {
            id: "deepl-main".to_owned(),
            r#type: ProviderType::DeepL,
            fields: HashMap::new(),
            created_at: None,
            preset_id: Some("deepl-pro".to_owned()),
        },
    );
    settings.general.default_translation_service = "deepl-main+translation".to_owned();

    assert!(apply_catalog_seed_for(&mut settings, true));
    assert_eq!(settings.catalog_seed_revision, 3);
    assert!(settings.providers.contains_key("ecdict"));
    assert_eq!(
        settings.general.default_translation_service,
        "deepl-main+translation"
    );
    assert_eq!(
        settings.general.default_directory_service,
        "ecdict+dictionary"
    );

    settings.providers.remove("ecdict");
    assert!(!apply_catalog_seed_for(&mut settings, true));
    assert!(!settings.providers.contains_key("ecdict"));
}

#[test]
fn seed_upgrades_the_previous_system_only_default() {
    let mut settings = Settings::default();
    settings.providers.insert(
        "system".to_owned(),
        ProviderConfigEntry {
            id: "system".to_owned(),
            r#type: ProviderType::System,
            fields: HashMap::new(),
            created_at: None,
            preset_id: None,
        },
    );
    settings.services.insert(
        "system+translation".to_owned(),
        ServiceConfigEntry {
            id: "system+translation".to_owned(),
            provider_id: "system".to_owned(),
            r#type: ServiceType::Translation,
            name: "System Translation".to_owned(),
            fields: HashMap::new(),
            created_at: None,
        },
    );
    settings.general.default_translation_service = "system+translation".to_owned();

    assert!(apply_catalog_seed_for(&mut settings, true));
    assert!(settings.providers.contains_key("google-web"));
    assert_eq!(
        settings.general.default_translation_service,
        "google-web+translation"
    );
    assert_eq!(
        settings.services["system+translation"]
            .fields
            .get("enabled")
            .map(String::as_str),
        Some("false")
    );
    assert_eq!(
        settings.services["system+translation"].name,
        "System Translation"
    );
}

#[test]
fn windows_seed_reuses_the_previous_system_provider_for_ocr() {
    let mut settings = Settings::default();
    settings.providers.insert(
        "system".to_owned(),
        ProviderConfigEntry {
            id: "system".to_owned(),
            r#type: ProviderType::System,
            fields: HashMap::new(),
            created_at: None,
            preset_id: None,
        },
    );

    assert!(apply_catalog_seed_for(&mut settings, false));
    assert!(settings.providers.contains_key("system"));
    assert!(settings.providers.contains_key("google-web"));
    assert_eq!(
        settings.general.default_translation_service,
        "google-web+translation"
    );
    assert_eq!(settings.general.default_ocr_service, "system+ocr");
    assert!(!settings.services.contains_key("system+translation"));
}

#[test]
fn deleted_default_is_not_recreated() {
    let mut settings = Settings::default();
    apply_catalog_seed_for(&mut settings, false);
    settings.providers.remove("google-web");
    settings.services.remove("google-web+translation");
    settings.general.translation_service_order.clear();
    settings.general.default_translation_service.clear();
    assert!(!apply_catalog_seed_for(&mut settings, false));
    assert!(!settings.providers.contains_key("google-web"));
    assert!(settings.providers.contains_key("ecdict"));
    assert!(settings.services.is_empty());
}

#[test]
fn translation_order_appends_missing_by_created_at_then_id() {
    let stored = vec!["b+translation".to_owned()];
    let ids = vec![
        "a+translation".to_owned(),
        "b+translation".to_owned(),
        "c+translation".to_owned(),
    ];
    let created = HashMap::from([
        ("a+translation".to_owned(), Some(20)),
        ("b+translation".to_owned(), Some(1)),
        ("c+translation".to_owned(), Some(20)),
    ]);
    assert_eq!(
        effective_translation_service_order(&stored, &ids, &created),
        vec![
            "b+translation".to_owned(),
            "a+translation".to_owned(),
            "c+translation".to_owned()
        ]
    );
}

#[test]
fn secret_field_values_are_not_copied_into_engine_as_preset_metadata() {
    let entry = ProviderConfigEntry {
        id: "openai".to_owned(),
        r#type: ProviderType::OpenAi,
        fields: HashMap::from([
            (
                "apiKey".to_owned(),
                "linguaray-secret://openai/apiKey".to_owned(),
            ),
            ("presetId".to_owned(), "openai".to_owned()),
        ]),
        created_at: None,
        preset_id: Some("openai".to_owned()),
    };
    let config = provider_config_from_settings(&entry).unwrap();
    assert!(!config.options.contains_key("presetId"));
    assert_eq!(
        config.options.get("apiKey"),
        Some(&serde_yaml::Value::String(
            "linguaray-secret://openai/apiKey".to_owned()
        ))
    );
}
