//! Providers capability: catalog + profile→preset. CRUD stays in `db/providers`.

use crate::adapter;
use crate::db::providers::ProviderProfile;
use crate::providers::ProviderPreset;
use futures::future::BoxFuture;
use linguaray_catalog::{CatalogFile, CatalogProvider};
use linguaray_kernel::{
    ActivationContext, CapabilityPlugin, PluginDescriptor, PluginError, PluginId, ServiceId,
    ServiceKey,
};
use std::sync::Arc;

pub static PROVIDERS: ServiceKey<ProviderHub> = ServiceKey::new("linguaray.providers");
static PROVIDES: &[ServiceId] = &[ServiceId("linguaray.providers")];
static REQUIRED: &[ServiceId] = &[ServiceId("linguaray.database")];

/// Catalog + adapter façade. Secrets are leased on demand by commands, not here.
pub struct ProviderHub;

impl ProviderHub {
    pub fn catalog() -> Result<CatalogFile, linguaray_catalog::CatalogError> {
        linguaray_catalog::load()
    }

    pub fn template(id: &str) -> Option<CatalogProvider> {
        linguaray_catalog::get(id)
    }

    pub fn profile_to_preset(profile: &ProviderProfile) -> Result<ProviderPreset, String> {
        adapter::profile_to_preset(profile)
    }
}

pub struct ProvidersPlugin;

impl CapabilityPlugin for ProvidersPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: PluginId("providers"),
            required: REQUIRED,
            optional: &[],
            provides: PROVIDES,
            manifest: None,
            restart_on_optional_change: false,
        }
    }

    fn config_fingerprint(&self) -> u64 {
        1
    }

    fn activate(&self, ctx: ActivationContext) -> BoxFuture<'_, Result<(), PluginError>> {
        Box::pin(async move {
            let _db = ctx.require(crate::plugins::database::DATABASE)?;
            ctx.stage_provide(PROVIDERS, Arc::new(ProviderHub))?;
            Ok(())
        })
    }
}
