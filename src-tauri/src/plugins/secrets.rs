//! Secrets infrastructure plugin. Provides `linguaray.secrets` when the keystore initialized.

use crate::keystore::Keystore;
use futures::future::BoxFuture;
use linguaray_kernel::{
    ActivationContext, CapabilityPlugin, PluginDescriptor, PluginError, PluginId, ServiceId,
    ServiceKey,
};
use std::sync::Arc;

pub static SECRETS: ServiceKey<Keystore> = ServiceKey::new("linguaray.secrets");
static PROVIDES: &[ServiceId] = &[ServiceId("linguaray.secrets")];

pub struct SecretsPlugin {
    keystore: Option<Arc<Keystore>>,
}

impl SecretsPlugin {
    pub fn new(keystore: Option<Arc<Keystore>>) -> Self {
        Self { keystore }
    }
}

impl CapabilityPlugin for SecretsPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: PluginId("keystore"),
            required: &[],
            optional: &[],
            provides: PROVIDES,
            manifest: None,
            restart_on_optional_change: false,
        }
    }

    fn config_fingerprint(&self) -> u64 {
        u64::from(self.keystore.is_some())
    }

    fn activate(&self, ctx: ActivationContext) -> BoxFuture<'_, Result<(), PluginError>> {
        Box::pin(async move {
            if let Some(ks) = &self.keystore {
                ctx.stage_provide(SECRETS, ks.clone())?;
            }
            Ok(())
        })
    }
}
