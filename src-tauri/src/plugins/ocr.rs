//! OCR slot Fiber. No capture implementation and no commands yet.

use futures::future::BoxFuture;
use linguaray_kernel::{
    ActivationContext, CapabilityPlugin, PluginDescriptor, PluginError, PluginId,
};

pub struct OcrPlugin;

impl CapabilityPlugin for OcrPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: PluginId("ocr"),
            required: &[],
            optional: &[],
            provides: &[],
            manifest: None,
            restart_on_optional_change: false,
        }
    }

    fn config_fingerprint(&self) -> u64 {
        1
    }

    fn activate(&self, _ctx: ActivationContext) -> BoxFuture<'_, Result<(), PluginError>> {
        Box::pin(async move { Ok(()) })
    }
}
