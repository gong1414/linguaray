//! HTTP transport plugin. Provides `linguaray.http` when the hardened client built.

use futures::future::BoxFuture;
use linguaray_kernel::{
    ActivationContext, CapabilityPlugin, PluginDescriptor, PluginError, PluginId, ServiceId,
    ServiceKey,
};
use std::sync::Arc;

pub static HTTP: ServiceKey<reqwest::Client> = ServiceKey::new("linguaray.http");
static PROVIDES: &[ServiceId] = &[ServiceId("linguaray.http")];

pub struct HttpPlugin {
    client: Option<reqwest::Client>,
}

impl HttpPlugin {
    pub fn new(client: Option<reqwest::Client>) -> Self {
        Self { client }
    }
}

impl CapabilityPlugin for HttpPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: PluginId("http"),
            required: &[],
            optional: &[],
            provides: PROVIDES,
            manifest: None,
            restart_on_optional_change: false,
        }
    }

    fn config_fingerprint(&self) -> u64 {
        u64::from(self.client.is_some())
    }

    fn activate(&self, ctx: ActivationContext) -> BoxFuture<'_, Result<(), PluginError>> {
        Box::pin(async move {
            if let Some(client) = &self.client {
                ctx.stage_provide(HTTP, Arc::new(client.clone()))?;
            }
            Ok(())
        })
    }
}
