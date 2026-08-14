//! Activation-time API: staging provide, require/optional, install_effect.

use crate::lease::{EffectDisposer, InstalledEffect, ServiceLease, Slot};
use crate::supervisor::Inner;
use crate::types::{
    ActivationEpoch, CancelToken, LeaseError, PluginError, PluginId, ServiceId, ServiceKey,
};
use std::any::TypeId;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

/// Visible only to the activating Fiber. Staging is invisible to dependents
/// until `activate` returns `Ok` and the supervisor commits.
#[derive(Clone)]
pub struct ActivationContext {
    pub(crate) inner: Arc<Mutex<Inner>>,
    pub(crate) plugin: PluginId,
    pub(crate) epoch: ActivationEpoch,
    pub(crate) cancel: CancelToken,
}

impl ActivationContext {
    pub fn epoch(&self) -> ActivationEpoch {
        self.epoch
    }

    pub fn cancellation(&self) -> CancelToken {
        self.cancel.clone()
    }

    pub fn handle(&self) -> KernelHandle {
        KernelHandle {
            inner: self.inner.clone(),
        }
    }

    /// Bind `value` into this Fiber's staging area. Not live until commit.
    pub fn stage_provide<T: ?Sized + Send + Sync + 'static>(
        &self,
        key: ServiceKey<T>,
        value: Arc<T>,
    ) -> Result<(), PluginError> {
        if self.cancel.is_cancelled() {
            return Err(PluginError::Cancelled);
        }
        let held_ty = TypeId::of::<crate::lease::Held<T>>();
        let mut g = self.inner.lock().expect("kernel mutex");
        if let Some(ty) = g.type_bind.get(&key.id) {
            if *ty != held_ty {
                return Err(PluginError::TypeMismatch { id: key.id });
            }
        }
        if g.live.contains_key(&key.id) {
            return Err(PluginError::DuplicateProvider { id: key.id });
        }
        let fiber = g.fibers.get_mut(&self.plugin).expect("activating fiber");
        if fiber.staging.provides.iter().any(|p| p.id == key.id) {
            return Err(PluginError::DuplicateProvider { id: key.id });
        }
        fiber.staging.provides.push(StagedProvide {
            id: key.id,
            type_id: held_ty,
            slot: Slot::new(key.id, self.epoch, value),
        });
        Ok(())
    }

    pub fn require<T: ?Sized + Send + Sync + 'static>(
        &self,
        key: ServiceKey<T>,
    ) -> Result<ServiceLease<T>, PluginError> {
        if self.cancel.is_cancelled() {
            return Err(PluginError::Cancelled);
        }
        let g = self.inner.lock().expect("kernel mutex");
        match g.live.get(&key.id) {
            Some(slot) => {
                ServiceLease::issue(slot).map_err(|_| PluginError::MissingRequired { id: key.id })
            }
            None => Err(PluginError::MissingRequired { id: key.id }),
        }
    }

    pub fn optional<T: ?Sized + Send + Sync + 'static>(
        &self,
        key: ServiceKey<T>,
    ) -> Option<ServiceLease<T>> {
        let g = self.inner.lock().expect("kernel mutex");
        g.live
            .get(&key.id)
            .and_then(|slot| ServiceLease::issue(slot).ok())
    }

    /// Run `setup`. On success, register the disposer immediately (still staging).
    /// On failure, do not register.
    pub async fn install_effect<F, Fut>(
        &self,
        name: &'static str,
        setup: F,
    ) -> Result<(), PluginError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<EffectDisposer, PluginError>>,
    {
        if self.cancel.is_cancelled() {
            return Err(PluginError::Cancelled);
        }
        let disposer = setup().await?;
        if self.cancel.is_cancelled() {
            InstalledEffect::new(name, disposer).run().await;
            return Err(PluginError::Cancelled);
        }
        let mut g = self.inner.lock().expect("kernel mutex");
        let fiber = g.fibers.get_mut(&self.plugin).expect("activating fiber");
        fiber
            .staging
            .effects
            .push(InstalledEffect::new(name, disposer));
        Ok(())
    }
}

/// Post-activation lookup. New leases fail once a slot is `Draining`.
#[derive(Clone)]
pub struct KernelHandle {
    pub(crate) inner: Arc<Mutex<Inner>>,
}

impl KernelHandle {
    pub fn lease<T: ?Sized + Send + Sync + 'static>(
        &self,
        key: ServiceKey<T>,
    ) -> Result<ServiceLease<T>, LeaseError> {
        let g = self.inner.lock().expect("kernel mutex");
        match g.live.get(&key.id) {
            Some(slot) => ServiceLease::issue(slot),
            None => Err(LeaseError::Unloaded),
        }
    }

    pub fn optional<T: ?Sized + Send + Sync + 'static>(
        &self,
        key: ServiceKey<T>,
    ) -> Option<ServiceLease<T>> {
        self.lease(key).ok()
    }

    pub fn is_live(&self, id: ServiceId) -> bool {
        let g = self.inner.lock().expect("kernel mutex");
        g.live
            .get(&id)
            .is_some_and(|s| s.state.load(Ordering::SeqCst) == crate::lease::SLOT_LIVE)
    }

    pub fn lease_count(&self, id: ServiceId) -> Option<usize> {
        let g = self.inner.lock().expect("kernel mutex");
        g.live.get(&id).map(|s| s.leases())
    }

    pub fn diagnostics(&self) -> Vec<crate::types::FiberDiagnostic> {
        let g = self.inner.lock().expect("kernel mutex");
        g.diagnostics()
    }
}

pub(crate) struct StagedProvide {
    pub id: ServiceId,
    pub type_id: TypeId,
    pub slot: Arc<Slot>,
}
