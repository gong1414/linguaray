//! Local HTTP API slot Fiber (S0 port 127.0.0.1:61742). No listener yet.

use futures::future::BoxFuture;
use linguaray_kernel::{
    ActivationContext, CapabilityPlugin, PluginDescriptor, PluginError, PluginId,
};

pub struct ExternalApiPlugin;

impl CapabilityPlugin for ExternalApiPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: PluginId("external-api"),
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
