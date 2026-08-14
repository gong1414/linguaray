//! In-tree Capability plugins. Production compose list lives here.

pub mod clipboard;
pub mod database;
pub mod drivers;
pub mod history;
pub mod http;
pub mod popup;
pub mod providers;
pub mod secrets;
pub mod selection;
pub mod selection_engine;
pub mod shortcuts;
pub mod translation;
pub mod tray_state;

use linguaray_kernel::CapabilityPlugin;
use std::sync::Arc;

/// Official plugins. No dictionary/ocr/tts stub Fibers.
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
        assert!(!ids.contains(&"azure-openai"));
        assert!(!ids.contains(&"custom-http"));
        assert!(!ids.contains(&"ocr"));
        assert!(!ids.contains(&"dictionary"));
    }
}
