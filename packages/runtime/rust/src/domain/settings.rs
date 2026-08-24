use std::collections::HashMap;

use linguaray_core::TranslationTarget;
use linguaray_engine::ProviderType;
use serde::{Deserialize, Serialize};
use struct_patch::Patch;

mod engine_mapping;
mod persistence;
mod seed;
mod service_order;

pub(crate) use engine_mapping::parse_provider_type;
use engine_mapping::{deserialize_providers, deserialize_services, serialize_providers};
pub use engine_mapping::{provider_config_from_settings, provider_entry_from_config};
pub use seed::{apply_catalog_seed, apply_catalog_seed_for};
pub use service_order::{
    append_translation_service_order, effective_translation_service_order,
    remove_provider_from_translation_order, remove_translation_service_order,
};

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
    #[serde(default, rename = "silentCaptureOcr")]
    pub silent_capture_ocr: String,
    #[serde(default, rename = "fileOcr")]
    pub file_ocr: String,
    #[serde(default, rename = "clipboardOcr")]
    pub clipboard_ocr: String,
    #[serde(default, rename = "showOcrWindow")]
    pub show_ocr_window: String,
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
            silent_capture_ocr: String::new(),
            file_ocr: String::new(),
            clipboard_ocr: String::new(),
            show_ocr_window: String::new(),
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
    #[serde(default = "default_proxy_mode", rename = "proxyMode")]
    pub proxy_mode: String,
    #[serde(default, rename = "proxyUrl")]
    pub proxy_url: String,
    #[serde(default, rename = "proxyBypass")]
    pub proxy_bypass: String,
    #[serde(default = "default_true", rename = "checkUpdatesOnLaunch")]
    pub check_updates_on_launch: bool,
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            api_server_enabled: false,
            api_server_host: default_api_server_host(),
            api_server_port: 0,
            proxy_mode: default_proxy_mode(),
            proxy_url: String::new(),
            proxy_bypass: "localhost,127.0.0.1".to_owned(),
            check_updates_on_launch: true,
        }
    }
}

fn default_api_server_host() -> String {
    "127.0.0.1".to_owned()
}

fn default_proxy_mode() -> String {
    "system".to_owned()
}

fn default_true() -> bool {
    true
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

#[cfg(test)]
#[path = "settings/tests.rs"]
mod tests;
