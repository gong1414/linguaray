# PR-2 K0 Go / No-Go review checklist

Spec: `docs/superpowers/specs/2026-08-14-linguaray-plugin-core-design.md` §14.4.
Reviewers sign **after** reading the crate and the K0 test output. This file is
not a Go declaration.

Production `builtin_plugins()` does **not** exist. This crate is not called
from `src-tauri/src/lib.rs`. A No-Go still keeps PR-1 catalog work.

## Gate (unsigned)

- [ ] 1. Kernel does not depend on Bevy or a second app runtime.
- [ ] 2. Lifecycle tests have no skip / flaky.
- [ ] 3. Loom **or equivalent** stress did not find double-dispose / out-of-order commit. (`lease_stress_concurrent_clone_drop` + `config_churn_1000_returns_to_baseline` are the equivalent in this crate; loom is not wired because it cannot share a tokio supervisor.)
- [ ] 4. Tauri test plugin commands/permissions build on macOS and Windows — **N/A until PR-4 hookup**.
- [ ] 5. 1000 config churns return Fiber / effect / lease counts to baseline (`config_churn_1000_returns_to_baseline`).
- [ ] 6. Falsifiable metrics (PR-4 still owns production Shortcuts):
  - [ ] 6.1 Shortcuts public suite assertions unchanged — **N/A (not hooked)**.
  - [ ] 6.2 No dual registration on the production path — **N/A (not hooked)**.
  - [ ] 6.3 `replace_all` is one effect — API shape is `install_effect(name, setup)`; per-binding effects are not provided.
  - [ ] 6.4 Second OS-effect plugin (K0 `tray-pulse`) is compose + one test plugin; supervisor unchanged.
  - [ ] 6.5 `Supervisor::diagnostics()` answers Fiber / state / epoch / last error without reading `AppState`.
- [ ] 7. Production binary is not hooked; startup / idle RSS must stay within noise of `main`.
- [ ] 8. This checklist signed before writing the PR-4 plan.

**Sign-off:** _name / date / Go or No-Go_

## K0 coverage map

| Invariant | Test |
|---|---|
| D-id | `d_id_duplicate_plugin_panics` |
| D-svc | `d_svc_second_stage_provide_rolls_back` |
| D-ty | `d_ty_second_type_on_same_service_id_fails` |
| D-self | `d_self_rejected_at_compose` |
| D-cyc | `d_cyc_rejected_at_compose` |
| D-ord | `d_ord_same_rank_by_plugin_id_bytes` |
| D-pick | `d_pick_duplicate_declare_is_compose_error` |
| Staging invisible | `dependent_stays_pending_until_provider_commits` |
| Partial rollback | `partial_activate_rolls_back_provide_and_effect` |
| Disposer panic continues | `disposer_panic_does_not_skip_rest` |
| Lease clone count | `lease_clone_counts_and_drop_reaches_zero` |
| Drain to zero | `drain_waits_for_zero_then_unloads` |
| 30s+5s ForcedStop | `drain_defaults_are_30s_then_5s` + `drain_timeout_then_grace_forced_stop` |
| Epoch / stale lease | `epoch_mismatch_on_stale_lease` |
| Optional no-restart | `optional_appear_withdraw_does_not_restart_by_default` |
| Optional restart flag | `optional_restart_flag_restarts_consumer` |
| Disable interrupts activate | `disable_interrupts_activate_does_not_commit` |
| Last desired wins | `concurrent_enable_disable_last_desired_wins` |
| Shutdown idempotent | `shutdown_is_idempotent` |
| Dependent disposer first | `dependent_disposer_runs_before_provider` |
| `lease.call` is `&T` | `lease_call_only_exposes_ref` |
