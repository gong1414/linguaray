//! Public kernel types. Zero domain: no vendor strings, no Tauri, no DB.

use futures::future::BoxFuture;
use std::fmt;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct PluginId(pub &'static str);

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ServiceId(pub &'static str);

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ActivationEpoch(pub u64);

impl fmt::Display for PluginId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl fmt::Display for ServiceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl fmt::Display for ActivationEpoch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Typed service slot. `id` is the only runtime key; `T` is bound at first
/// `stage_provide` (spec §5.3.1 D-ty).
pub struct ServiceKey<T: ?Sized + Send + Sync + 'static> {
    pub id: ServiceId,
    _ty: PhantomData<fn() -> Arc<T>>,
}

impl<T: ?Sized + Send + Sync + 'static> ServiceKey<T> {
    pub const fn new(id: &'static str) -> Self {
        Self {
            id: ServiceId(id),
            _ty: PhantomData,
        }
    }
}

impl<T: ?Sized + Send + Sync + 'static> Clone for ServiceKey<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized + Send + Sync + 'static> Copy for ServiceKey<T> {}

impl<T: ?Sized + Send + Sync + 'static> fmt::Debug for ServiceKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServiceKey").field("id", &self.id).finish()
    }
}

/// Future WASM seam. v1 official plugins leave this `None`.
#[derive(Clone, Copy, Debug)]
pub struct PluginManifest;

#[derive(Clone, Copy, Debug)]
pub struct PluginDescriptor {
    pub id: PluginId,
    pub required: &'static [ServiceId],
    pub optional: &'static [ServiceId],
    pub provides: &'static [ServiceId],
    pub manifest: Option<&'static PluginManifest>,
    /// Default `false`: optional appear/withdraw does not restart this Fiber.
    pub restart_on_optional_change: bool,
}

impl PluginDescriptor {
    pub const fn new(id: &'static str) -> Self {
        Self {
            id: PluginId(id),
            required: &[],
            optional: &[],
            provides: &[],
            manifest: None,
            restart_on_optional_change: false,
        }
    }
}

pub trait CapabilityPlugin: Send + Sync + 'static {
    fn descriptor(&self) -> PluginDescriptor;
    fn config_fingerprint(&self) -> u64;
    fn activate(
        &self,
        ctx: crate::context::ActivationContext,
    ) -> BoxFuture<'_, Result<(), PluginError>>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FiberState {
    Disabled,
    Pending,
    Starting,
    Active,
    Stopping,
    Failed,
}

#[derive(Debug, thiserror::Error)]
pub enum ComposeError {
    #[error("self-dependency on plugin {id}")]
    SelfDependency { id: PluginId },
    #[error("dependency cycle: {ids:?}")]
    DependencyCycle { ids: Vec<PluginId> },
    #[error("service {id} declared by both {a} and {b}")]
    DuplicateProvide {
        id: ServiceId,
        a: PluginId,
        b: PluginId,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("duplicate provider for {id}")]
    DuplicateProvider { id: ServiceId },
    #[error("type mismatch for {id}")]
    TypeMismatch { id: ServiceId },
    #[error("self-dependency on {id}")]
    SelfDependency { id: PluginId },
    #[error("dependency cycle involving {ids:?}")]
    DependencyCycle { ids: Vec<PluginId> },
    #[error("missing required service {id}")]
    MissingRequired { id: ServiceId },
    #[error("cancelled")]
    Cancelled,
    #[error("{0}")]
    Failed(String),
    #[error("forced stop with {leftover_leases} leftover leases")]
    ForcedStop { leftover_leases: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum LeaseError {
    #[error("service unloaded")]
    Unloaded,
    #[error("epoch mismatch")]
    EpochMismatch,
    #[error("forced stop")]
    ForcedStop,
    #[error("type mismatch")]
    TypeMismatch,
}

#[derive(Clone, Debug)]
pub struct FiberDiagnostic {
    pub id: PluginId,
    pub state: FiberState,
    pub epoch: ActivationEpoch,
    pub last_error: Option<String>,
    pub lease_count: usize,
    pub effect_count: usize,
    pub fingerprint: u64,
}

/// Drain waits `wait` for `lease_count == 0`, then cancels and waits
/// `force_grace` before `ForcedStop`. Spec: 30s + 5s.
#[derive(Clone, Copy, Debug)]
pub struct DrainConfig {
    pub wait: Duration,
    pub force_grace: Duration,
}

impl Default for DrainConfig {
    fn default() -> Self {
        Self {
            wait: Duration::from_secs(30),
            force_grace: Duration::from_secs(5),
        }
    }
}

#[derive(Clone)]
pub struct CancelToken {
    flag: Arc<AtomicBool>,
    notify: Arc<Notify>,
}

impl CancelToken {
    pub fn new() -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn cancel(&self) {
        self.flag.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
    }

    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }

    pub async fn cancelled(&self) {
        loop {
            if self.is_cancelled() {
                return;
            }
            let notified = self.notify.notified();
            if self.is_cancelled() {
                return;
            }
            notified.await;
        }
    }
}

impl Default for CancelToken {
    fn default() -> Self {
        Self::new()
    }
}
