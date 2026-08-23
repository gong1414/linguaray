use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use linguaray_core::TranslationTarget;
use linguaray_engine::{ProviderConfig, ProviderType};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use struct_patch::Patch;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Patch, uniffi::Record)]
#[patch(attribute(derive(Clone, Debug, Default, Deserialize, Serialize, uniffi::Record)))]
pub struct ShortcutSettings {
    #[serde(
        default = "default_toggle_mini_translator_shortcut",
        rename = "toggleMiniTranslator"
    )]
    pub toggle_mini_translator: String,
    #[serde(
        default = "default_extract_text_from_screen_selection_shortcut",
        rename = "extractTextFromScreenSelection",
        alias = "extractFromScreenSelection"
    )]
    pub extract_text_from_screen_selection: String,
    #[serde(
        default = "default_extract_text_from_screen_capture_shortcut",
        rename = "extractTextFromScreenCapture",
        alias = "extractFromScreenCapture"
    )]
    pub extract_text_from_screen_capture: String,
    #[serde(default = "default_capture_ocr_shortcut", rename = "captureOcr")]
    pub capture_ocr: String,
    #[serde(
        default = "default_extract_text_from_clipboard_shortcut",
        rename = "extractTextFromClipboard",
        alias = "extractFromClipboard"
    )]
    pub extract_text_from_clipboard: String,
    #[serde(
        default = "default_translate_input_content_shortcut",
        rename = "translateInputContent"
    )]
    pub translate_input_content: String,
}

impl Default for ShortcutSettings {
    fn default() -> Self {
        Self {
            toggle_mini_translator: default_toggle_mini_translator_shortcut(),
            extract_text_from_screen_selection: default_extract_text_from_screen_selection_shortcut(
            ),
            extract_text_from_screen_capture: default_extract_text_from_screen_capture_shortcut(),
            capture_ocr: default_capture_ocr_shortcut(),
            extract_text_from_clipboard: default_extract_text_from_clipboard_shortcut(),
            translate_input_content: default_translate_input_content_shortcut(),
        }
    }
}

fn default_toggle_mini_translator_shortcut() -> String {
    "Option+1".to_owned()
}

fn default_extract_text_from_screen_selection_shortcut() -> String {
    "Option+Q".to_owned()
}

fn default_extract_text_from_screen_capture_shortcut() -> String {
    "Option+W".to_owned()
}

fn default_capture_ocr_shortcut() -> String {
    "Option+Shift+W".to_owned()
}

fn default_extract_text_from_clipboard_shortcut() -> String {
    "Option+E".to_owned()
}

fn default_translate_input_content_shortcut() -> String {
    "Option+Z".to_owned()
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Patch, uniffi::Record)]
#[patch(attribute(derive(Clone, Debug, Default, Deserialize, Serialize, uniffi::Record)))]
#[serde(default)]
pub struct AppearanceSettings {
    pub language: String,
    #[serde(rename = "themeMode")]
    pub theme_mode: String,
    /// Which palette family the design system paints with: `studio` or
    /// `bright`. Independent of [`theme_mode`], which only picks light vs dark.
    pub theme: String,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            language: "zh-Hans".to_owned(),
            theme_mode: "light".to_owned(),
            theme: "bright".to_owned(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, uniffi::Enum)]
#[serde(rename_all = "camelCase")]
pub enum InputSubmitMode {
    #[default]
    Enter,
    CommandEnter,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Patch, uniffi::Record)]
#[patch(attribute(derive(Clone, Debug, Default, Deserialize, Serialize, uniffi::Record)))]
#[serde(default)]
pub struct GeneralSettings {
    #[serde(rename = "launchAtLogin")]
    pub launch_at_login: bool,
    #[serde(rename = "showInMenuBar")]
    pub show_in_menu_bar: bool,
    // OCR
    #[serde(rename = "defaultOcrService")]
    pub default_ocr_service: String,
    #[serde(rename = "autoCopyDetectedText")]
    pub auto_copy_detected_text: bool,
    // Directory
    #[serde(rename = "defaultDirectoryService")]
    pub default_directory_service: String,
    // Translation
    #[serde(rename = "defaultTranslationService")]
    pub default_translation_service: String,
    #[serde(rename = "translationTargets")]
    pub translation_targets: Vec<TranslationTarget>,
    #[serde(rename = "inputSubmitMode")]
    pub input_submit_mode: InputSubmitMode,
    #[serde(rename = "doubleClickCopyResult")]
    pub double_click_copy_result: bool,
    /// Language codes that the user has marked as "common" / frequently used.
    /// These languages appear first in language selection menus, with the
    /// remaining languages collapsed into a secondary "More languages..." menu.
    #[serde(rename = "commonLanguages")]
    pub common_languages: Vec<String>,
    #[serde(default, rename = "translationServiceOrder")]
    pub translation_service_order: Vec<String>,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            launch_at_login: false,
            show_in_menu_bar: true,
            default_ocr_service: String::new(),
            auto_copy_detected_text: true,
            default_directory_service: String::new(),
            default_translation_service: String::new(),
            translation_targets: Vec::new(),
            input_submit_mode: InputSubmitMode::default(),
            double_click_copy_result: true,
            common_languages: Vec::new(),
            translation_service_order: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Patch, uniffi::Record)]
#[patch(attribute(derive(Clone, Debug, Default, Deserialize, Serialize, uniffi::Record)))]
#[serde(default)]
pub struct AdvancedSettings {
    #[serde(rename = "apiServerEnabled")]
    pub api_server_enabled: bool,
    #[serde(default = "default_api_server_host", rename = "apiServerHost")]
    pub api_server_host: String,
    #[serde(rename = "apiServerPort")]
    pub api_server_port: u16,
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            api_server_enabled: false,
            api_server_host: default_api_server_host(),
            api_server_port: 0,
        }
    }
}

fn default_api_server_host() -> String {
    "127.0.0.1".to_owned()
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, uniffi::Record)]
pub struct ProviderConfigEntry {
    #[serde(default)]
    pub id: String,
    /// Provider type (baidu, deepl, google, etc.)
    #[serde(rename = "type")]
    pub r#type: ProviderType,
    #[serde(default)]
    pub fields: HashMap<String, String>,
    /// Creation timestamp (Unix epoch seconds). Set automatically when a
    /// provider is first created; `None` for providers migrated from an
    /// older version of the settings file.
    #[serde(default, rename = "createdAt", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<u64>,
    #[serde(default, rename = "presetId", skip_serializing_if = "Option::is_none")]
    pub preset_id: Option<String>,
}

impl Default for ProviderConfigEntry {
    fn default() -> Self {
        Self {
            id: String::default(),
            r#type: ProviderType::System,
            fields: HashMap::default(),
            created_at: None,
            preset_id: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize, uniffi::Enum)]
#[serde(rename_all = "camelCase")]
pub enum ServiceType {
    Dictionary,
    Ocr,
    #[default]
    Translation,
    Llm,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, uniffi::Record)]
pub struct ServiceConfigEntry {
    #[serde(default)]
    pub id: String,
    #[serde(default, rename = "providerId")]
    pub provider_id: String,
    #[serde(default)]
    #[serde(rename = "type")]
    pub r#type: ServiceType,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub fields: HashMap<String, String>,
    #[serde(default, rename = "createdAt", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<u64>,
}

impl Default for ServiceConfigEntry {
    fn default() -> Self {
        Self {
            id: String::default(),
            provider_id: String::default(),
            r#type: ServiceType::Translation,
            name: String::default(),
            fields: HashMap::default(),
            created_at: None,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Settings {
    #[serde(default, rename = "lastUpdated")]
    pub last_updated: u64,
    #[serde(
        default,
        skip_serializing_if = "HashMap::is_empty",
        serialize_with = "serialize_providers",
        deserialize_with = "deserialize_providers"
    )]
    pub providers: HashMap<String, ProviderConfigEntry>,
    #[serde(
        default,
        skip_serializing_if = "HashMap::is_empty",
        deserialize_with = "deserialize_services"
    )]
    pub services: HashMap<String, ServiceConfigEntry>,
    #[serde(default)]
    pub general: GeneralSettings,
    #[serde(default)]
    pub shortcuts: ShortcutSettings,
    #[serde(default)]
    pub appearance: AppearanceSettings,
    #[serde(default)]
    pub advanced: AdvancedSettings,
    #[serde(default, rename = "catalogSeedRevision")]
    pub catalog_seed_revision: u32,
}

impl Settings {
    pub fn load(file_path: impl AsRef<Path>) -> Result<Self, String> {
        let path = file_path.as_ref();
        eprintln!("[Settings::load] path: {}", path.display());
        if !path.exists() {
            eprintln!("[Settings::load] file not found, returning defaults");
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path).map_err(|error| {
            format!("failed to read settings file `{}`: {error}", path.display())
        })?;

        let settings: Self = serde_json::from_str(&content).map_err(|error| {
            format!(
                "failed to parse settings file `{}`: {error}",
                path.display()
            )
        })?;
        eprintln!(
            "[Settings::load] loaded {} providers",
            settings.providers.len()
        );

        // System provider is now a normal provider; no special handling needed.

        Ok(settings)
    }

    pub fn save(&self, file_path: impl AsRef<Path>) -> Result<(), String> {
        let path = file_path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create settings directory `{}`: {error}",
                    parent.display()
                )
            })?;
        }

        let content = self.to_pretty_json()?;
        fs::write(path, content).map_err(|error| {
            format!(
                "failed to write settings file `{}`: {error}",
                path.display()
            )
        })
    }

    pub fn to_pretty_json(&self) -> Result<String, String> {
        let root = serde_json::to_value(self)
            .map_err(|error| format!("failed to encode settings: {error}"))?;

        if !root.is_object() {
            return Err("settings root must encode to a JSON object".to_owned());
        }

        serde_json::to_string_pretty(&root)
            .map_err(|error| format!("failed to render settings json: {error}"))
    }

    pub fn touch_last_updated(&mut self) -> Result<(), String> {
        self.last_updated = current_timestamp_millis()?;
        Ok(())
    }
}

fn current_timestamp_millis() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system clock is before unix epoch: {error}"))?
        .as_millis()
        .try_into()
        .map_err(|_| "current timestamp does not fit in u64".to_owned())
}

pub fn apply_catalog_seed(settings: &mut Settings) -> bool {
    apply_catalog_seed_for(settings, cfg!(target_os = "macos"))
}

pub fn apply_catalog_seed_for(settings: &mut Settings, macos: bool) -> bool {
    if settings.catalog_seed_revision >= linguaray_engine::catalog::CATALOG_SEED_REVISION {
        return false;
    }

    // Builds before the provider catalog created `system` eagerly on macOS.
    // Treat that untouched system-only shape as first-run data so existing
    // Flutter testers receive the new working Google Web default as well.
    let legacy_system_only = !settings.providers.is_empty()
        && settings.providers.keys().all(|id| id == "system")
        && settings
            .services
            .values()
            .all(|service| service.provider_id == "system");
    if legacy_system_only && !macos {
        settings.providers.remove("system");
        settings
            .services
            .retain(|_, service| service.provider_id != "system");
    }
    if (settings.providers.is_empty() && settings.services.is_empty()) || legacy_system_only {
        let seed = linguaray_engine::catalog::default_seed(macos);
        for provider in seed.providers {
            settings
                .providers
                .entry(provider.id.clone())
                .or_insert_with(|| ProviderConfigEntry {
                    id: provider.id,
                    r#type: provider.provider_type,
                    fields: HashMap::new(),
                    created_at: None,
                    preset_id: Some(provider.preset_id),
                });
        }
        for service in seed.services {
            let enabled = if service.enabled { "true" } else { "false" };
            settings
                .services
                .entry(service.id.clone())
                .and_modify(|entry| {
                    entry
                        .fields
                        .insert("enabled".to_owned(), enabled.to_owned());
                })
                .or_insert_with(|| ServiceConfigEntry {
                    id: service.id.clone(),
                    provider_id: service.provider_id,
                    r#type: ServiceType::Translation,
                    name: service.id.clone(),
                    fields: HashMap::from([("enabled".to_owned(), enabled.to_owned())]),
                    created_at: None,
                });
        }
        settings.general.default_translation_service = seed.default_translation_service;
        settings.general.default_directory_service = seed.default_dictionary_service;
        settings.general.translation_service_order = seed.translation_service_order;
    }

    // Revision 2 adds a stable, offline dictionary to every existing install.
    // Since this provider did not exist in revision 1, inserting it cannot
    // undo a user's earlier deletion. Once revision 2 is recorded, a later
    // explicit deletion remains respected.
    if settings.catalog_seed_revision < 2 {
        let preset =
            linguaray_engine::catalog::preset_by_id("ecdict").expect("ecdict catalog preset");
        settings
            .providers
            .entry("ecdict".to_owned())
            .or_insert_with(|| ProviderConfigEntry {
                id: "ecdict".to_owned(),
                r#type: preset.engine_type,
                fields: HashMap::new(),
                created_at: None,
                preset_id: Some(preset.id.to_owned()),
            });
        if settings.general.default_directory_service.is_empty() {
            settings.general.default_directory_service = "ecdict+dictionary".to_owned();
        }
    }

    settings.catalog_seed_revision = linguaray_engine::catalog::CATALOG_SEED_REVISION;
    true
}

pub fn append_translation_service_order(settings: &mut Settings, service_id: &str) {
    if !settings
        .general
        .translation_service_order
        .iter()
        .any(|id| id == service_id)
    {
        settings
            .general
            .translation_service_order
            .push(service_id.to_owned());
    }
}

pub fn remove_translation_service_order(settings: &mut Settings, service_id: &str) {
    settings
        .general
        .translation_service_order
        .retain(|id| id != service_id);
}

pub fn remove_provider_from_translation_order(settings: &mut Settings, provider_id: &str) {
    let prefix = format!("{provider_id}+");
    settings
        .general
        .translation_service_order
        .retain(|id| id != provider_id && !id.starts_with(&prefix));
}

pub fn effective_translation_service_order(
    stored: &[String],
    translation_ids: &[String],
    created_at: &HashMap<String, Option<u64>>,
) -> Vec<String> {
    let known: std::collections::HashSet<&String> = translation_ids.iter().collect();
    let mut order: Vec<String> = stored
        .iter()
        .filter(|id| known.contains(id))
        .cloned()
        .collect();
    let mut missing: Vec<String> = translation_ids
        .iter()
        .filter(|id| !order.iter().any(|existing| existing == *id))
        .cloned()
        .collect();
    missing.sort_by(|a, b| {
        created_at
            .get(a)
            .copied()
            .flatten()
            .cmp(&created_at.get(b).copied().flatten())
            .then(a.cmp(b))
    });
    order.extend(missing);
    order
}

pub fn provider_config_from_settings(
    provider: &ProviderConfigEntry,
) -> Result<ProviderConfig, String> {
    let provider_type = provider.r#type;
    let mut options = BTreeMap::new();
    for (key, value) in &provider.fields {
        if key == "presetId" {
            continue;
        }
        options.insert(key.clone(), serde_yaml::Value::String(value.clone()));
    }
    Ok(ProviderConfig {
        provider_type,
        options,
    })
}

fn serialize_providers<S>(
    providers: &HashMap<String, ProviderConfigEntry>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(providers.len()))?;
    for (provider_id, provider) in providers {
        let config = provider_config_from_settings(provider).map_err(serde::ser::Error::custom)?;
        let mut value = provider_config_json_value(&config).map_err(serde::ser::Error::custom)?;
        if let Some(preset_id) = &provider.preset_id {
            if let Value::Object(object) = &mut value {
                object.insert("presetId".to_owned(), Value::String(preset_id.clone()));
            }
        }
        map.serialize_entry(provider_id, &value)?;
    }
    map.end()
}

fn deserialize_providers<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, ProviderConfigEntry>, D::Error>
where
    D: Deserializer<'de>,
{
    let providers = HashMap::<String, Value>::deserialize(deserializer)?;
    Ok(providers
        .into_iter()
        .filter_map(|(provider_id, value)| {
            provider_entry_from_value(&provider_id, value)
                .ok()
                .map(|entry| (provider_id, entry))
        })
        .collect())
}

fn provider_entry_from_value(
    provider_id: &str,
    mut value: Value,
) -> Result<ProviderConfigEntry, String> {
    let preset_id = value.as_object_mut().and_then(|object| {
        object
            .remove("presetId")
            .and_then(|value| value.as_str().map(str::to_owned))
    });
    let config = serde_json::from_value::<ProviderConfig>(value)
        .map_err(|error| format!("invalid provider config `{provider_id}`: {error}"))?;
    let mut entry = provider_entry_from_config(provider_id, &config)?;
    entry.preset_id = preset_id;
    Ok(entry)
}

/// Deserializes the `Settings::services` map leniently: service entries whose
/// `type` is unknown (for example a `tts` service from an older schema that
/// was removed) are skipped instead of failing the whole settings load. This
/// keeps a single stale entry from bricking app startup; the next `save()`
/// rewrites a cleaned, schema-valid file.
fn deserialize_services<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, ServiceConfigEntry>, D::Error>
where
    D: Deserializer<'de>,
{
    let services = HashMap::<String, Value>::deserialize(deserializer)?;
    Ok(services
        .into_iter()
        .filter_map(|(service_id, value)| {
            serde_json::from_value::<ServiceConfigEntry>(value)
                .ok()
                .map(|entry| (service_id, entry))
        })
        .collect())
}

pub fn provider_entry_from_config(
    provider_id: &str,
    config: &ProviderConfig,
) -> Result<ProviderConfigEntry, String> {
    let value = provider_config_json_value(config)?;
    let Value::Object(mut object) = value else {
        return Err("provider config must encode to an object".to_owned());
    };
    object.remove("type");

    let fields = object
        .into_iter()
        .filter_map(|(key, value)| provider_config_field_value(value).map(|value| (key, value)))
        .collect();

    Ok(ProviderConfigEntry {
        id: provider_id.to_owned(),
        r#type: config.provider_type,
        fields,
        created_at: None,
        preset_id: None,
    })
}

pub(crate) fn parse_provider_type(value: &str) -> Result<ProviderType, String> {
    serde_yaml::from_value::<ProviderType>(serde_yaml::Value::String(value.to_owned()))
        .map_err(|error| format!("invalid provider type `{value}`: {error}"))
}

fn provider_config_field_value(value: Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Null => None,
        Value::Array(_) | Value::Object(_) => Some(value.to_string()),
    }
}

fn provider_config_json_value(config: &ProviderConfig) -> Result<Value, String> {
    let mut value = serde_json::to_value(config)
        .map_err(|error| format!("failed to encode provider: {error}"))?;
    normalize_provider_config_keys(&mut value);
    Ok(value)
}

fn normalize_provider_config_keys(value: &mut Value) {
    let Value::Object(object) = value else {
        return;
    };

    for (from, to) in [
        ("api_key", "apiKey"),
        ("app_key", "appKey"),
        ("app_id", "appId"),
        ("base_url", "baseUrl"),
        ("request_id", "requestId"),
        ("secret_id", "secretId"),
        ("secret_key", "secretKey"),
        ("app_secret", "appSecret"),
        ("picture_base_url", "pictureBaseUrl"),
    ] {
        if let Some(value) = object.remove(from) {
            object.insert(to.to_owned(), value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

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
        std::env::temp_dir()
            .join(format!("linguaray-settings-{unique}"))
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
        assert_eq!(settings.catalog_seed_revision, 2);
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
    fn windows_seed_skips_system_and_enables_google_web() {
        let mut settings = Settings::default();
        assert!(apply_catalog_seed_for(&mut settings, false));
        assert!(!settings.providers.contains_key("system"));
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
        assert_eq!(settings.catalog_seed_revision, 2);
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
        assert_eq!(settings.catalog_seed_revision, 2);
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
    fn windows_seed_removes_the_previous_unsupported_system_default() {
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
        assert!(!settings.providers.contains_key("system"));
        assert!(settings.providers.contains_key("google-web"));
        assert_eq!(
            settings.general.default_translation_service,
            "google-web+translation"
        );
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
}
