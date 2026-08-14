use std::sync::{Arc, Mutex};

use linguaray_lib::db::{schema, shortcuts as shortcut_db, Database};
use linguaray_lib::shortcuts::{
    canonicalize, Registrar, ShortcutAction, ShortcutController, ShortcutError,
    ShortcutRegistrationState,
};

fn database() -> Arc<Database> {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.keep().join("shortcuts.sqlite3");
    let db = Arc::new(Database::open(&path).expect("open database"));
    db.with_conn(|conn| {
        schema::create_all_tables(conn)?;
        schema::seed_singletons(conn)
    })
    .expect("create schema");
    db
}

#[derive(Default)]
struct FakeRegistrar {
    registered: Mutex<Vec<(ShortcutAction, String)>>,
    fail_next: Mutex<bool>,
}

impl FakeRegistrar {
    fn fail_next(&self) {
        *self.fail_next.lock().expect("fail mutex") = true;
    }

    fn registered(&self) -> Vec<(ShortcutAction, String)> {
        self.registered.lock().expect("registered mutex").clone()
    }
}

impl Registrar for FakeRegistrar {
    fn replace_all(&self, shortcuts: &[(ShortcutAction, String)]) -> Result<(), String> {
        let mut fail = self.fail_next.lock().expect("fail mutex");
        if *fail {
            *fail = false;
            return Err("system reserved".into());
        }
        *self.registered.lock().expect("registered mutex") = shortcuts.to_vec();
        Ok(())
    }
}

fn controller() -> (Arc<Database>, Arc<FakeRegistrar>, ShortcutController) {
    let db = database();
    let registrar = Arc::new(FakeRegistrar::default());
    let controller =
        ShortcutController::new(db.clone(), registrar.clone()).expect("construct controller");
    (db, registrar, controller)
}

#[test]
fn canonical_modifier_order_and_aliases_are_stable() {
    assert_eq!(
        canonicalize("shift+cmd+option+control+k").unwrap(),
        "Ctrl+Alt+Shift+Super+K"
    );
    assert_eq!(canonicalize("alt+space").unwrap(), "Alt+Space");
    assert_eq!(canonicalize("CTRL+esc").unwrap(), "Ctrl+Escape");
    assert!(
        canonicalize("Ctrl+Alt").is_err(),
        "a primary key is required"
    );
    assert!(
        canonicalize("Ctrl+A+B").is_err(),
        "exactly one primary key is allowed"
    );
}

#[test]
fn seeds_exactly_four_defaults_and_registers_ocr() {
    let (db, registrar, controller) = controller();
    let snapshot = controller.snapshot().unwrap();

    assert_eq!(snapshot.revision, 0);
    assert_eq!(snapshot.entries.len(), 4);
    assert_eq!(snapshot.combo(ShortcutAction::Selection), Some("Alt+Space"));
    assert_eq!(snapshot.combo(ShortcutAction::Input), Some("Ctrl+Space"));
    assert_eq!(
        snapshot.combo(ShortcutAction::Clipboard),
        Some("Ctrl+Alt+Space")
    );
    assert_eq!(snapshot.combo(ShortcutAction::Ocr), Some("Alt+Shift+Space"));
    let ocr = snapshot.entry(ShortcutAction::Ocr).unwrap();
    assert!(ocr.available);
    assert_eq!(
        ocr.registration_state,
        ShortcutRegistrationState::Registered
    );
    assert_eq!(ocr.registration_error, None);
    assert_eq!(
        snapshot
            .entry(ShortcutAction::Selection)
            .unwrap()
            .registration_state,
        ShortcutRegistrationState::Registered
    );
    assert_eq!(shortcut_db::load(&db).unwrap().len(), 4);
    assert_eq!(
        registrar.registered().len(),
        4,
        "OCR is registered"
    );
}

#[test]
fn serialized_action_ids_and_snapshot_shape_match_the_frontend_contract() {
    let (_db, _registrar, controller) = controller();
    let value = serde_json::to_value(controller.snapshot().unwrap()).unwrap();
    let actions = value["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["action"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        actions,
        [
            "translate_selection",
            "translate_input",
            "translate_clipboard",
            "ocr_translate",
        ]
    );
    for entry in value["entries"].as_array().unwrap() {
        assert!(entry.get("registration_state").is_some());
        assert!(entry.get("registration_error").is_some());
    }
}

#[test]
fn recording_flag_is_ephemeral_and_single_action() {
    let (_db, _registrar, controller) = controller();
    controller
        .recording_begin(ShortcutAction::Clipboard)
        .unwrap();
    assert!(controller.is_recording());
    let snapshot = controller.snapshot().unwrap();
    assert!(snapshot.entry(ShortcutAction::Clipboard).unwrap().recording);
    assert!(!snapshot.entry(ShortcutAction::Selection).unwrap().recording);
    assert_eq!(
        snapshot.revision, 0,
        "recording does not mutate persisted revision"
    );

    controller.recording_end();
    assert!(!controller.is_recording());
    assert!(controller
        .snapshot()
        .unwrap()
        .entries
        .iter()
        .all(|entry| !entry.recording));
    assert!(controller.recording_begin(ShortcutAction::Ocr).is_ok());
}

#[test]
fn conflict_requires_override_then_swaps_combos() {
    let (_db, _registrar, controller) = controller();
    let initial = controller.snapshot().unwrap();

    assert_eq!(
        controller.save(
            ShortcutAction::Selection,
            "ctrl+space",
            initial.revision,
            None,
        ),
        Err(ShortcutError::Conflict {
            action: ShortcutAction::Input,
        }),
    );

    let saved = controller
        .save(
            ShortcutAction::Selection,
            "ctrl+space",
            initial.revision,
            Some(ShortcutAction::Input),
        )
        .unwrap();
    assert_eq!(saved.revision, 1);
    assert_eq!(saved.combo(ShortcutAction::Selection), Some("Ctrl+Space"));
    assert_eq!(saved.combo(ShortcutAction::Input), Some("Alt+Space"));
}

#[test]
fn conflict_check_is_action_specific_and_revision_guarded() {
    let (_db, _registrar, controller) = controller();
    assert_eq!(
        controller
            .check_conflict(ShortcutAction::Selection, "ctrl+space", 0)
            .unwrap(),
        Some(ShortcutAction::Input)
    );
    assert_eq!(
        controller
            .check_conflict(ShortcutAction::Input, "ctrl+space", 0)
            .unwrap(),
        None,
        "an action does not conflict with its own binding"
    );
    controller
        .save(ShortcutAction::Clipboard, "Ctrl+Alt+C", 0, None)
        .unwrap();
    assert_eq!(
        controller.check_conflict(ShortcutAction::Selection, "Ctrl+K", 0),
        Err(ShortcutError::StaleRevision {
            expected: 0,
            actual: 1,
        })
    );
}

#[test]
fn stale_revision_cannot_overwrite_newer_snapshot() {
    let (_db, _registrar, controller) = controller();
    let initial = controller.snapshot().unwrap();
    controller
        .save(
            ShortcutAction::Clipboard,
            "Ctrl+Alt+C",
            initial.revision,
            None,
        )
        .unwrap();

    assert_eq!(
        controller.save(ShortcutAction::Input, "Ctrl+Alt+I", initial.revision, None),
        Err(ShortcutError::StaleRevision {
            expected: initial.revision,
            actual: initial.revision + 1,
        }),
    );
}

#[test]
fn registrar_failure_rolls_back_database_and_revision() {
    let (db, registrar, controller) = controller();
    let before = controller.snapshot().unwrap();
    let rows_before = shortcut_db::load(&db).unwrap();
    let registered_before = registrar.registered();
    registrar.fail_next();

    assert!(matches!(
        controller.save(
            ShortcutAction::Clipboard,
            "Ctrl+Alt+C",
            before.revision,
            None
        ),
        Err(ShortcutError::RegistrationFailed { .. })
    ));
    assert_eq!(controller.snapshot().unwrap(), before);
    assert_eq!(shortcut_db::load(&db).unwrap(), rows_before);
    assert_eq!(registrar.registered(), registered_before);
}

#[test]
fn database_failure_restores_old_os_registrations_and_revision() {
    let (db, registrar, controller) = controller();
    let before = controller.snapshot().unwrap();
    let registered_before = registrar.registered();
    db.with_conn(|conn| {
        conn.execute_batch(
            "CREATE TRIGGER reject_shortcut_change
             BEFORE INSERT ON shortcuts
             WHEN NEW.keys = 'Ctrl+Alt+C'
             BEGIN
                 SELECT RAISE(ABORT, 'injected shortcut write failure');
             END;",
        )?;
        Ok(())
    })
    .unwrap();

    assert!(matches!(
        controller.save(ShortcutAction::Clipboard, "Ctrl+Alt+C", 0, None),
        Err(ShortcutError::DatabaseFailed { .. })
    ));
    assert_eq!(controller.snapshot().unwrap(), before);
    assert_eq!(registrar.registered(), registered_before);
}

#[test]
fn startup_restores_persisted_bindings_instead_of_overwriting_them() {
    let db = database();
    shortcut_db::ensure_defaults(&db).unwrap();
    let mut persisted = shortcut_db::load(&db).unwrap();
    persisted
        .iter_mut()
        .find(|binding| binding.action == ShortcutAction::Clipboard)
        .unwrap()
        .combo = "Ctrl+Alt+C".into();
    shortcut_db::replace_all(&db, &persisted).unwrap();

    let registrar = Arc::new(FakeRegistrar::default());
    let controller = ShortcutController::new(db, registrar.clone()).unwrap();
    assert_eq!(
        controller
            .snapshot()
            .unwrap()
            .combo(ShortcutAction::Clipboard),
        Some("Ctrl+Alt+C")
    );
    assert!(registrar
        .registered()
        .contains(&(ShortcutAction::Clipboard, "Ctrl+Alt+C".into())));
}

#[test]
fn reset_defaults_is_revision_guarded_and_atomic() {
    let (_db, _registrar, controller) = controller();
    let changed = controller
        .save(ShortcutAction::Clipboard, "Ctrl+Alt+C", 0, None)
        .unwrap();
    let reset = controller.reset_defaults(changed.revision).unwrap();
    assert_eq!(reset.revision, changed.revision + 1);
    assert_eq!(
        reset.combo(ShortcutAction::Clipboard),
        Some("Ctrl+Alt+Space")
    );
}

#[test]
fn database_replace_is_all_or_nothing_and_rejects_incomplete_maps() {
    let db = database();
    shortcut_db::ensure_defaults(&db).unwrap();
    let before = shortcut_db::load(&db).unwrap();
    let incomplete = before[..3].to_vec();
    assert!(shortcut_db::replace_all(&db, &incomplete).is_err());
    assert_eq!(shortcut_db::load(&db).unwrap(), before);
}
