//! Dictionary slot Fiber. `dict.rs` stays the lookup module; R4 fills UI.
//! No commands registered from this stub.

use futures::future::BoxFuture;
use linguaray_kernel::{
    ActivationContext, CapabilityPlugin, PluginDescriptor, PluginError, PluginId, ServiceId,
};

static REQUIRED: &[ServiceId] = &[ServiceId("linguaray.database")];

pub struct DictionaryPlugin;

impl CapabilityPlugin for DictionaryPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: PluginId("dictionary"),
            required: REQUIRED,
            optional: &[],
            provides: &[],
            manifest: None,
            restart_on_optional_change: false,
        }
    }

    fn config_fingerprint(&self) -> u64 {
        1
    }

    fn activate(&self, ctx: ActivationContext) -> BoxFuture<'_, Result<(), PluginError>> {
        Box::pin(async move {
            let _ = ctx.require(crate::plugins::database::DATABASE)?;
            Ok(())
        })
    }
}
