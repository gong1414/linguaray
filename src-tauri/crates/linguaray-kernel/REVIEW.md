# PR-2 K0 Go / No-Go review checklist

Spec: `docs/superpowers/specs/2026-08-14-linguaray-plugin-core-design.md` §14.4.

Crate-level gates were verified when PR-4 hooked Shortcuts (branch
`feat/plugin-core-pr1-closeout-pr2`). Production compose is
`plugins::builtin_plugins` — database, secrets, http, drivers, providers,
shortcuts; no stub Fibers.

## Gate

- [x] 1. Kernel does not depend on Bevy or a second app runtime. (`cargo tree -p linguaray-kernel`: futures / tokio / thiserror only.)
- [x] 2. Lifecycle tests have no skip / flaky. (K0: 26 passed, 0 ignored.)
- [x] 3. Equivalent stress: `lease_stress_concurrent_clone_drop` + `config_churn_1000_returns_to_baseline`. Loom not wired (tokio supervisor).
- [ ] 4. Tauri test plugin commands/permissions build on macOS and Windows — still a hookup smoke, not a second runtime.
- [x] 5. 1000 config churns return Fiber / effect / lease counts to baseline.
- [x] 6. Falsifiable metrics:
  - [x] 6.1 Shortcuts public suite assertions unchanged (`tests/shortcuts.rs` still uses `ShortcutController::new`).
  - [x] 6.2 Production startup uses `ShortcutController::load` + `install_effect("shortcuts.replace_all")`. `Controller::new` is not called from `lib.rs`.
  - [x] 6.3 `replace_all` is one effect named `shortcuts.replace_all`; disposer is `replace_all(&[])`.
  - [x] 6.4 Second OS-effect plugin remains a K0 compose-list item (`tray-pulse` test).
  - [x] 6.5 `Supervisor::diagnostics()` answers Fiber / state / epoch / last error.
- [x] 7. Kernel crate itself is unchanged; hookup is one compose list + host inject.
- [x] 8. Sequential continue after crate-level K0 evidence; hookup is PR-4.

**Sign-off:** implementation record 2026-08-14 — crate-level Go; PR-4 is the first production Fiber.

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
