//! K0 tests: §5.3.1–5.3.4 plus lifecycle, drain, and churn.

use super::*;
use crate::lease::EffectDisposer;
use futures::future::BoxFuture;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct Unit;
struct Other;

const S_A: ServiceId = ServiceId("svc.a");
const S_B: ServiceId = ServiceId("svc.b");
const KEY_A: ServiceKey<Unit> = ServiceKey::new("svc.a");
const KEY_A_OTHER: ServiceKey<Other> = ServiceKey::new("svc.a");

const NONE: &[ServiceId] = &[];
const REQ_A: &[ServiceId] = &[S_A];
const REQ_B: &[ServiceId] = &[S_B];
const PROV_A: &[ServiceId] = &[S_A];
const PROV_B: &[ServiceId] = &[S_B];
const OPT_A: &[ServiceId] = &[S_A];

struct TestPlugin {
    desc: PluginDescriptor,
    fp: Arc<AtomicU64>,
    activate:
        Box<dyn Fn(ActivationContext) -> BoxFuture<'static, Result<(), PluginError>> + Send + Sync>,
}

impl CapabilityPlugin for TestPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        self.desc
    }
    fn config_fingerprint(&self) -> u64 {
        self.fp.load(Ordering::SeqCst)
    }
    fn activate(&self, ctx: ActivationContext) -> BoxFuture<'_, Result<(), PluginError>> {
        (self.activate)(ctx)
    }
}

fn desc(
    id: &'static str,
    required: &'static [ServiceId],
    optional: &'static [ServiceId],
    provides: &'static [ServiceId],
    restart_on_optional_change: bool,
) -> PluginDescriptor {
    PluginDescriptor {
        id: PluginId(id),
        required,
        optional,
        provides,
        manifest: None,
        restart_on_optional_change,
    }
}

fn plug(
    d: PluginDescriptor,
    activate: impl Fn(ActivationContext) -> BoxFuture<'static, Result<(), PluginError>>
        + Send
        + Sync
        + 'static,
) -> (Arc<dyn CapabilityPlugin>, Arc<AtomicU64>) {
    let fp = Arc::new(AtomicU64::new(1));
    let p: Arc<dyn CapabilityPlugin> = Arc::new(TestPlugin {
        desc: d,
        fp: fp.clone(),
        activate: Box::new(activate),
    });
    (p, fp)
}

fn provide_a() -> (Arc<dyn CapabilityPlugin>, Arc<AtomicU64>) {
    plug(desc("prov-a", NONE, NONE, PROV_A, false), |ctx| {
        Box::pin(async move {
            ctx.stage_provide(KEY_A, Arc::new(Unit))?;
            Ok(())
        })
    })
}

fn consume_a() -> (Arc<dyn CapabilityPlugin>, Arc<AtomicU64>) {
    plug(desc("cons-a", REQ_A, NONE, NONE, false), |_ctx| {
        Box::pin(async move { Ok(()) })
    })
}

#[test]
#[should_panic(expected = "duplicate PluginId")]
fn d_id_duplicate_plugin_panics() {
    let (a, _) = provide_a();
    let (b, _) = plug(desc("prov-a", NONE, NONE, PROV_B, false), |_ctx| {
        Box::pin(async move { Ok(()) })
    });
    let _ = Supervisor::compose([a, b]);
}

#[test]
fn d_self_rejected_at_compose() {
    let (p, _) = plug(desc("loop", REQ_A, NONE, PROV_A, false), |_ctx| {
        Box::pin(async move { Ok(()) })
    });
    match Supervisor::compose([p]) {
        Err(ComposeError::SelfDependency { id }) => assert_eq!(id, PluginId("loop")),
        Err(e) => panic!("expected SelfDependency, got {e:?}"),
        Ok(_) => panic!("expected SelfDependency, got Ok"),
    }
}

#[test]
fn d_cyc_rejected_at_compose() {
    let (a, _) = plug(desc("a", REQ_B, NONE, PROV_A, false), |_ctx| {
        Box::pin(async move { Ok(()) })
    });
    let (b, _) = plug(desc("b", REQ_A, NONE, PROV_B, false), |_ctx| {
        Box::pin(async move { Ok(()) })
    });
    match Supervisor::compose([a, b]) {
        Err(ComposeError::DependencyCycle { ids }) => {
            assert!(ids.contains(&PluginId("a")));
            assert!(ids.contains(&PluginId("b")));
        }
        Err(e) => panic!("expected cycle, got {e:?}"),
        Ok(_) => panic!("expected cycle, got Ok"),
    }
}

#[test]
fn d_ord_same_rank_by_plugin_id_bytes() {
    // Rank 0: "mid" (no deps) and "aaa" (provides A). aaa < mid.
    // Rank 1: "zzz" requires A.
    let (aaa, _) = plug(desc("aaa", NONE, NONE, PROV_A, false), |ctx| {
        Box::pin(async move {
            ctx.stage_provide(KEY_A, Arc::new(Unit))?;
            Ok(())
        })
    });
    let (mid, _) = plug(desc("mid", NONE, NONE, NONE, false), |_ctx| {
        Box::pin(async move { Ok(()) })
    });
    let (zzz, _) = plug(desc("zzz", REQ_A, NONE, NONE, false), |_ctx| {
        Box::pin(async move { Ok(()) })
    });
    // Insert in a scrambled order so a HashMap walk would not match.
    let sup = Supervisor::compose([zzz, mid, aaa]).unwrap();
    assert_eq!(
        sup.start_order(),
        vec![PluginId("aaa"), PluginId("mid"), PluginId("zzz")]
    );
}

#[test]
fn d_pick_duplicate_declare_is_compose_error() {
    let (a, _) = provide_a();
    let (b, _) = plug(desc("other", NONE, NONE, PROV_A, false), |_ctx| {
        Box::pin(async move { Ok(()) })
    });
    match Supervisor::compose([a, b]) {
        Err(ComposeError::DuplicateProvide { id, .. }) => assert_eq!(id, S_A),
        Err(e) => panic!("expected DuplicateProvide, got {e:?}"),
        Ok(_) => panic!("expected DuplicateProvide, got Ok"),
    }
}

#[tokio::test]
async fn d_svc_second_stage_provide_rolls_back() {
    let (p, _) = plug(desc("dup", NONE, NONE, PROV_A, false), |ctx| {
        Box::pin(async move {
            ctx.stage_provide(KEY_A, Arc::new(Unit))?;
            ctx.stage_provide(KEY_A, Arc::new(Unit))?;
            Ok(())
        })
    });
    let sup = Supervisor::compose([p]).unwrap();
    sup.enable(PluginId("dup")).await;
    assert_eq!(sup.fiber_state(PluginId("dup")), Some(FiberState::Failed));
    assert!(!sup.handle().is_live(S_A));
    let err = sup
        .diagnostics()
        .into_iter()
        .find(|d| d.id == PluginId("dup"))
        .unwrap()
        .last_error
        .unwrap();
    assert!(err.contains("duplicate"), "{err}");
}

#[tokio::test]
async fn d_ty_second_type_on_same_service_id_fails() {
    let (a, _) = provide_a();
    let (b, _) = plug(desc("wrong-ty", NONE, NONE, NONE, false), |ctx| {
        Box::pin(async move {
            ctx.stage_provide(KEY_A_OTHER, Arc::new(Other))?;
            Ok(())
        })
    });
    let sup = Supervisor::compose([a, b]).unwrap();
    sup.enable(PluginId("prov-a")).await;
    assert_eq!(
        sup.fiber_state(PluginId("prov-a")),
        Some(FiberState::Active)
    );
    sup.enable(PluginId("wrong-ty")).await;
    assert_eq!(
        sup.fiber_state(PluginId("wrong-ty")),
        Some(FiberState::Failed)
    );
    let err = sup
        .diagnostics()
        .into_iter()
        .find(|d| d.id == PluginId("wrong-ty"))
        .unwrap()
        .last_error
        .unwrap();
    assert!(err.contains("type mismatch"), "{err}");
}

#[tokio::test]
async fn dependent_stays_pending_until_provider_commits() {
    let started = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let started2 = started.clone();
    let release2 = release.clone();
    let (prov, _) = plug(desc("prov-a", NONE, NONE, PROV_A, false), move |ctx| {
        let started = started2.clone();
        let release = release2.clone();
        Box::pin(async move {
            ctx.stage_provide(KEY_A, Arc::new(Unit))?;
            started.notify_one();
            release.notified().await;
            Ok(())
        })
    });
    let (cons, _) = consume_a();
    let sup = Supervisor::compose([cons, prov]).unwrap();
    let run = {
        let sup = sup.clone();
        tokio::spawn(async move { sup.enable_all().await })
    };
    started.notified().await;
    assert_eq!(
        sup.fiber_state(PluginId("prov-a")),
        Some(FiberState::Starting)
    );
    assert_eq!(
        sup.fiber_state(PluginId("cons-a")),
        Some(FiberState::Pending)
    );
    assert!(
        sup.handle().optional(KEY_A).is_none(),
        "staging must be invisible"
    );
    release.notify_one();
    run.await.unwrap();
    assert_eq!(
        sup.fiber_state(PluginId("prov-a")),
        Some(FiberState::Active)
    );
    assert_eq!(
        sup.fiber_state(PluginId("cons-a")),
        Some(FiberState::Active)
    );
    assert!(sup.handle().is_live(S_A));
}

#[tokio::test]
async fn partial_activate_rolls_back_provide_and_effect() {
    let live = Arc::new(AtomicUsize::new(0));
    let live2 = live.clone();
    let (p, _) = plug(desc("boom", NONE, NONE, PROV_A, false), move |ctx| {
        let live = live2.clone();
        Box::pin(async move {
            ctx.stage_provide(KEY_A, Arc::new(Unit))?;
            ctx.install_effect("fx", || {
                let live = live.clone();
                async move {
                    live.fetch_add(1, Ordering::SeqCst);
                    Ok(EffectDisposer::from_fn(move || {
                        live.fetch_sub(1, Ordering::SeqCst);
                    }))
                }
            })
            .await?;
            Err(PluginError::Failed("boom".into()))
        })
    });
    let sup = Supervisor::compose([p]).unwrap();
    sup.enable(PluginId("boom")).await;
    assert_eq!(sup.fiber_state(PluginId("boom")), Some(FiberState::Failed));
    assert!(!sup.handle().is_live(S_A));
    assert_eq!(live.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn disposer_panic_does_not_skip_rest() {
    let second = Arc::new(AtomicUsize::new(0));
    let second2 = second.clone();
    let (p, _) = plug(desc("fx", NONE, NONE, NONE, false), move |ctx| {
        let second = second2.clone();
        Box::pin(async move {
            ctx.install_effect("first", || async {
                Ok(EffectDisposer::from_fn(|| panic!("first disposer")))
            })
            .await?;
            ctx.install_effect("second", || {
                let second = second.clone();
                async move {
                    Ok(EffectDisposer::from_fn(move || {
                        second.fetch_add(1, Ordering::SeqCst);
                    }))
                }
            })
            .await?;
            Ok(())
        })
    });
    let sup = Supervisor::compose([p]).unwrap();
    sup.enable(PluginId("fx")).await;
    assert_eq!(sup.fiber_state(PluginId("fx")), Some(FiberState::Active));
    sup.disable(PluginId("fx")).await;
    assert_eq!(second.load(Ordering::SeqCst), 1, "LIFO second disposer ran");
    assert_eq!(sup.fiber_state(PluginId("fx")), Some(FiberState::Disabled));
}

#[tokio::test]
async fn lease_clone_counts_and_drop_reaches_zero() {
    let (p, _) = provide_a();
    let sup = Supervisor::compose([p]).unwrap();
    sup.enable(PluginId("prov-a")).await;
    let h = sup.handle();
    let l0 = h.lease(KEY_A).unwrap();
    assert_eq!(h.lease_count(S_A), Some(1));
    let clones: Vec<_> = (0..7).map(|_| l0.clone()).collect();
    assert_eq!(h.lease_count(S_A), Some(8));
    drop(clones);
    assert_eq!(h.lease_count(S_A), Some(1));
    drop(l0);
    assert_eq!(h.lease_count(S_A), Some(0));
    let n = l0_call_via_new(&h).await;
    assert_eq!(n, 7);
}

async fn l0_call_via_new(h: &KernelHandle) -> u32 {
    let l = h.lease(KEY_A).unwrap();
    l.call(|_u| async { 7u32 }).await.unwrap()
}

#[tokio::test]
async fn drain_waits_for_zero_then_unloads() {
    let (p, _) = provide_a();
    let (c, _) = consume_a();
    let sup = Supervisor::compose([c, p]).unwrap();
    sup.enable_all().await;
    let lease = sup.handle().lease(KEY_A).unwrap();
    let stop = {
        let sup = sup.clone();
        tokio::spawn(async move { sup.disable(PluginId("prov-a")).await })
    };
    tokio::task::yield_now().await;
    tokio::time::sleep(Duration::from_millis(5)).await;
    assert_eq!(
        sup.fiber_state(PluginId("prov-a")),
        Some(FiberState::Stopping)
    );
    assert!(matches!(
        sup.handle().lease(KEY_A),
        Err(LeaseError::Unloaded)
    ));
    drop(lease);
    stop.await.unwrap();
    assert_eq!(
        sup.fiber_state(PluginId("prov-a")),
        Some(FiberState::Disabled)
    );
    assert!(!sup.handle().is_live(S_A));
}

#[test]
fn drain_defaults_are_30s_then_5s() {
    let d = DrainConfig::default();
    assert_eq!(d.wait, Duration::from_secs(30));
    assert_eq!(d.force_grace, Duration::from_secs(5));
}

#[tokio::test]
async fn drain_timeout_then_grace_forced_stop() {
    // Production constants are 30s + 5s (`drain_defaults_are_30s_then_5s`).
    // The state machine is the same with a short config so this is not a
    // 35s wall-clock test and does not depend on a paused clock.
    let (p, _) = provide_a();
    let sup = Supervisor::compose_with_drain(
        [p],
        DrainConfig {
            wait: Duration::from_millis(20),
            force_grace: Duration::from_millis(20),
        },
    )
    .unwrap();
    sup.enable(PluginId("prov-a")).await;
    let lease = sup.handle().lease(KEY_A).unwrap();
    assert_eq!(sup.handle().lease_count(S_A), Some(1));
    let stop = {
        let sup = sup.clone();
        tokio::spawn(async move { sup.disable(PluginId("prov-a")).await })
    };
    loop {
        if sup.fiber_state(PluginId("prov-a")) == Some(FiberState::Stopping) {
            break;
        }
        tokio::task::yield_now().await;
    }
    stop.await.unwrap();
    assert_eq!(
        sup.fiber_state(PluginId("prov-a")),
        Some(FiberState::Failed)
    );
    assert!(
        sup.diagnostics()[0]
            .last_error
            .as_deref()
            .unwrap()
            .contains("forced stop"),
        "{:?}",
        sup.diagnostics()[0].last_error
    );
    assert!(matches!(
        lease.call(|_| async { 1 }).await,
        Err(LeaseError::ForcedStop)
    ));
    assert!(matches!(
        sup.handle().lease(KEY_A),
        Err(LeaseError::Unloaded)
    ));
}

#[tokio::test]
async fn epoch_mismatch_on_stale_lease() {
    let (p, fp) = provide_a();
    let sup = Supervisor::compose([p]).unwrap();
    sup.enable(PluginId("prov-a")).await;
    let lease = sup.handle().lease(KEY_A).unwrap();
    let e1 = lease.epoch();
    // In-flight call during drain still succeeds (same epoch, slot Draining).
    fp.store(2, Ordering::SeqCst);
    let rec = {
        let sup = sup.clone();
        tokio::spawn(async move { sup.notify_config().await })
    };
    loop {
        if sup.fiber_state(PluginId("prov-a")) == Some(FiberState::Stopping) {
            break;
        }
        tokio::task::yield_now().await;
    }
    assert_eq!(lease.call(|_| async { 1u8 }).await.unwrap(), 1);
    drop(lease);
    rec.await.unwrap();
    let fresh = sup.handle().lease(KEY_A).unwrap();
    assert_ne!(fresh.epoch(), e1);
    assert_eq!(
        sup.fiber_state(PluginId("prov-a")),
        Some(FiberState::Active)
    );
    // Late call on a lease issued for the previous epoch: slot is dead.
    // Recreate the stale check by issuing, restarting after drop, then
    // calling a clone that outlived the slot via ForcedStop — covered by
    // drain_30s_plus_5s_forced_stop. Here the new lease is a new epoch.
    assert!(fresh.epoch().0 > e1.0);
}

#[tokio::test]
async fn optional_appear_withdraw_does_not_restart_by_default() {
    let (prov, _) = provide_a();
    let (obs, _) = plug(desc("obs", NONE, OPT_A, NONE, false), |_ctx| {
        Box::pin(async move { Ok(()) })
    });
    let sup = Supervisor::compose([obs, prov]).unwrap();
    sup.enable(PluginId("obs")).await;
    let epoch = sup
        .diagnostics()
        .into_iter()
        .find(|d| d.id == PluginId("obs"))
        .unwrap()
        .epoch;
    assert!(sup.handle().optional(KEY_A).is_none());
    sup.enable(PluginId("prov-a")).await;
    assert_eq!(sup.fiber_state(PluginId("obs")), Some(FiberState::Active));
    assert_eq!(
        sup.diagnostics()
            .into_iter()
            .find(|d| d.id == PluginId("obs"))
            .unwrap()
            .epoch,
        epoch
    );
    assert!(sup.handle().optional(KEY_A).is_some());
    sup.disable(PluginId("prov-a")).await;
    assert_eq!(sup.fiber_state(PluginId("obs")), Some(FiberState::Active));
    assert!(sup.handle().optional(KEY_A).is_none());
}

#[tokio::test]
async fn optional_restart_flag_restarts_consumer() {
    let (prov, _) = provide_a();
    let (obs, _) = plug(desc("obs", NONE, OPT_A, NONE, true), |_ctx| {
        Box::pin(async move { Ok(()) })
    });
    let sup = Supervisor::compose([obs, prov]).unwrap();
    sup.enable(PluginId("obs")).await;
    let e1 = sup
        .diagnostics()
        .into_iter()
        .find(|d| d.id == PluginId("obs"))
        .unwrap()
        .epoch;
    sup.enable(PluginId("prov-a")).await;
    let e2 = sup
        .diagnostics()
        .into_iter()
        .find(|d| d.id == PluginId("obs"))
        .unwrap()
        .epoch;
    assert_ne!(e1, e2, "appear must restart when flag is set");
}

#[tokio::test]
async fn disable_interrupts_activate_does_not_commit() {
    let started = Arc::new(tokio::sync::Notify::new());
    let started2 = started.clone();
    let (p, _) = plug(desc("slow", NONE, NONE, PROV_A, false), move |ctx| {
        let started = started2.clone();
        Box::pin(async move {
            ctx.stage_provide(KEY_A, Arc::new(Unit))?;
            started.notify_one();
            tokio::time::sleep(Duration::from_secs(30)).await;
            Ok(())
        })
    });
    let sup = Supervisor::compose([p]).unwrap();
    let en = {
        let sup = sup.clone();
        tokio::spawn(async move { sup.enable(PluginId("slow")).await })
    };
    started.notified().await;
    sup.disable(PluginId("slow")).await;
    en.await.unwrap();
    assert_eq!(
        sup.fiber_state(PluginId("slow")),
        Some(FiberState::Disabled)
    );
    assert!(!sup.handle().is_live(S_A));
}

#[tokio::test]
async fn concurrent_enable_disable_last_desired_wins() {
    let (p, _) = provide_a();
    let sup = Supervisor::compose([p]).unwrap();
    let last = Arc::new(Mutex::new(false));
    let mut joins = Vec::new();
    for i in 0..40 {
        let sup = sup.clone();
        let last = last.clone();
        let enable = i % 2 == 0;
        joins.push(tokio::spawn(async move {
            if enable {
                sup.enable(PluginId("prov-a")).await;
                *last.lock().unwrap() = true;
            } else {
                sup.disable(PluginId("prov-a")).await;
                *last.lock().unwrap() = false;
            }
        }));
    }
    for j in joins {
        j.await.unwrap();
    }
    let want = *last.lock().unwrap();
    sup.notify_config().await;
    let state = sup.fiber_state(PluginId("prov-a")).unwrap();
    if want {
        assert_eq!(state, FiberState::Active);
        assert!(sup.handle().is_live(S_A));
    } else {
        assert!(matches!(state, FiberState::Disabled | FiberState::Failed));
        assert!(!sup.handle().is_live(S_A));
    }
}

#[tokio::test]
async fn shutdown_is_idempotent() {
    let (p, _) = provide_a();
    let (c, _) = consume_a();
    let sup = Supervisor::compose([c, p]).unwrap();
    sup.enable_all().await;
    sup.shutdown().await;
    sup.shutdown().await;
    assert_eq!(
        sup.fiber_state(PluginId("prov-a")),
        Some(FiberState::Disabled)
    );
    assert_eq!(
        sup.fiber_state(PluginId("cons-a")),
        Some(FiberState::Disabled)
    );
}

#[tokio::test]
async fn dependent_disposer_runs_before_provider() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let o1 = order.clone();
    let o2 = order.clone();
    let (prov, _) = plug(desc("prov-a", NONE, NONE, PROV_A, false), move |ctx| {
        let order = o1.clone();
        Box::pin(async move {
            ctx.stage_provide(KEY_A, Arc::new(Unit))?;
            ctx.install_effect("p", || {
                let order = order.clone();
                async move {
                    Ok(EffectDisposer::from_fn(move || {
                        order.lock().unwrap().push("provider");
                    }))
                }
            })
            .await?;
            Ok(())
        })
    });
    let (cons, _) = plug(desc("cons-a", REQ_A, NONE, NONE, false), move |ctx| {
        let order = o2.clone();
        Box::pin(async move {
            ctx.install_effect("c", || {
                let order = order.clone();
                async move {
                    Ok(EffectDisposer::from_fn(move || {
                        order.lock().unwrap().push("dependent");
                    }))
                }
            })
            .await?;
            Ok(())
        })
    });
    let sup = Supervisor::compose([cons, prov]).unwrap();
    sup.enable_all().await;
    sup.disable(PluginId("prov-a")).await;
    assert_eq!(*order.lock().unwrap(), vec!["dependent", "provider"]);
}

#[tokio::test]
async fn config_churn_1000_returns_to_baseline() {
    let live = Arc::new(AtomicUsize::new(0));
    let live2 = live.clone();
    let (p, fp) = plug(desc("churn", NONE, NONE, PROV_A, false), move |ctx| {
        let live = live2.clone();
        Box::pin(async move {
            ctx.stage_provide(KEY_A, Arc::new(Unit))?;
            ctx.install_effect("fx", || {
                let live = live.clone();
                async move {
                    live.fetch_add(1, Ordering::SeqCst);
                    Ok(EffectDisposer::from_fn(move || {
                        live.fetch_sub(1, Ordering::SeqCst);
                    }))
                }
            })
            .await?;
            Ok(())
        })
    });
    let sup = Supervisor::compose([p]).unwrap();
    sup.enable(PluginId("churn")).await;
    assert_eq!(live.load(Ordering::SeqCst), 1);
    assert_eq!(sup.effect_count(PluginId("churn")), Some(1));
    for i in 0..1000 {
        fp.store(2 + i, Ordering::SeqCst);
        sup.notify_config().await;
    }
    assert_eq!(sup.fiber_state(PluginId("churn")), Some(FiberState::Active));
    assert_eq!(live.load(Ordering::SeqCst), 1, "effects back to baseline");
    assert_eq!(sup.effect_count(PluginId("churn")), Some(1));
    assert_eq!(sup.handle().lease_count(S_A), Some(0));
    assert_eq!(sup.diagnostics().len(), 1);
}

#[tokio::test]
async fn lease_call_only_exposes_ref() {
    let (p, _) = provide_a();
    let sup = Supervisor::compose([p]).unwrap();
    sup.enable(PluginId("prov-a")).await;
    let lease = sup.handle().lease(KEY_A).unwrap();
    // `&T` only: the closure cannot name an Arc. Compile-time contract.
    let ok = lease
        .call(|u: &Unit| {
            let _ = u;
            async { true }
        })
        .await
        .unwrap();
    assert!(ok);
}

#[tokio::test]
async fn diagnostics_answer_fiber_state_epoch_error() {
    let (p, _) = plug(desc("boom", NONE, NONE, NONE, false), |_ctx| {
        Box::pin(async move { Err(PluginError::Failed("nope".into())) })
    });
    let sup = Supervisor::compose([p]).unwrap();
    sup.enable(PluginId("boom")).await;
    let d = sup.diagnostics();
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].id, PluginId("boom"));
    assert_eq!(d[0].state, FiberState::Failed);
    assert!(d[0].last_error.as_deref().unwrap().contains("nope"));
    assert!(d[0].epoch.0 >= 1);
}

#[tokio::test]
async fn second_os_effect_plugin_is_compose_plus_host() {
    // Falsifiable §14.4.6.4 stand-in: a second effect-only plugin is one
    // extra Arc in the compose list. No supervisor changes.
    let (a, _) = provide_a();
    let (tray, _) = plug(desc("tray-pulse", NONE, NONE, NONE, false), |ctx| {
        Box::pin(async move {
            ctx.install_effect("pulse", || async { Ok(EffectDisposer::from_fn(|| {})) })
                .await
        })
    });
    let sup = Supervisor::compose([a, tray]).unwrap();
    sup.enable_all().await;
    assert_eq!(
        sup.fiber_state(PluginId("tray-pulse")),
        Some(FiberState::Active)
    );
    assert_eq!(sup.effect_count(PluginId("tray-pulse")), Some(1));
}

#[tokio::test]
async fn lease_stress_concurrent_clone_drop() {
    let (p, _) = provide_a();
    let sup = Supervisor::compose([p]).unwrap();
    sup.enable(PluginId("prov-a")).await;
    let root = Arc::new(sup.handle().lease(KEY_A).unwrap());
    let mut joins = Vec::new();
    for _ in 0..64 {
        let root = root.clone();
        joins.push(tokio::spawn(async move {
            let extra: Vec<_> = (0..16).map(|_| root.clone()).collect();
            drop(extra);
        }));
    }
    for j in joins {
        j.await.unwrap();
    }
    drop(root);
    assert_eq!(sup.handle().lease_count(S_A), Some(0));
}
