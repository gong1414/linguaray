//! In-tree Capability plugins. Production compose list lives here.

pub mod database;
pub mod http;
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
    let mut out: Vec<Arc<dyn CapabilityPlugin>> = vec![database, secrets, http];
    if let Some(shortcuts) = shortcuts {
        out.push(shortcuts);
    }
    out
}
