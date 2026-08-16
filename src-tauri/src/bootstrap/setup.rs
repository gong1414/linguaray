//! Tauri `setup()` stages (moved verbatim from the `lib.rs` closure in refactor
//! P3.1, split one phase per function). `lib.rs::run` keeps the plugin wiring
//! and the `collect_commands!` table and delegates here.

use std::sync::Arc;

use tauri::Manager;

use crate::bootstrap::readiness::{
    build_http_client, compute_startup_readiness, init_last_resort_keystore,
    preset_gate_allows_client, startup_migration_guard, validate_all_preset_endpoints,
};
use crate::bootstrap::state::{require_database, AppState, Session};
use crate::bootstrap::tray::build_tray;
use crate::db::migration::{run_migration, FailpointCell, MigrationError};
use crate::db::readiness::DataReadiness;
use crate::db::Database;
use crate::{concurrency, keystore, shortcuts::ShortcutController, tray_state};

/// The `setup()` body. Every stage is fail-soft EXCEPT where a comment says
/// otherwise: the app must always launch, with `DataReadiness` carrying any
/// degradation to the recovery banner.
pub(crate) fn run_setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Resolve the app-local data dir. Review P1 #2: this MUST NOT crash
    // setup — if the platform path is unavailable, fall back to a temp
    // dir so the app still launches (keys simply won't persist; the
    // recovery banner surfaces the problem). `dir` feeds both the
    // keystore and the DB, so a fallback here keeps every downstream
    // `.expect()`-free path alive.
    let dir = app.path().app_local_data_dir().unwrap_or_else(|e| {
        log::error!("app_local_data_dir unavailable, falling back to temp dir: {e}");
        std::env::temp_dir().join("linguaray-data")
    });
    let (keystore, keystore_init_error, client) = init_session(app, &dir);
    manage_data_layer(app, &dir, keystore_init_error);

    // R5: unattended startup update check. The only unsolicited network
    // request the app makes (documented in the README privacy section,
    // disable via check_updates_on_startup). Failure maps onto
    // UpdateCheck::Error, which never disturbs the user.
    startup_update_check(app);

    maybe_show_onboarding(app);

    history_retention_cleanup(app);

    init_shortcuts(app);

    compose_kernel(app, keystore, client);

    // Surface 04: system tray (R2b). Built LAST so a tray init failure
    // does not block DB/keystore/window/shortcut setup. Log-only on
    // error — the app stays usable without a tray.
    if let Err(e) = build_tray(app.handle()) {
        log::error!("tray init failed: {e}");
    }

    autoshow_windows(app);
    Ok(())
}

/// Keystore init (fail-soft, three-level fallback) + hardened HTTP client +
/// preset-endpoint fail-closed gate. Returns the `Arc`'d keystore, the recorded
/// keystore-init error (feeds the readiness reducer), and the client (None =
/// degraded / fail-closed).
fn init_session(
    app: &mut tauri::App,
    dir: &std::path::Path,
) -> (Option<Arc<keystore::Keystore>>, Option<String>, Option<reqwest::Client>) {
    // Review P1 #2: keystore init must NOT crash either. On failure,
    // build the Session WITHOUT a keystore (translate will surface a
    // clear error) and record the failure so the DB readiness block
    // below degrades to NeedsKeystoreRecovery.
    let (keystore, keystore_init_error) = match keystore::Keystore::new(dir.to_path_buf()) {
        Ok(ks) => (Some(ks), None),
        Err(e) => {
            log::error!(
                "keystore init in {} failed: {e}; falling back to temp dir",
                dir.display()
            );
            let fallback_dir = std::env::temp_dir().join("linguaray-keystore");
            // Try the shared temp fallback, then the PID-uniquified
            // last-resort (which itself returns Result — no panic).
            let (ks, lr_err) =
                match keystore::Keystore::new(fallback_dir) {
                    Ok(ks) => (Some(ks), None),
                    Err(e2) => {
                        log::error!("temp keystore fallback also failed: {e2}");
                        match init_last_resort_keystore() {
                            Ok(ks) => (Some(ks), None),
                            // Total failure (OS temp dir unwritable —
                            // unreachable in practice): Session.keystore
                            // is None; every keystore-touching command
                            // surfaces a clear error, and readiness
                            // degrades to NeedsKeystoreRecovery.
                            Err(lr) => {
                                log::error!(
                                    "all last-resort keystore dirs failed: {lr}"
                                );
                                (None, Some(lr))
                            }
                        }
                    }
                };
            let reason = match lr_err {
                Some(lr) => format!(
                    "keystore init in {} failed: {e}; last-resort also failed: {lr}",
                    dir.display()
                ),
                None => format!("keystore init in {}: {e}", dir.display()),
            };
            (ks, Some(reason))
        }
    };
    // Spec §Privacy: every preset endpoint must be HTTPS (loopback HTTP
    // allowed for local engines like Ollama). Reject at config-load so an
    // invalid/leaked preset never ships a request. A bad preset is logged
    // (not fatal) so a single broken catalog entry can't crash startup —
    // see `validate_all_preset_endpoints`. The invalid list is recorded
    // here; every shipped preset currently validates, so this is the
    // fail-closed seam that surfaces a future bad catalog entry.
    let invalid_presets = validate_all_preset_endpoints();
    let preset_validation_ok = preset_gate_allows_client(&invalid_presets);
    if !preset_validation_ok {
        log::error!(
            "preset endpoint validation failed for {} preset(s): {:?}; \
             ALL translation requests are disabled until the catalog is fixed",
            invalid_presets.len(),
            invalid_presets
        );
    }
    // Spec §Privacy: no cross-origin redirects. Review P1 #7: a 30s total
    // request timeout so a hung connection can't freeze translate + the
    // popup indefinitely; lets wire::call classify a real Timeout.
    // `build_http_client` returns the ONLY client we ever use (hardened
    // builder with redirect=none). On a builder failure (pathological
    // TLS-init env) we log + degrade: Session.client is None, so every
    // translate path returns a clear "client unavailable" error. We do
    // NOT fall back to a default client — that would drop
    // redirect(Policy::none()), re-opening the cross-origin-redirect
    // leak the policy exists to close.
    let client = match build_http_client() {
        Ok(c) if preset_validation_ok => Some(c),
        Ok(_) => {
            // Preset validation failed — disable ALL outbound requests
            // (fail-closed, see `preset_gate_allows_client`). A bad preset
            // catalog must not ship any request.
            log::error!("preset validation failed; client disabled (fail-closed)");
            None
        }
        Err(e) => {
            log::error!(
                "{e}; translate is unavailable until the app is restarted in a healthy TLS environment"
            );
            None
        }
    };
    let keystore = keystore.map(Arc::new);
    app.manage(Arc::new(Session {
        client: client.clone(),
        keystore: keystore.clone(),
        gen: concurrency::GenerationToken::new(),
    }));
    (keystore, keystore_init_error, client)
}

/// ── S2a data-readiness startup (DB open → migrate → resume → gate) ──
///
/// NO `.expect()` on DB/migration — the app always launches. Every
/// failure mode degrades `DataReadiness`; provider commands then fail
/// closed via `require_ready`, while the always-available commands
/// (keystore_health / archive_keystore / reset_keystore) keep working
/// so the user can recover.
fn manage_data_layer(app: &mut tauri::App, dir: &std::path::Path, keystore_init_error: Option<String>) {
    let db_path = dir.join("linguaray.db");
    let keystore_dir = dir.to_path_buf();
    // Resolve the canonical settings path via the store plugin. On
    // failure we MUST NOT guess a fallback path (a wrong-dir guess would
    // read/write the wrong settings file — on Windows the store plugin
    // targets AppData (Roaming) while `dir` here is AppLocalData (Local),
    // so `dir.join("settings.json")` is a different file). Instead record
    // the failure by storing `settings_path = None` and degrade to
    // MigrationIncomplete below; migration is skipped entirely (it needs
    // the real settings path). `archive_database_core` also treats `None` as a
    // hard stop: it refuses to re-run migration so the user must retry
    // from a state where the path resolves.
    let (settings_path, settings_resolution_error) =
        match tauri_plugin_store::resolve_store_path(app.handle(), "settings.json") {
            Ok(p) => (Some(p), None),
            Err(e) => {
                let reason = format!("settings path resolution failed: {e}");
                log::error!("{reason}");
                // None: a non-existent sentinel that the readiness gate
                // keeps from ever being read. The startup block below
                // degrades to MigrationIncomplete.
                (None, Some(reason))
            }
        };

    // 1. Open the DB. Err → db=None (app keeps running; readiness
    //    computed below degrades to NeedsDatabaseRecovery).
    let (db_handle, db_open_result) = match Database::open(&db_path) {
        Ok(db) => (Some(Arc::new(db)), Ok(())),
        Err(e) => (None, Err(format!("open linguaray.db: {e}"))),
    };

    // Compute the pre-migration readiness from the three independent
    // startup outcomes via the priority reducer. P1.4: a failed DB open
    // is LOCKED IN — subsequent settings/keystore errors must NOT mask
    // NeedsDatabaseRecovery (the DB is the foundation; there is nothing
    // for a keystore error to gate if no DB exists). Keystore failure
    // beats settings failure (writes need a usable keystore). See
    // `compute_startup_readiness` + tests/startup_readiness.rs.
    let mut readiness = compute_startup_readiness(
        db_open_result,
        settings_resolution_error.clone(),
        keystore_init_error.clone(),
    );

    // 2-4. Only run migration + resume + preflight when the DB opened
    // AND the keystore initialized in its canonical dir AND the settings
    // path resolved. `startup_migration_guard` is the single source of
    // truth for the refusal decision (round-3 P1.3): a None settings
    // path (resolution failed) must refuse migration entirely — no
    // backup, no DB write — rather than run against a guessed path (on
    // Windows the store plugin targets AppData Roaming while `dir` here
    // is AppLocalData Local, so a guessed path would touch a DIFFERENT
    // settings file). The refusal itself is already reflected in
    // `readiness` by the reducer above (NeedsKeystoreRecovery /
    // MigrationIncomplete "settings_path"), so on Err we just skip
    // migration and keep the rest of setup running.
    if let Some(db) = db_handle.clone() {
        let fp = FailpointCell::none();
        match startup_migration_guard(
            settings_path.as_deref(),
            keystore_init_error.as_deref(),
        ) {
            Ok(settings_path_ref) => {
                readiness = match run_migration(&db, &keystore_dir, settings_path_ref, &fp)
                {
                    Ok(()) => {
                        // Resume any in-flight deletes (3-step sweep). A
                        // failure here does NOT exit setup — log + mark
                        // incomplete so the next startup retries.
                        match crate::db::delete::provider_resume_deletions(
                            &db,
                            &keystore_dir,
                        ) {
                            Ok(_) => {
                                // Final keystore preflight: a Corrupt
                                // keystore (detected after migration) →
                                // recovery.
                                match keystore::load_state(&keystore_dir) {
                                    keystore::KeystoreLoadState::Corrupt(e) => {
                                        DataReadiness::NeedsKeystoreRecovery {
                                            reason: format!("keystore corrupt: {e}"),
                                        }
                                    }
                                    _ => DataReadiness::Ready,
                                }
                            }
                            Err(e) => {
                                log::error!("resume_deletions failed: {e}");
                                DataReadiness::migration_incomplete(
                                    "resume_deletions",
                                    format!("resume deletions: {e}"),
                                )
                            }
                        }
                    }
                    Err(MigrationError::NeedsKeystoreRecovery(reason)) => {
                        DataReadiness::NeedsKeystoreRecovery { reason }
                    }
                    Err(MigrationError::SettingsCorrupt(reason)) => {
                        DataReadiness::migration_incomplete("settings", reason)
                    }
                    Err(other) => DataReadiness::migration_incomplete(
                        "migration",
                        other.to_string(),
                    ),
                };
            }
            Err(reason) => {
                // Refused: keystore init failed or settings path could
                // not be resolved. Migration is skipped entirely — NO
                // backup, NO DB write. readiness already carries the
                // correct degraded state from the reducer above.
                log::debug!("startup migration refused: {reason}");
            }
        }
    }

    app.manage(Arc::new(crate::commands::external_api::ExternalApiSlot::new()));
    app.manage(Arc::new(AppState {
        db: parking_lot::RwLock::new(db_handle),
        data_gate: parking_lot::RwLock::new(()),
        readiness: parking_lot::RwLock::new(readiness),
        db_path,
        keystore_dir,
        settings_path,
        tray: Arc::new(parking_lot::Mutex::new(
            tray_state::TrayStateController::new(app.handle().clone()),
        )),
        update_install_in_flight: std::sync::atomic::AtomicBool::new(false),
    }));
}

/// R5: unattended startup update check (gated on `check_updates_on_startup`).
fn startup_update_check(app: &tauri::App) {
    let state: Arc<AppState> = app.state::<Arc<AppState>>().inner().clone();
    let handle = app.handle().clone();
    if crate::settings::load(&handle).check_updates_on_startup {
        tauri::async_runtime::spawn(async move {
            let check = crate::updater::check_remote(&handle).await;
            state.tray.lock().set_update_available(
                crate::updater::tray_should_show_update(&check),
            );
            log::info!("startup update check: {check:?}");
        });
    }
}

/// Show the onboarding window when startup is ready and the Dev data says the
/// user hasn't completed onboarding.
fn maybe_show_onboarding(app: &tauri::App) {
    let startup_ready = app.state::<Arc<AppState>>().readiness.read().is_ready();
    if !startup_ready {
        return;
    }
    if let Some(w) = app.get_webview_window("onboarding") {
        let app_state = app.state::<Arc<AppState>>();
        let gate = app_state.data_gate.read();
        let show = require_database(&app_state, &gate).ok().and_then(|db| {
            db.with_conn(|conn| {
                crate::db::schema::ensure_preference_columns(conn)?;
                let complete: i64 = conn.query_row(
                    "SELECT onboarding_complete FROM preferences WHERE id=1",
                    [],
                    |r| r.get(0),
                )?;
                Ok(complete == 0)
            })
            .ok()
        });
        if show == Some(true) {
            let _ = w.show();
        }
    }
}

/// S2b retention is enforced at startup, independently of whether
/// history is currently enabled. Disabling history intentionally
/// preserves existing encrypted rows, but rows older than the
/// consented retention window must still be removed. Favorites are
/// excluded by `cleanup_expired_now`. Failure is fail-soft because
/// cleanup must never prevent translation or recovery UI startup.
fn history_retention_cleanup(app: &tauri::App) {
    let startup_ready = app.state::<Arc<AppState>>().readiness.read().is_ready();
    if !startup_ready {
        return;
    }
    if let Some(history_db) = app.state::<Arc<AppState>>().db.read().clone() {
        match history_db.with_conn(crate::history::cleanup_expired_now) {
            Ok(removed) if removed > 0 => {
                log::info!("expired encrypted history removed: {removed}");
            }
            Ok(_) => {}
            Err(error) => {
                log::warn!("encrypted history retention cleanup failed: {error}");
            }
        }
    }
}

/// R3b Surface 07: seed/load the persisted revisioned shortcut map,
/// then atomically bind every available action through one registrar.
/// Startup OS conflicts are fail-soft and appear in shortcut_list;
/// DB failures leave the Settings page in its retryable load-error
/// state without preventing the rest of the application from starting.
fn init_shortcuts(app: &tauri::App) -> Option<Arc<crate::plugins::shortcuts::ShortcutsPlugin>> {
    let shortcut_db = app
        .state::<Arc<AppState>>()
        .db
        .read()
        .clone();
    if let Some(shortcut_db) = shortcut_db {
        let registrar = Arc::new(crate::plugins::shortcuts::TauriShortcutRegistrar::new(
            app.handle().clone(),
        ));
        match ShortcutController::load(shortcut_db, registrar.clone()) {
            Ok(controller) => {
                let controller = Arc::new(controller);
                app.manage(controller.clone());
                return Some(Arc::new(
                    crate::plugins::shortcuts::ShortcutsPlugin::new(registrar, controller),
                ));
            }
            Err(error) => {
                log::error!("shortcut controller startup failed: {error}");
            }
        }
    }
    None
}

/// Compose + enable the kernel plugin supervisor (database / secrets / http /
/// shortcuts / popup / tray_state).
fn compose_kernel(
    app: &tauri::App,
    keystore: Option<Arc<keystore::Keystore>>,
    client: Option<reqwest::Client>,
) {
    let shortcuts_plugin = init_shortcuts(app);
    let database_plugin = Arc::new(crate::plugins::database::DatabasePlugin::new(
        app.state::<Arc<AppState>>().db.read().clone(),
    ));
    let secrets_plugin =
        Arc::new(crate::plugins::secrets::SecretsPlugin::new(keystore));
    let http_plugin = Arc::new(crate::plugins::http::HttpPlugin::new(client));
    let popup_plugin = Some(Arc::new(crate::plugins::popup::PopupPlugin::new(
        app.handle().clone(),
    )));
    let tray_plugin = Some(Arc::new(crate::plugins::tray_state::TrayPlugin::new(
        app.state::<Arc<AppState>>().tray.clone(),
    )));
    match linguaray_kernel::Supervisor::compose(crate::plugins::builtin_plugins(
        database_plugin,
        secrets_plugin,
        http_plugin,
        shortcuts_plugin,
        popup_plugin,
        tray_plugin,
    )) {
        Ok(supervisor) => {
            tauri::async_runtime::block_on(supervisor.enable_all());
            app.manage(supervisor);
        }
        Err(error) => {
            log::error!("kernel compose failed: {error}");
        }
    }
}

/// Testability hook (hygiene-6): the screenshot-baseline script
/// launches the packaged app with one of these env vars to surface a
/// normally hidden window without UI automation (tray-hidden
/// main, consent-gated input, event-driven popup) or to build the
/// on-demand OCR overlay exactly like the Windows capture path.
fn autoshow_windows(app: &tauri::App) {
    for (var, label) in [
        ("LINGUARAY_AUTOSHOW_MAIN", "main"),
        ("LINGUARAY_AUTOSHOW_INPUT", "input"),
        ("LINGUARAY_AUTOSHOW_POPUP", "popup"),
    ] {
        if std::env::var_os(var).is_some() {
            if let Some(w) = app.get_webview_window(label) {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }
    }
    if std::env::var_os("LINGUARAY_AUTOSHOW_OCR").is_some() {
        if let Err(e) = tauri::async_runtime::block_on(crate::commands::ocr::ensure_overlay_window(
            app.handle(),
        )) {
            log::error!("autoshow ocr overlay: {e}");
        }
    }
}
