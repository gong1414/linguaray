//! In-tree Capability plugins. Production compose list lives here.

pub mod database;
pub mod drivers;
pub mod http;
pub mod providers;
pub mod secrets;
pub mod shortcuts;

use linguaray_kernel::CapabilityPlugin;
use std::sync::Arc;

/// Official plugins. No dictionary/ocr/tts stub Fibers.
pub fn builtin_plugins(
    database: Arc<database::DatabasePlugin>,
    secrets: Arc<secrets::SecretsPlugin>,
    http: Arc<http::HttpPlugin>,
    shortcuts: Option<Arc<shortcuts::ShortcutsPlugin>>,
) -> Vec<Arc<dyn CapabilityPlugin>> {
    let mut out: Vec<Arc<dyn CapabilityPlugin>> = vec![
        database,
        secrets,
        http,
        Arc::new(drivers::DriversPlugin::new()),
        Arc::new(providers::ProvidersPlugin),
    ];
    if let Some(shortcuts) = shortcuts {
        out.push(shortcuts);
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
        );
        let ids: Vec<_> = plugins.iter().map(|p| p.descriptor().id.0).collect();
        assert!(ids.contains(&"drivers"));
        assert!(ids.contains(&"providers"));
        assert!(!ids.contains(&"azure-openai"));
        assert!(!ids.contains(&"custom-http"));
        assert!(!ids.contains(&"ocr"));
        assert!(!ids.contains(&"dictionary"));
    }
}
