//! In-tree Capability plugins. Production compose list lives here.

pub mod shortcuts;

use linguaray_kernel::CapabilityPlugin;
use std::sync::Arc;

/// Official plugins. No dictionary/ocr/tts stub Fibers.
pub fn builtin_plugins(
    shortcuts: Arc<shortcuts::ShortcutsPlugin>,
) -> Vec<Arc<dyn CapabilityPlugin>> {
    vec![shortcuts]
}
