//! EngineDriver registry: one openai-chat Fiber, one anthropic Fiber.
//! Traditional MT lives in `traditional/` as TraditionalEngine impls (not
//! ProtocolKind map entries). No azure-openai / custom-http Driver.

pub mod anthropic;
pub mod openai_chat;
pub mod traditional;

use anthropic::AnthropicDriver;
use futures::future::BoxFuture;
use linguaray_contracts::{EngineDriver, EngineDriverRegistry, ProtocolKind};
use linguaray_kernel::{
    ActivationContext, CapabilityPlugin, PluginDescriptor, PluginError, PluginId, ServiceId,
    ServiceKey,
};
use openai_chat::OpenaiChatDriver;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

pub static DRIVERS: ServiceKey<DriverRegistry> = ServiceKey::new("linguaray.drivers");
static PROVIDES: &[ServiceId] = &[ServiceId("linguaray.drivers")];
static REQUIRED: &[ServiceId] = &[ServiceId("linguaray.http")];

pub struct DriverRegistry {
    by_protocol: parking_lot::RwLock<HashMap<ProtocolKind, Arc<dyn EngineDriver>>>,
}

impl DriverRegistry {
    pub fn new() -> Self {
        Self {
            by_protocol: parking_lot::RwLock::new(HashMap::new()),
        }
    }

    pub fn register(&self, driver: Arc<dyn EngineDriver>) {
        self.by_protocol.write().insert(driver.protocol(), driver);
    }

    pub fn ids(&self) -> Vec<&'static str> {
        let mut ids: Vec<_> = self.by_protocol.read().values().map(|d| d.id()).collect();
        ids.sort_unstable();
        ids
    }
}

impl Default for DriverRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineDriverRegistry for DriverRegistry {
    fn get(&self, protocol: ProtocolKind) -> Option<Arc<dyn EngineDriver>> {
        self.by_protocol.read().get(&protocol).cloned()
    }
}

pub fn install_builtin_drivers(registry: &DriverRegistry) {
    registry.register(Arc::new(OpenaiChatDriver));
    registry.register(Arc::new(AnthropicDriver));
    traditional::install(registry);
}

pub fn builtin_registry() -> &'static DriverRegistry {
    static REG: OnceLock<DriverRegistry> = OnceLock::new();
    REG.get_or_init(|| {
        let registry = DriverRegistry::new();
        install_builtin_drivers(&registry);
        registry
    })
}

/// Apply catalog `AuthKind` to a reqwest builder (models fetch, probes).
pub fn apply_auth(
    mut req: reqwest::RequestBuilder,
    auth: linguaray_contracts::AuthKind,
    key: &str,
) -> reqwest::RequestBuilder {
    for (name, value) in auth.http_headers(key) {
        req = req.header(name, value);
    }
    let pairs = auth.query_pairs(key);
    if !pairs.is_empty() {
        req = req.query(&pairs);
    }
    req
}

pub struct DriversPlugin {
    registry: Arc<DriverRegistry>,
}

impl DriversPlugin {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(DriverRegistry::new()),
        }
    }
}

impl Default for DriversPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityPlugin for DriversPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: PluginId("drivers"),
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
            install_builtin_drivers(&self.registry);
            ctx.stage_provide(DRIVERS, self.registry.clone())?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_registry_is_openai_chat_and_anthropic_only() {
        let ids = builtin_registry().ids();
        assert_eq!(ids, ["anthropic", "openai-chat"]);
        assert!(!ids.contains(&"azure-openai"));
        assert!(!ids.contains(&"custom-http"));
        assert!(builtin_registry()
            .get(ProtocolKind::OpenaiChat)
            .is_some_and(|d| d.id() == "openai-chat"));
        assert!(builtin_registry()
            .get(ProtocolKind::Anthropic)
            .is_some_and(|d| d.id() == "anthropic"));
    }
}
