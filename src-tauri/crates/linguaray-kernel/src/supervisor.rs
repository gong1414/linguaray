//! Compose + reconcile. One global reconcile lock; per-Fiber transition flag.

use crate::context::{ActivationContext, KernelHandle, StagedProvide};
use crate::lease::{dispose_lifo, wait_zero, InstalledEffect, Slot, SLOT_LIVE};
use crate::types::{
    ActivationEpoch, CancelToken, CapabilityPlugin, ComposeError, DrainConfig, FiberDiagnostic,
    FiberState, PluginError, PluginId, ServiceId,
};
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::timeout;

#[derive(Default)]
pub(crate) struct Staging {
    pub provides: Vec<StagedProvide>,
    pub effects: Vec<InstalledEffect>,
}

pub(crate) struct Fiber {
    plugin: Arc<dyn CapabilityPlugin>,
    pub state: FiberState,
    pub epoch: ActivationEpoch,
    pub desired_enabled: bool,
    pub desired_fp: u64,
    pub active_fp: Option<u64>,
    pub transition: bool,
    pub last_error: Option<String>,
    pub cancel: CancelToken,
    pub provided: Vec<ServiceId>,
    pub effects: Vec<InstalledEffect>,
    pub optional_seen: BTreeSet<ServiceId>,
    pub staging: Staging,
}

pub(crate) struct Inner {
    pub fibers: BTreeMap<PluginId, Fiber>,
    pub live: HashMap<ServiceId, Arc<Slot>>,
    pub type_bind: HashMap<ServiceId, std::any::TypeId>,
    pub start_order: Vec<PluginId>,
    pub shutting_down: bool,
}

impl Inner {
    pub(crate) fn diagnostics(&self) -> Vec<FiberDiagnostic> {
        self.start_order
            .iter()
            .filter_map(|id| {
                let f = self.fibers.get(id)?;
                let lease_count = f
                    .provided
                    .iter()
                    .filter_map(|sid| self.live.get(sid).map(|s| s.leases()))
                    .sum();
                Some(FiberDiagnostic {
                    id: *id,
                    state: f.state,
                    epoch: f.epoch,
                    last_error: f.last_error.clone(),
                    lease_count,
                    effect_count: f.effects.len() + f.staging.effects.len(),
                    fingerprint: f.desired_fp,
                })
            })
            .collect()
    }

    fn required_ready(&self, id: PluginId) -> bool {
        let f = &self.fibers[&id];
        f.plugin.descriptor().required.iter().all(|sid| {
            self.live
                .get(sid)
                .is_some_and(|s| s.state.load(Ordering::SeqCst) == SLOT_LIVE)
        })
    }

    fn optional_live_set(&self, id: PluginId) -> BTreeSet<ServiceId> {
        let f = &self.fibers[&id];
        f.plugin
            .descriptor()
            .optional
            .iter()
            .copied()
            .filter(|sid| {
                self.live
                    .get(sid)
                    .is_some_and(|s| s.state.load(Ordering::SeqCst) == SLOT_LIVE)
            })
            .collect()
    }

    fn provider_of(&self, sid: ServiceId) -> Option<PluginId> {
        self.live.get(&sid).and_then(|slot| {
            self.fibers.iter().find_map(|(id, f)| {
                if f.provided.contains(&sid) {
                    Some(*id)
                } else {
                    let _ = slot;
                    None
                }
            })
        })
    }
}

#[derive(Clone)]
pub struct Supervisor {
    inner: Arc<Mutex<Inner>>,
    reconcile_lock: Arc<AsyncMutex<()>>,
    drain: DrainConfig,
}

impl Supervisor {
    /// Build a supervisor. Duplicate `PluginId` **panics** (D-id). Self-deps
    /// and required-edge cycles return `Err`.
    pub fn compose(
        plugins: impl IntoIterator<Item = Arc<dyn CapabilityPlugin>>,
    ) -> Result<Self, ComposeError> {
        Self::compose_with_drain(plugins, DrainConfig::default())
    }

    pub fn compose_with_drain(
        plugins: impl IntoIterator<Item = Arc<dyn CapabilityPlugin>>,
        drain: DrainConfig,
    ) -> Result<Self, ComposeError> {
        let plugins: Vec<Arc<dyn CapabilityPlugin>> = plugins.into_iter().collect();
        let mut seen = BTreeSet::new();
        for p in &plugins {
            let id = p.descriptor().id;
            if !seen.insert(id) {
                panic!("duplicate PluginId: {id}");
            }
        }

        let mut provider_of: BTreeMap<ServiceId, PluginId> = BTreeMap::new();
        for p in &plugins {
            let d = p.descriptor();
            if d.required.iter().any(|r| d.provides.contains(r)) {
                return Err(ComposeError::SelfDependency { id: d.id });
            }
            for sid in d.provides {
                if let Some(prev) = provider_of.insert(*sid, d.id) {
                    return Err(ComposeError::DuplicateProvide {
                        id: *sid,
                        a: prev,
                        b: d.id,
                    });
                }
            }
        }

        // Kahn: edge provider → consumer. Same rank by PluginId bytes.
        let mut indeg: BTreeMap<PluginId, usize> = BTreeMap::new();
        let mut adj: BTreeMap<PluginId, Vec<PluginId>> = BTreeMap::new();
        for p in &plugins {
            indeg.insert(p.descriptor().id, 0);
            adj.insert(p.descriptor().id, Vec::new());
        }
        for p in &plugins {
            let d = p.descriptor();
            let mut deps = BTreeSet::new();
            for req in d.required {
                if let Some(prov) = provider_of.get(req) {
                    if deps.insert(*prov) {
                        adj.get_mut(prov).expect("adj").push(d.id);
                        *indeg.get_mut(&d.id).expect("indeg") += 1;
                    }
                }
            }
        }
        for succs in adj.values_mut() {
            succs.sort();
        }

        let mut ready: Vec<PluginId> = indeg
            .iter()
            .filter(|(_, n)| **n == 0)
            .map(|(id, _)| *id)
            .collect();
        ready.sort();
        let mut start_order = Vec::with_capacity(plugins.len());
        let mut queue: VecDeque<PluginId> = ready.into();
        while let Some(id) = queue.pop_front() {
            start_order.push(id);
            let succs = adj.get(&id).cloned().unwrap_or_default();
            for s in succs {
                let n = indeg.get_mut(&s).expect("indeg");
                *n -= 1;
                if *n == 0 {
                    // insert keeping PluginId order among newly-ready
                    let pos = queue.iter().position(|q| *q > s).unwrap_or(queue.len());
                    queue.insert(pos, s);
                }
            }
        }
        if start_order.len() != plugins.len() {
            let leftover: Vec<PluginId> = indeg
                .into_iter()
                .filter(|(_, n)| *n > 0)
                .map(|(id, _)| id)
                .collect();
            return Err(ComposeError::DependencyCycle { ids: leftover });
        }

        let mut fibers = BTreeMap::new();
        for p in plugins {
            let d = p.descriptor();
            fibers.insert(
                d.id,
                Fiber {
                    desired_fp: p.config_fingerprint(),
                    plugin: p,
                    state: FiberState::Disabled,
                    epoch: ActivationEpoch(0),
                    desired_enabled: false,
                    active_fp: None,
                    transition: false,
                    last_error: None,
                    cancel: CancelToken::new(),
                    provided: Vec::new(),
                    effects: Vec::new(),
                    optional_seen: BTreeSet::new(),
                    staging: Staging::default(),
                },
            );
        }

        Ok(Self {
            inner: Arc::new(Mutex::new(Inner {
                fibers,
                live: HashMap::new(),
                type_bind: HashMap::new(),
                start_order,
                shutting_down: false,
            })),
            reconcile_lock: Arc::new(AsyncMutex::new(())),
            drain,
        })
    }

    pub fn handle(&self) -> KernelHandle {
        KernelHandle {
            inner: self.inner.clone(),
        }
    }

    pub fn start_order(&self) -> Vec<PluginId> {
        self.inner.lock().expect("kernel mutex").start_order.clone()
    }

    pub fn fiber_state(&self, id: PluginId) -> Option<FiberState> {
        self.inner
            .lock()
            .expect("kernel mutex")
            .fibers
            .get(&id)
            .map(|f| f.state)
    }

    pub fn diagnostics(&self) -> Vec<FiberDiagnostic> {
        self.inner.lock().expect("kernel mutex").diagnostics()
    }

    pub fn effect_count(&self, id: PluginId) -> Option<usize> {
        self.diagnostics()
            .into_iter()
            .find(|d| d.id == id)
            .map(|d| d.effect_count)
    }

    pub async fn enable(&self, id: PluginId) {
        {
            let mut g = self.inner.lock().expect("kernel mutex");
            if let Some(f) = g.fibers.get_mut(&id) {
                f.desired_enabled = true;
            }
        }
        self.reconcile().await;
    }

    pub async fn disable(&self, id: PluginId) {
        {
            let mut g = self.inner.lock().expect("kernel mutex");
            if let Some(f) = g.fibers.get_mut(&id) {
                f.desired_enabled = false;
                f.cancel.cancel();
            }
        }
        self.reconcile().await;
    }

    pub async fn enable_all(&self) {
        {
            let mut g = self.inner.lock().expect("kernel mutex");
            for f in g.fibers.values_mut() {
                f.desired_enabled = true;
            }
        }
        self.reconcile().await;
    }

    pub async fn notify_config(&self) {
        self.reconcile().await;
    }

    pub async fn shutdown(&self) {
        {
            let mut g = self.inner.lock().expect("kernel mutex");
            g.shutting_down = true;
            for f in g.fibers.values_mut() {
                f.desired_enabled = false;
                f.cancel.cancel();
            }
        }
        self.reconcile().await;
    }

    pub async fn reconcile(&self) {
        let _guard = self.reconcile_lock.lock().await;
        loop {
            let work = self.plan();
            if work.stop.is_empty() && work.start.is_empty() {
                break;
            }
            for id in work.stop {
                self.stop_fiber(id).await;
            }
            for id in work.start {
                self.start_fiber(id).await;
            }
        }
    }

    fn plan(&self) -> Work {
        let mut g = self.inner.lock().expect("kernel mutex");
        let ids: Vec<PluginId> = g.start_order.clone();
        for id in &ids {
            let fp = g.fibers[id].plugin.config_fingerprint();
            g.fibers.get_mut(id).expect("fiber").desired_fp = fp;
        }

        let mut must_stop: BTreeSet<PluginId> = BTreeSet::new();
        for id in &ids {
            let f = &g.fibers[id];
            if f.transition {
                continue;
            }
            let dirty =
                f.state == FiberState::Active && f.active_fp.is_some_and(|fp| fp != f.desired_fp);
            let optional_restart = f.plugin.descriptor().restart_on_optional_change
                && f.state == FiberState::Active
                && g.optional_live_set(*id) != f.optional_seen;
            if !f.desired_enabled && matches!(f.state, FiberState::Active | FiberState::Pending) {
                must_stop.insert(*id);
            }
            // Failed + leftover provides still need teardown. A torn-down
            // ForcedStop Fiber must stay Failed, not be parked to Disabled.
            if !f.desired_enabled && f.state == FiberState::Failed && !f.provided.is_empty() {
                must_stop.insert(*id);
            }
            if f.desired_enabled && (dirty || optional_restart) {
                must_stop.insert(*id);
            }
        }
        // Dependents of a stopping provider must stop first (reverse topo).
        let mut grew = true;
        while grew {
            grew = false;
            for id in &ids {
                if must_stop.contains(id) {
                    continue;
                }
                let f = &g.fibers[id];
                if f.transition || f.state != FiberState::Active {
                    continue;
                }
                let lost =
                    f.plugin
                        .descriptor()
                        .required
                        .iter()
                        .any(|sid| match g.provider_of(*sid) {
                            Some(p) if must_stop.contains(&p) => true,
                            None => true,
                            Some(_) => g
                                .live
                                .get(sid)
                                .is_none_or(|s| s.state.load(Ordering::SeqCst) != SLOT_LIVE),
                        });
                if lost {
                    must_stop.insert(*id);
                    grew = true;
                }
            }
        }

        let mut stop = Vec::new();
        for id in ids.iter().rev() {
            let f = &g.fibers[id];
            if must_stop.contains(id)
                && !f.transition
                && matches!(
                    f.state,
                    FiberState::Active | FiberState::Pending | FiberState::Failed
                )
            {
                stop.push(*id);
            }
        }

        let mut start = Vec::new();
        for id in &ids {
            let f = &g.fibers[id];
            if f.transition || !f.desired_enabled {
                continue;
            }
            // Failed stays Failed until disable or a later explicit re-enable
            // (which parks it Pending). Auto-restarting Failed would loop
            // forever when activate keeps returning Err.
            if matches!(f.state, FiberState::Disabled | FiberState::Pending)
                && g.required_ready(*id)
            {
                start.push(*id);
            }
        }

        // Pending with missing deps: park as Pending.
        for id in &ids {
            let f = g.fibers.get_mut(id).expect("fiber");
            if f.desired_enabled
                && !f.transition
                && f.state == FiberState::Disabled
                && !must_stop.contains(id)
            {
                f.state = FiberState::Pending;
            }
        }

        Work { stop, start }
    }

    async fn start_fiber(&self, id: PluginId) {
        let (plugin, ctx, cancel) = {
            let mut g = self.inner.lock().expect("kernel mutex");
            let Some(f) = g.fibers.get_mut(&id) else {
                return;
            };
            if f.transition {
                return;
            }
            f.transition = true;
            f.state = FiberState::Starting;
            f.epoch = ActivationEpoch(f.epoch.0.saturating_add(1));
            f.cancel = CancelToken::new();
            f.staging = Staging::default();
            f.last_error = None;
            let plugin = f.plugin.clone();
            let ctx = ActivationContext {
                inner: self.inner.clone(),
                plugin: id,
                epoch: f.epoch,
                cancel: f.cancel.clone(),
            };
            (plugin, ctx, f.cancel.clone())
        };

        let result = tokio::select! {
            r = plugin.activate(ctx) => r,
            _ = cancel.cancelled() => Err(PluginError::Cancelled),
        };

        let rollback_effects;
        {
            let mut g = self.inner.lock().expect("kernel mutex");
            let shutting = g.shutting_down;
            let (allow_commit, staging, epoch) = {
                let f = g.fibers.get_mut(&id).expect("fiber");
                let allow =
                    result.is_ok() && !f.cancel.is_cancelled() && f.desired_enabled && !shutting;
                if allow {
                    let staging = std::mem::take(&mut f.staging);
                    let epoch = f.epoch;
                    (true, Some(staging), epoch)
                } else {
                    (false, None, f.epoch)
                }
            };
            if allow_commit {
                let staging = staging.expect("staging");
                match Self::commit(&mut g, id, epoch, staging) {
                    Err((e, effects)) => {
                        let f = g.fibers.get_mut(&id).expect("fiber");
                        rollback_effects = effects;
                        f.last_error = Some(e.to_string());
                        f.state = FiberState::Failed;
                        f.transition = false;
                        f.active_fp = None;
                    }
                    Ok(()) => {
                        let seen = g.optional_live_set(id);
                        let f = g.fibers.get_mut(&id).expect("fiber");
                        f.optional_seen = seen;
                        f.active_fp = Some(f.desired_fp);
                        f.state = FiberState::Active;
                        f.transition = false;
                        rollback_effects = Vec::new();
                    }
                }
            } else {
                let f = g.fibers.get_mut(&id).expect("fiber");
                rollback_effects = std::mem::take(&mut f.staging.effects);
                f.staging = Staging::default();
                let cancelled = matches!(result, Err(PluginError::Cancelled));
                f.last_error = result.err().map(|e| e.to_string());
                f.state = if f.desired_enabled {
                    FiberState::Failed
                } else {
                    FiberState::Disabled
                };
                if cancelled && !f.desired_enabled {
                    f.state = FiberState::Disabled;
                    f.last_error = None;
                }
                f.transition = false;
                f.active_fp = None;
            }
        }
        dispose_lifo(&mut { rollback_effects }).await;
    }

    fn commit(
        g: &mut Inner,
        id: PluginId,
        epoch: ActivationEpoch,
        staging: Staging,
    ) -> Result<(), (PluginError, Vec<InstalledEffect>)> {
        for p in &staging.provides {
            if g.live.contains_key(&p.id) {
                return Err((PluginError::DuplicateProvider { id: p.id }, staging.effects));
            }
            if let Some(ty) = g.type_bind.get(&p.id) {
                if *ty != p.type_id {
                    return Err((PluginError::TypeMismatch { id: p.id }, staging.effects));
                }
            }
        }
        let mut provided = Vec::new();
        for p in staging.provides {
            g.type_bind.insert(p.id, p.type_id);
            debug_assert_eq!(p.slot.epoch.load(Ordering::SeqCst), epoch.0);
            g.live.insert(p.id, p.slot);
            provided.push(p.id);
        }
        let f = g.fibers.get_mut(&id).expect("fiber");
        f.provided = provided;
        f.effects = staging.effects;
        Ok(())
    }

    async fn stop_fiber(&self, id: PluginId) {
        let slots;
        {
            let mut g = self.inner.lock().expect("kernel mutex");
            let Some(f) = g.fibers.get_mut(&id) else {
                return;
            };
            if f.transition {
                return;
            }
            if matches!(f.state, FiberState::Disabled) {
                return;
            }
            if matches!(f.state, FiberState::Pending | FiberState::Failed)
                && f.provided.is_empty()
                && f.effects.is_empty()
            {
                if f.state != FiberState::Failed {
                    f.state = if f.desired_enabled {
                        FiberState::Pending
                    } else {
                        FiberState::Disabled
                    };
                }
                return;
            }
            f.transition = true;
            f.state = FiberState::Stopping;
            f.cancel.cancel();
            let provided = f.provided.clone();
            slots = provided
                .iter()
                .filter_map(|sid| g.live.get(sid).cloned())
                .collect::<Vec<_>>();
            for s in &slots {
                s.mark_draining();
            }
        }

        let mut leftover = 0usize;
        let mut forced = false;
        let drain_ok = timeout(self.drain.wait, async {
            for s in &slots {
                wait_zero(s).await;
            }
        })
        .await
        .is_ok();
        if !drain_ok {
            {
                let g = self.inner.lock().expect("kernel mutex");
                if let Some(f) = g.fibers.get(&id) {
                    f.cancel.cancel();
                }
            }
            let grace_ok = timeout(self.drain.force_grace, async {
                for s in &slots {
                    wait_zero(s).await;
                }
            })
            .await
            .is_ok();
            if !grace_ok {
                forced = true;
                leftover = slots.iter().map(|s| s.leases()).sum();
                for s in &slots {
                    s.force_stop();
                }
            }
        }

        let mut effects;
        {
            let mut g = self.inner.lock().expect("kernel mutex");
            for s in &slots {
                s.mark_dead();
            }
            let provided = g
                .fibers
                .get(&id)
                .map(|f| f.provided.clone())
                .unwrap_or_default();
            for sid in &provided {
                g.live.remove(sid);
            }
            let f = g.fibers.get_mut(&id).expect("fiber");
            f.provided.clear();
            effects = std::mem::take(&mut f.effects);
            if forced {
                f.last_error = Some(
                    PluginError::ForcedStop {
                        leftover_leases: leftover,
                    }
                    .to_string(),
                );
                f.state = FiberState::Failed;
            } else {
                f.state = if f.desired_enabled {
                    FiberState::Pending
                } else {
                    FiberState::Disabled
                };
                f.last_error = None;
            }
            f.active_fp = None;
            f.optional_seen.clear();
            f.transition = false;
        }
        dispose_lifo(&mut effects).await;
    }
}

struct Work {
    stop: Vec<PluginId>,
    start: Vec<PluginId>,
}
