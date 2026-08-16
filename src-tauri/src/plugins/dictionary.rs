//! Dictionary slot Fiber. Parsers and package install live in `crate::dict`.
//! This Fiber only requires the database; commands stay in `commands/dict`.

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
