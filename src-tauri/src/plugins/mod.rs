//! In-tree Capability plugins. Production compose list lives here.

pub mod clipboard;
pub mod database;
pub mod dictionary;
pub mod drivers;
pub mod external_api;
pub mod history;
pub mod http;
pub mod ocr;
pub mod popup;
pub mod providers;
pub mod secrets;
pub mod selection;
pub mod selection_engine;
pub mod shortcuts;
pub mod translation;
pub mod tray_state;
pub mod tts;
pub mod updater;
pub mod vocabulary;

use linguaray_kernel::CapabilityPlugin;
use std::sync::Arc;

/// Official plugins. Slot Fibers (dictionary/ocr/tts/external-api/updater)
/// are stubs: descriptor + empty activate, no commands.
pub fn builtin_plugins(
    database: Arc<database::DatabasePlugin>,
    secrets: Arc<secrets::SecretsPlugin>,
    http: Arc<http::HttpPlugin>,
    shortcuts: Option<Arc<shortcuts::ShortcutsPlugin>>,
    popup: Option<Arc<popup::PopupPlugin>>,
    tray: Option<Arc<tray_state::TrayPlugin>>,
) -> Vec<Arc<dyn CapabilityPlugin>> {
    let mut out: Vec<Arc<dyn CapabilityPlugin>> = vec![
        database,
        secrets,
        http,
        Arc::new(drivers::DriversPlugin::new()),
        Arc::new(providers::ProvidersPlugin),
        Arc::new(translation::TranslationPlugin),
        Arc::new(selection::SelectionPlugin),
        Arc::new(clipboard::ClipboardPlugin),
        Arc::new(history::HistoryPlugin),
        Arc::new(dictionary::DictionaryPlugin),
        Arc::new(ocr::OcrPlugin),
        Arc::new(tts::TtsPlugin),
        Arc::new(external_api::ExternalApiPlugin),
        Arc::new(updater::UpdaterPlugin),
    ];
    if let Some(shortcuts) = shortcuts {
        out.push(shortcuts);
    }
    if let Some(popup) = popup {
        out.push(popup);
    }
    if let Some(tray) = tray {
        out.push(tray);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_list_has_drivers_and_providers_not_vendor_fibers() {
        let plugins = builtin_plugins(
            Arc::new(database::DatabasePlugin::new(None)),
            Arc::new(secrets::SecretsPlugin::new(None)),
            Arc::new(http::HttpPlugin::new(None)),
            None,
            None,
            None,
        );
        let ids: Vec<_> = plugins.iter().map(|p| p.descriptor().id.0).collect();
        assert!(ids.contains(&"drivers"));
        assert!(ids.contains(&"providers"));
        assert!(ids.contains(&"translation"));
        assert!(ids.contains(&"selection"));
        assert!(ids.contains(&"clipboard"));
        assert!(ids.contains(&"history"));
        assert!(ids.contains(&"dictionary"));
        assert!(ids.contains(&"ocr"));
        assert!(ids.contains(&"tts"));
        assert!(ids.contains(&"external-api"));
        assert!(ids.contains(&"updater"));
        assert!(!ids.contains(&"azure-openai"));
        assert!(!ids.contains(&"custom-http"));
    }

    #[test]
    fn shipped_slot_commands_are_registered() {
        use linguaray_kernel::CapabilityPlugin;
        for p in [
            &dictionary::DictionaryPlugin as &dyn CapabilityPlugin,
            &ocr::OcrPlugin,
            &tts::TtsPlugin,
            &external_api::ExternalApiPlugin,
            &updater::UpdaterPlugin,
        ] {
            assert!(
                p.descriptor().provides.is_empty(),
                "{} Fiber must not provide a lease until a service is staged",
                p.descriptor().id.0
            );
        }
        let host = include_str!("../lib.rs");
        let handler = host
            .split("collect_commands![")
            .nth(1)
            .and_then(|s| s.split(']').next())
            .unwrap_or("");
        for cmd in [
            "ocr_capture",
            "tts_speak",
            "external_api_enable",
            "updater_check",
        ] {
            assert!(
                handler.contains(cmd),
                "shipped command {cmd} must be registered"
            );
        }
    }
}
