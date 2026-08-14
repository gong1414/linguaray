//! Database infrastructure plugin. Provides `linguaray.database` when a handle exists.

use crate::db::Database;
use futures::future::BoxFuture;
use linguaray_kernel::{
    ActivationContext, CapabilityPlugin, PluginDescriptor, PluginError, PluginId, ServiceId,
    ServiceKey,
};
use std::sync::Arc;

pub static DATABASE: ServiceKey<Database> = ServiceKey::new("linguaray.database");
static PROVIDES: &[ServiceId] = &[ServiceId("linguaray.database")];

pub struct DatabasePlugin {
    db: Option<Arc<Database>>,
}

impl DatabasePlugin {
    pub fn new(db: Option<Arc<Database>>) -> Self {
        Self { db }
    }
}

impl CapabilityPlugin for DatabasePlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: PluginId("database"),
            required: &[],
            optional: &[],
            provides: PROVIDES,
            manifest: None,
            restart_on_optional_change: false,
        }
    }

    fn config_fingerprint(&self) -> u64 {
        u64::from(self.db.is_some())
    }

    fn activate(&self, ctx: ActivationContext) -> BoxFuture<'_, Result<(), PluginError>> {
        Box::pin(async move {
            if let Some(db) = &self.db {
                ctx.stage_provide(DATABASE, db.clone())?;
            }
            Ok(())
        })
    }
}
