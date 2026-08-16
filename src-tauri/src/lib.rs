//! LinguaRay — translation core.
//!
//! Thin host: Tauri commands live in `commands/`. Vendor rows live in
//! `linguaray-catalog`. Official features are in-tree Capability/Driver plugins.
//! Traditional engines live in `plugins/drivers/traditional`.
//! Startup/state/tray decomposition lives in `bootstrap/` (refactor P3.1).

pub mod a11y;
pub mod adapter;
pub use crate::plugins::clipboard;
pub mod bootstrap;
pub mod commands;
pub mod concurrency;
pub mod cursor;
pub mod db;
pub mod dict;
pub mod ocr;
pub mod tts;
pub mod external_api;
pub mod onboarding;
pub mod updater;
pub mod balance;
pub mod engines;
pub mod error;
pub mod fs_acl;
pub use crate::plugins::history;
pub use crate::plugins::vocabulary;
pub mod keystore;
pub mod plugins;
pub use crate::plugins::popup;
pub mod providers;
pub use crate::plugins::selection;
pub use crate::plugins::selection_engine;
pub mod service;
pub mod settings;
pub mod shortcuts;
pub use crate::plugins::tray_state;
pub mod uuid_util;
pub mod wire;

#[cfg(test)]
use std::sync::Arc;

use tauri::Manager;
use tauri_plugin_global_shortcut::Builder as GlobalShortcutBuilder;
use tauri_specta::{collect_commands, Builder as SpectaBuilder, ErrorHandlingMode};

use crate::commands::{
    a11y_status, archive_keystore, dict_install_package, dict_list_packages,
    screen_capture_status,
    dict_lookup, get_settings,
    history_clear_all, history_delete_session, history_export, history_privacy_status,
    history_search, history_set_enabled, history_set_retention, history_toggle_favorite,
    key_status, keystore_health, open_settings_window,
    provider_confirm_and_set_active, provider_create, provider_delete, provider_duplicate,
    provider_get_active_selection, provider_get_models, provider_list, provider_list_presets,
    provider_reorder, provider_set_active, provider_set_key, provider_test_connection,
    provider_toggle, provider_update, reset_keystore, set_setting, shortcut_check_conflict,
    shortcut_list, shortcut_recording_begin, shortcut_recording_end, shortcut_reset_defaults,
    shortcut_save, translate_clipboard, translate_selection_ipc,
    translate_session, vocabulary_add, vocabulary_delete, vocabulary_export_anki,
    vocabulary_export_file, vocabulary_list, ocr_capture, ocr_capture_region, ocr_from_image,
    ocr_recognize_bytes, ocr_from_clipboard, tts_speak, tts_stop, external_api_disable,
    external_api_enable, external_api_regenerate_token, external_api_status, updater_check,
    updater_download_install,
    onboarding_complete, onboarding_next, onboarding_status, provider_get_balance,
};
#[cfg(test)]
use crate::db::readiness::DataReadiness;
#[cfg(test)]
use crate::db::Database;

// Re-export so integration tests can reference the error enum as
// `linguaray_lib::Error` (mirrors `service::TranslationOutcome` usage).
pub use crate::error::Error;

pub use crate::commands::providers::{
    db_set_active_primary, handle_switch_provider, handle_switch_provider_core, measure_latency_ms,
    set_key_blocking, ConnectionResult, ModelInfo, ProviderCommandError, SetActiveOutcome,
    SetActiveResult,
};
pub use crate::commands::translate::{
    decide_clipboard_popup, resolve_target_language, run_translate_session_no_settings,
    ClipboardPopupDecision, TranslateSessionRequest,
};
pub use crate::service::TranslateSessionResult;

/// Single source of truth for Tauri command registration and generated
/// TypeScript bindings. Keeping the command list here prevents the runtime
/// handler and `src/bridge/bindings.ts` from drifting apart.
pub fn specta_builder() -> SpectaBuilder<tauri::Wry> {
    SpectaBuilder::new()
        .commands(collect_commands![
            translate_clipboard,
            translate_session,
            translate_selection_ipc,
            key_status,
            get_settings,
            set_setting,
            a11y_status,
            screen_capture_status,
            keystore_health,
            archive_keystore,
            reset_keystore,
            provider_list_presets,
            provider_list,
            provider_create,
            provider_update,
            provider_duplicate,
            provider_delete,
            provider_reorder,
            provider_toggle,
            provider_set_key,
            provider_set_active,
            provider_get_active_selection,
            provider_confirm_and_set_active,
            provider_get_models,
            provider_test_connection,
            provider_get_balance,
            open_settings_window,
            shortcut_list,
            shortcut_check_conflict,
            shortcut_save,
            shortcut_reset_defaults,
            shortcut_recording_begin,
            shortcut_recording_end,
            history_privacy_status,
            history_set_enabled,
            history_set_retention,
            history_clear_all,
            history_search,
            history_toggle_favorite,
            history_delete_session,
            history_export,
            vocabulary_add,
            vocabulary_list,
            vocabulary_delete,
            vocabulary_export_file,
            vocabulary_export_anki,
            dict_lookup,
            dict_list_packages,
            dict_install_package,
            ocr_capture,
            ocr_capture_region,
            ocr_from_image,
            ocr_recognize_bytes,
            ocr_from_clipboard,
            tts_speak,
            tts_stop,
            external_api_enable,
            external_api_status,
            external_api_disable,
            external_api_regenerate_token,
            updater_check,
            updater_download_install,
            onboarding_status,
            onboarding_next,
            onboarding_complete,
        ])
        // Match raw Tauri invoke semantics so existing UI error handling stays
        // unchanged while callers move to the generated command wrappers.
        .error_handling(ErrorHandlingMode::Throw)
        // SQLite revisions and timestamps are represented as JSON numbers in
        // the existing wire contract and remain well inside JS safe integers.
        .dangerously_cast_bigints_to_number()
}

pub fn export_typescript_bindings(path: impl AsRef<std::path::Path>) {
    specta_builder()
        .export(specta_typescript::Typescript::default(), path)
        .expect("failed to export TypeScript bindings");
}

// Bootstrap split (refactor P3.1): the former monolithic startup/state/tray
// code lives in `bootstrap/`. These re-exports keep every historical path —
// `crate::AppState`, `crate::refresh_tray_if_available`, `crate::on_hotkey`,
// `linguaray_lib::compute_startup_readiness`, the `lib.rs` unit tests —
// resolving unchanged.
pub use crate::bootstrap::readiness::{compute_startup_readiness, startup_migration_guard};
pub use crate::bootstrap::state::{AppState, Session};
pub use crate::bootstrap::tray::{refresh_tray, refresh_tray_if_available};
use crate::bootstrap::hotkeys::{on_hotkey, on_input_hotkey};
use crate::bootstrap::readiness::apply_keystore_recovery_db_cleanup;
use crate::bootstrap::state::{require_database, require_database_write};
use crate::bootstrap::tray::surface_main;
#[cfg(test)]
use crate::bootstrap::readiness::{
    build_http_client, init_last_resort_keystore, preset_gate_allows_client,
    validate_all_preset_endpoints, validate_preset_endpoints,
};
#[cfg(test)]
use crate::bootstrap::state::session_client;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    export_typescript_bindings(concat!(env!("CARGO_MANIFEST_DIR"), "/../src/bridge/bindings.ts"));

    let specta_builder = specta_builder();

    // Round-2 review P1 #2: the REAL registration failure point is the plugin's
    // `setup`, which calls `manager.register(shortcut)?` for every `with_shortcut`
    // and propagates a conflict error to `.run().expect()` → startup crash.
    // Parse-time tolerance (round-1) was insufficient. Fix: register the plugin
    // with NO shortcuts (Builder builds a plugin, but registers nothing at setup),
    // then in the app `setup()` call the runtime `on_shortcut` PER shortcut and
    // catch each Result — a conflict logs + skips THAT shortcut only, the app and
    // the other shortcut keep running.
    let shortcut_plugin = GlobalShortcutBuilder::new().build();

    tauri::Builder::default()
        // single-instance MUST be first: defense-in-depth on top of the real
        // per-dir fs2 flock in keystore.rs (the flock is what serializes a second
        // instance/external writer on the same dir; single-instance just avoids
        // spawning a second process). This plugin focuses the existing instance
        // instead of launching a second.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = surface_main(app);
        }))
        .plugin(shortcut_plugin)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| crate::bootstrap::setup::run_setup(app))
        .invoke_handler(specta_builder.invoke_handler())
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                if let Some(supervisor) = app.try_state::<linguaray_kernel::Supervisor>() {
                    let supervisor = supervisor.inner().clone();
                    tauri::async_runtime::spawn(async move {
                        supervisor.shutdown().await;
                    });
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::TranslationOutcome;

    /// Build a `TrayStateController` backed by a `RecordingRenderer` for unit
    /// tests that construct an `AppState` (the `tray` field is required by the
    /// struct but these tests only inspect `readiness`).
    fn test_tray_controller() -> tray_state::TrayStateController {
        tray_state::TrayStateController::with_renderer(
            Arc::new(tray_state::RecordingRenderer::default()),
            tray_state::Locale::En,
        )
    }

    #[test]
    fn keystore_recovery_disables_and_clears_irrecoverable_encrypted_content() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("recovery-history.db");
        let db = Arc::new(Database::open(&db_path).unwrap());
        db.with_conn(|conn| {
            let tx = conn.transaction()?;
            crate::db::schema::create_all_tables(&tx)?;
            crate::db::schema::seed_singletons(&tx)?;
            tx.execute("UPDATE preferences SET history_enabled=1 WHERE id=1", [])?;
            tx.execute(
                "INSERT INTO history_sessions
                 (session_uuid,timestamp,trigger_source,target_language,is_favorite,
                  source_text_encrypted,source_text_nonce,crypto_version)
                 VALUES ('s1',1,'input','zh',0,X'AA',X'000102030405060708090A0B',1)",
                [],
            )?;
            tx.execute(
                "INSERT INTO vocabulary
                 (item_uuid,timestamp,source_language,target_language,word_encrypted,
                  word_nonce,definition_encrypted,definition_nonce,crypto_version)
                 VALUES ('v1',1,'en','zh',X'AA',X'000102030405060708090A0B',
                         X'BB',X'000102030405060708090A0B',1)",
                [],
            )?;
            tx.commit()?;
            Ok(())
        })
        .unwrap();
        let app = Arc::new(AppState {
            db: parking_lot::RwLock::new(Some(db.clone())),
            data_gate: parking_lot::RwLock::new(()),
            readiness: parking_lot::RwLock::new(DataReadiness::Ready),
            db_path,
            keystore_dir: dir.path().join("keystore"),
            settings_path: Some(dir.path().join("settings.json")),
            tray: Arc::new(parking_lot::Mutex::new(test_tray_controller())),
            update_install_in_flight: std::sync::atomic::AtomicBool::new(false),
        });

        apply_keystore_recovery_db_cleanup(&app).unwrap();
        let state = db
            .with_conn(|conn| {
                Ok((
                    conn.query_row(
                        "SELECT history_enabled FROM preferences WHERE id=1",
                        [],
                        |row| row.get::<_, i64>(0),
                    )?,
                    conn.query_row("SELECT COUNT(*) FROM history_sessions", [], |row| {
                        row.get::<_, i64>(0)
                    })?,
                    conn.query_row("SELECT COUNT(*) FROM vocabulary", [], |row| {
                        row.get::<_, i64>(0)
                    })?,
                ))
            })
            .unwrap();
        assert_eq!(state, (0, 0, 0));
    }

    /// Task 5a: `get_data_readiness` returns a `DataReadiness` (not a hand-rolled
    /// JSON `String`) so the frontend gets a properly serialized tagged union via
    /// Tauri's auto-serialization. This IS a wire-contract change from the
    /// pre-S2a `String` return: the old command returned a JSON-ENCODED STRING,
    /// the new one ships a JSON object via `#[serde(tag="state", rename_all="snake_case")]`
    /// on `DataReadiness` itself.
    #[test]
    fn read_data_readiness_from_state_returns_typed_object() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState {
            db: parking_lot::RwLock::new(None),
            data_gate: parking_lot::RwLock::new(()),
            readiness: parking_lot::RwLock::new(DataReadiness::Ready),
            db_path: dir.path().join("linguaray.db"),
            keystore_dir: dir.path().join("keystore"),
            settings_path: Some(dir.path().join("settings.json")),
            tray: Arc::new(parking_lot::Mutex::new(test_tray_controller())),
            update_install_in_flight: std::sync::atomic::AtomicBool::new(false),
        };
        let got = state.readiness.read().clone();
        assert_eq!(got, DataReadiness::Ready);

        // Verify the serialized shape is the SAME tagged-union JSON the frontend
        // already consumes (`{"state":"ready"}`), so the signature change does
        // not alter the wire format.
        let json = serde_json::to_string(&got).unwrap();
        assert_eq!(json, "{\"state\":\"ready\"}");
    }

    /// A non-Ready readiness must round-trip with its `reason` payload intact
    /// (this is the case that matters for the recovery banner).
    #[test]
    fn read_data_readiness_preserves_reason_payload() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState {
            db: parking_lot::RwLock::new(None),
            data_gate: parking_lot::RwLock::new(()),
            readiness: parking_lot::RwLock::new(DataReadiness::NeedsKeystoreRecovery {
                reason: "corrupt envelope".into(),
            }),
            db_path: dir.path().join("linguaray.db"),
            keystore_dir: dir.path().join("keystore"),
            settings_path: Some(dir.path().join("settings.json")),
            tray: Arc::new(parking_lot::Mutex::new(test_tray_controller())),
            update_install_in_flight: std::sync::atomic::AtomicBool::new(false),
        };
        let got = state.readiness.read().clone();
        let json = serde_json::to_string(&got).unwrap();
        assert!(
            json.contains("\"state\":\"needs_keystore_recovery\""),
            "{json}"
        );
        assert!(json.contains("\"reason\":\"corrupt envelope\""), "{json}");
    }

    /// Task 5b: building the HTTP client must NOT `.expect()`/panic. The hardened
    /// builder (redirect=none + timeouts) is the only client we ever return — on
    /// a builder error we surface `Err` rather than silently degrading to a
    /// privacy-losing default client. This test is network-free: it only checks
    /// the builder succeeds and returns a usable `reqwest::Client`.
    #[test]
    fn build_http_client_returns_usable_client() {
        let c =
            build_http_client().expect("hardened HTTP client builder must succeed in a normal env");
        // No network: a freshly built client is still a real reqwest::Client. We
        // confirm it's usable by constructing a request (build, not send).
        let _req = c.get("https://invalid.invalid/");
    }

    /// Task 5b: `build_http_client` returns `Result` and must NOT silently fall
    /// back to a default client (which would drop `redirect(Policy::none())`). A
    /// builder error must propagate as `Err`, not be swallowed. In a normal
    /// environment the hardened builder succeeds, so this asserts the Ok shape.
    #[test]
    fn build_http_client_returns_result_not_client() {
        // Signature contract: the function returns Result<Client, String>, not a
        // bare Client. This compiles only if the signature is the Result form,
        // locking in the no-panic / no-silent-fallback contract at the type level.
        let result: Result<reqwest::Client, String> = build_http_client();
        assert!(result.is_ok(), "normal env must build the hardened client");
    }

    /// Task 5b: preset-endpoint validation must NOT `.expect()`/panic. A bad
    /// preset is logged + skipped, not fatal — every shipped preset validates,
    /// so this exercises the happy path AND the skip path via the helper directly.
    #[test]
    fn validate_preset_endpoints_does_not_panic() {
        // All shipped presets are HTTPS/loopback-valid → Ok (empty error list).
        let invalid = validate_all_preset_endpoints();
        assert!(
            invalid.is_empty(),
            "shipped presets must all validate: {invalid:?}"
        );

        // A single bad endpoint validates to Err (the per-endpoint check the
        // loop calls), proving the loop would skip rather than panic.
        assert!(
            providers::validate_endpoint("ftp://evil.example/x").is_err(),
            "ftp must be rejected"
        );
    }

    /// Task 5b: `init_last_resort_keystore` must return `Result<Keystore, String>`
    /// and NEVER panic. In a normal environment the OS temp dir is writable, so
    /// this asserts the Ok shape — locking in the no-panic contract at the type
    /// level (the function signature is `Result`, so a panic in the unreachable
    /// final arm would now be a compile error).
    #[test]
    fn init_last_resort_keystore_returns_result_no_panic() {
        // Signature contract: returns Result, not a bare Keystore. Compiles only
        // if the signature is the Result form.
        let result: Result<keystore::Keystore, String> = init_last_resort_keystore();
        assert!(
            result.is_ok(),
            "normal OS temp dir must be writable for a last-resort keystore: {:?}",
            result.err()
        );
    }

    // ─── Round-3 P1.1: preset fail-closed chain, deterministically ──────────
    //
    // The review requirement: prove that when an invalid preset EXISTS, no
    // network request can be produced — validating `validate_endpoint("ftp://…")`
    // in isolation is not enough. These tests pin the full chain:
    //   1. a catalog containing an invalid endpoint surfaces that id,
    //   2. that id flips `preset_gate_allows_client` to false (client disabled),
    //   3. a client-less Session makes `session_client` return Err — the first
    //      barrier every translate entry-point (`translate_clipboard`,
    //      `on_hotkey`) hits before it can build a
    //      request. No client handle ⇒ no request can ever be shipped.
    #[test]
    fn invalid_preset_in_catalog_blocks_client_gate() {
        let bad = providers::ProviderPreset {
            id: "evil".into(),
            label: "Evil".into(),
            endpoint: "ftp://evil.example/x".into(),
            protocol: linguaray_contracts::ProtocolKind::OpenaiChat,
            default_model: "x".into(),
            needs_key: true,
            auth: linguaray_contracts::AuthKind::Bearer,
        };
        let good = providers::presets()
            .into_iter()
            .next()
            .expect("shipped catalog is non-empty");
        let invalid = validate_preset_endpoints(&[good, bad]);
        assert_eq!(
            invalid,
            vec!["evil".to_string()],
            "the invalid endpoint must surface in the invalid list"
        );
        assert!(
            !preset_gate_allows_client(&invalid),
            "a single invalid preset must disable the client entirely (fail-closed)"
        );
    }

    #[test]
    fn all_valid_presets_keep_client_gate_open() {
        // Positive control: a clean catalog keeps the gate open.
        let invalid = validate_preset_endpoints(&providers::presets());
        assert!(
            invalid.is_empty(),
            "shipped catalog must validate: {invalid:?}"
        );
        assert!(preset_gate_allows_client(&invalid));
    }

    #[test]
    fn session_client_refuses_when_client_disabled() {
        // A Session whose client is None (the fail-closed setup outcome) must
        // make `session_client` return Err — the deterministic barrier every
        // translate entry-point uses before building a request. No network, no
        // reqwest involvement: a None handle cannot ship anything.
        let session = Session {
            client: None,
            keystore: None,
            gen: concurrency::GenerationToken::new(),
        };
        let err = session_client(&session).unwrap_err();
        assert!(err.contains("unavailable"), "{err}");
    }

    #[test]
    fn session_client_returns_client_when_present() {
        // Positive control: a healthy Session yields the client, so the barrier
        // only trips on the disabled path (not universally).
        let c = build_http_client().expect("hardened builder succeeds in a normal env");
        let session = Session {
            client: Some(c),
            keystore: None,
            gen: concurrency::GenerationToken::new(),
        };
        assert!(session_client(&session).is_ok());
    }

    #[test]
    fn database_gate_allows_keystore_recovery_banner() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("gate.db");
        let db = Arc::new(Database::open(&db_path).unwrap());
        let state = AppState {
            db: parking_lot::RwLock::new(Some(db)),
            data_gate: parking_lot::RwLock::new(()),
            readiness: parking_lot::RwLock::new(DataReadiness::NeedsKeystoreRecovery {
                reason: "corrupt".into(),
            }),
            db_path,
            keystore_dir: dir.path().join("keystore"),
            settings_path: Some(dir.path().join("settings.json")),
            tray: Arc::new(parking_lot::Mutex::new(test_tray_controller())),
            update_install_in_flight: std::sync::atomic::AtomicBool::new(false),
        };
        let gate = state.data_gate.read();
        assert!(
            require_database(&state, &gate).is_ok(),
            "NeedsKeystoreRecovery must not block the database gate"
        );
    }

    #[test]
    fn database_gate_fails_without_handle() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState {
            db: parking_lot::RwLock::new(None),
            data_gate: parking_lot::RwLock::new(()),
            readiness: parking_lot::RwLock::new(DataReadiness::Ready),
            db_path: dir.path().join("missing.db"),
            keystore_dir: dir.path().join("keystore"),
            settings_path: Some(dir.path().join("settings.json")),
            tray: Arc::new(parking_lot::Mutex::new(test_tray_controller())),
            update_install_in_flight: std::sync::atomic::AtomicBool::new(false),
        };
        let gate = state.data_gate.read();
        assert!(require_database(&state, &gate).is_err());
    }

    // ─── R2a Task 6: translate_clipboard 分支决策 ──────────────────────────────

    #[test]
    fn clipboard_decision_single_success_uses_legacy_event() {
        let result = TranslateSessionResult {
            outcomes: vec![TranslationOutcome {
                uuid: "u1".into(),
                result: Ok(service::Translation {
                    text: "你好".into(),
                    engine: "provider/u1".into(),
                }),
            }],
            actual_engine: Some("provider/u1".into()),
        };
        let d = decide_clipboard_popup(&result);
        assert!(matches!(d, ClipboardPopupDecision::SingleSuccess { .. }));
        if let ClipboardPopupDecision::SingleSuccess { text, engine } = d {
            assert_eq!(text, "你好");
            assert_eq!(engine, "provider/u1");
        }
    }

    #[test]
    fn clipboard_decision_parallel_uses_multi_event() {
        let result = TranslateSessionResult {
            outcomes: vec![
                TranslationOutcome {
                    uuid: "u1".into(),
                    result: Ok(service::Translation {
                        text: "a".into(),
                        engine: "p/u1".into(),
                    }),
                },
                TranslationOutcome {
                    uuid: "u2".into(),
                    result: Err(crate::error::Error::LocalNoFallback),
                },
            ],
            actual_engine: None,
        };
        let d = decide_clipboard_popup(&result);
        assert!(matches!(d, ClipboardPopupDecision::Multi));
    }

    #[test]
    fn clipboard_decision_single_failure_is_error() {
        let result = TranslateSessionResult {
            outcomes: vec![TranslationOutcome {
                uuid: "u1".into(),
                result: Err(crate::error::Error::LocalNoFallback),
            }],
            actual_engine: None,
        };
        let d = decide_clipboard_popup(&result);
        match d {
            ClipboardPopupDecision::Error(msg) => assert!(msg.contains("no fallback"), "{msg}"),
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn clipboard_decision_all_parallel_failed_is_error() {
        let result = TranslateSessionResult {
            outcomes: vec![
                TranslationOutcome {
                    uuid: "u1".into(),
                    result: Err(crate::error::Error::LocalNoFallback),
                },
                TranslationOutcome {
                    uuid: "u2".into(),
                    result: Err(crate::error::Error::LocalNoFallback),
                },
            ],
            actual_engine: None,
        };
        let d = decide_clipboard_popup(&result);
        assert!(matches!(d, ClipboardPopupDecision::Error(_)));
    }

    #[test]
    fn parse_model_ids_openai_data_array() {
        let body = serde_json::json!({
            "data": [
                {"id": "gpt-4o-mini", "object": "model"},
                {"id": "gpt-4o", "object": "model"},
                {"object": "model"}
            ]
        });
        assert_eq!(
            crate::commands::providers::parse_model_ids(&body),
            vec!["gpt-4o-mini".to_string(), "gpt-4o".to_string()]
        );
    }

    #[test]
    fn parse_model_ids_anthropic_top_level_array() {
        let body = serde_json::json!([
            {"id": "claude-sonnet-4-5", "type": "model"},
            {"id": "claude-haiku-4-5"}
        ]);
        assert_eq!(
            crate::commands::providers::parse_model_ids(&body),
            vec![
                "claude-sonnet-4-5".to_string(),
                "claude-haiku-4-5".to_string()
            ]
        );
    }

    #[test]
    fn parse_model_ids_unknown_shape_is_empty() {
        let body = serde_json::json!({"models": [{"name": "x"}]});
        assert!(crate::commands::providers::parse_model_ids(&body).is_empty());
    }
}
