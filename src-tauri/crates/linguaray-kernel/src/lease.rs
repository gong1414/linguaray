//! `ServiceLease<T>`: hold a counted epoch lease; never expose `Arc<T>`.

use crate::types::{ActivationEpoch, LeaseError, ServiceId};
use futures::future::BoxFuture;
use std::any::{Any, TypeId};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;

pub(crate) const SLOT_LIVE: u8 = 0;
pub(crate) const SLOT_DRAINING: u8 = 1;
pub(crate) const SLOT_DEAD: u8 = 2;

/// Sized holder so `Arc<T>` can be stored in `dyn Any` even when `T: ?Sized`.
pub(crate) struct Held<T: ?Sized + Send + Sync + 'static> {
    pub value: Arc<T>,
}

pub(crate) struct Slot {
    pub service_id: ServiceId,
    #[allow(dead_code)]
    pub type_id: TypeId,
    pub epoch: AtomicU64,
    pub state: AtomicU8,
    pub lease_count: AtomicUsize,
    pub forced: AtomicBool,
    pub zero: Notify,
    pub force_notify: Notify,
    pub held: Arc<dyn Any + Send + Sync>,
}

impl Slot {
    pub(crate) fn new<T: ?Sized + Send + Sync + 'static>(
        service_id: ServiceId,
        epoch: ActivationEpoch,
        value: Arc<T>,
    ) -> Arc<Self> {
        let type_id = TypeId::of::<Held<T>>();
        Arc::new(Self {
            service_id,
            type_id,
            epoch: AtomicU64::new(epoch.0),
            state: AtomicU8::new(SLOT_LIVE),
            lease_count: AtomicUsize::new(0),
            forced: AtomicBool::new(false),
            zero: Notify::new(),
            force_notify: Notify::new(),
            held: Arc::new(Held { value }),
        })
    }

    pub(crate) fn mark_draining(&self) {
        self.state.store(SLOT_DRAINING, Ordering::SeqCst);
    }

    pub(crate) fn mark_dead(&self) {
        self.state.store(SLOT_DEAD, Ordering::SeqCst);
        self.epoch.fetch_add(1, Ordering::SeqCst);
        self.zero.notify_waiters();
    }

    pub(crate) fn force_stop(&self) {
        self.forced.store(true, Ordering::SeqCst);
        self.force_notify.notify_waiters();
        self.zero.notify_waiters();
    }

    pub(crate) fn leases(&self) -> usize {
        self.lease_count.load(Ordering::SeqCst)
    }
}

/// Lease on a provided service. `Clone` increments the supervisor count;
/// there is no API that yields the inner `Arc`.
pub struct ServiceLease<T: ?Sized + Send + Sync + 'static> {
    value: Arc<T>,
    epoch: ActivationEpoch,
    slot: Arc<Slot>,
}

impl<T: ?Sized + Send + Sync + 'static> ServiceLease<T> {
    pub(crate) fn issue(slot: &Arc<Slot>) -> Result<Self, LeaseError> {
        if slot.forced.load(Ordering::SeqCst) {
            return Err(LeaseError::ForcedStop);
        }
        if slot.state.load(Ordering::SeqCst) != SLOT_LIVE {
            return Err(LeaseError::Unloaded);
        }
        let held = slot
            .held
            .downcast_ref::<Held<T>>()
            .ok_or(LeaseError::TypeMismatch)?;
        slot.lease_count.fetch_add(1, Ordering::SeqCst);
        Ok(Self {
            value: held.value.clone(),
            epoch: ActivationEpoch(slot.epoch.load(Ordering::SeqCst)),
            slot: slot.clone(),
        })
    }

    pub fn epoch(&self) -> ActivationEpoch {
        self.epoch
    }

    pub fn is_live(&self) -> bool {
        !self.slot.forced.load(Ordering::SeqCst)
            && self.slot.state.load(Ordering::SeqCst) == SLOT_LIVE
            && self.slot.epoch.load(Ordering::SeqCst) == self.epoch.0
    }

    fn classify(&self) -> Result<(), LeaseError> {
        if self.slot.forced.load(Ordering::SeqCst) {
            return Err(LeaseError::ForcedStop);
        }
        if self.slot.epoch.load(Ordering::SeqCst) != self.epoch.0 {
            return Err(LeaseError::EpochMismatch);
        }
        match self.slot.state.load(Ordering::SeqCst) {
            SLOT_LIVE | SLOT_DRAINING => Ok(()),
            _ => Err(LeaseError::Unloaded),
        }
    }

    /// Run `f` for the lifetime of the future. `f` receives `&T` only.
    ///
    /// The future must not capture `&T` (the bound is not higher-ranked).
    /// Use [`scope`] when the business future needs the borrowed service.
    pub async fn call<F, Fut, R>(&self, f: F) -> Result<R, LeaseError>
    where
        F: FnOnce(&T) -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        self.classify()?;
        let fut = f(&self.value);
        tokio::pin!(fut);
        tokio::select! {
            biased;
            _ = self.slot.force_notify.notified() => Err(LeaseError::ForcedStop),
            r = &mut fut => {
                self.classify()?;
                Ok(r)
            }
        }
    }

    /// Like [`call`], but the future may borrow `&T` for its whole lifetime.
    pub async fn scope<F, R>(&self, f: F) -> Result<R, LeaseError>
    where
        F: for<'a> FnOnce(&'a T) -> BoxFuture<'a, R>,
    {
        self.classify()?;
        let mut fut = f(&self.value);
        tokio::select! {
            biased;
            _ = self.slot.force_notify.notified() => Err(LeaseError::ForcedStop),
            r = &mut fut => {
                self.classify()?;
                Ok(r)
            }
        }
    }

    pub fn service_id(&self) -> ServiceId {
        self.slot.service_id
    }
}

impl<T: ?Sized + Send + Sync + 'static> Clone for ServiceLease<T> {
    fn clone(&self) -> Self {
        self.slot.lease_count.fetch_add(1, Ordering::SeqCst);
        Self {
            value: self.value.clone(),
            epoch: self.epoch,
            slot: self.slot.clone(),
        }
    }
}

impl<T: ?Sized + Send + Sync + 'static> Drop for ServiceLease<T> {
    fn drop(&mut self) {
        if self.slot.lease_count.fetch_sub(1, Ordering::SeqCst) == 1 {
            self.slot.zero.notify_waiters();
        }
    }
}

pub(crate) async fn wait_zero(slot: &Slot) {
    loop {
        if slot.leases() == 0 {
            return;
        }
        let notified = slot.zero.notified();
        if slot.leases() == 0 {
            return;
        }
        notified.await;
    }
}

/// Sync or async disposer. Setup failure must not register this.
pub struct EffectDisposer {
    inner: Box<dyn FnOnce() -> BoxFuture<'static, ()> + Send>,
}

impl EffectDisposer {
    pub fn from_fn(f: impl FnOnce() + Send + 'static) -> Self {
        Self {
            inner: Box::new(move || {
                f();
                Box::pin(async {})
            }),
        }
    }

    pub fn from_async<Fut>(f: impl FnOnce() -> Fut + Send + 'static) -> Self
    where
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        Self {
            inner: Box::new(move || Box::pin(f())),
        }
    }
}

pub(crate) struct InstalledEffect {
    #[allow(dead_code)]
    pub name: &'static str,
    disposer: Option<EffectDisposer>,
}

impl InstalledEffect {
    pub(crate) fn new(name: &'static str, disposer: EffectDisposer) -> Self {
        Self {
            name,
            disposer: Some(disposer),
        }
    }

    pub(crate) async fn run(mut self) {
        if let Some(d) = self.disposer.take() {
            // Panic in a sync disposer must not skip the rest (spec §5.4).
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(d.inner)) {
                Ok(fut) => fut.await,
                Err(_) => {}
            }
        }
    }
}

pub(crate) async fn dispose_lifo(effects: &mut Vec<InstalledEffect>) {
    while let Some(effect) = effects.pop() {
        effect.run().await;
    }
}
