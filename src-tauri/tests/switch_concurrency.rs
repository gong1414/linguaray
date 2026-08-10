//! Task A3 (P1-3): concurrent provider switch guarantees last-click-wins at the DB.
//! Two rapid switch clicks (A then B) must result in primary=B in the DB.
use linguaray_lib::db::Database;
use linguaray_lib::db::readiness::DataReadiness;
use linguaray_lib::db::providers as db_providers;
use linguaray_lib::db::schema;
use linguaray_lib::tray_state::{Locale, RecordingRenderer, TrayStateController};
use std::sync::Arc;

fn fresh_app_state(dir: &tempfile::TempDir) -> Arc<linguaray_lib::AppState> {
    let db_path = dir.path().join("linguaray.db");
    let db = Database::open(&db_path).expect("Database::open");
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        schema::create_all_tables(&tx)?;
        schema::seed_singletons(&tx)?;
        tx.commit()?;
        Ok(())
    })
    .expect("create_all_tables + seed_singletons");
    let renderer = Arc::new(RecordingRenderer::default());
    Arc::new(linguaray_lib::AppState {
        db: parking_lot::RwLock::new(Some(Arc::new(db))),
        data_gate: parking_lot::RwLock::new(()),
        readiness: parking_lot::RwLock::new(DataReadiness::Ready),
        db_path,
        keystore_dir: dir.path().join("keystore"),
        settings_path: Some(dir.path().join("settings.json")),
        tray: Arc::new(parking_lot::Mutex::new(
            TrayStateController::with_renderer(renderer, Locale::En),
        )),
    })
}

#[test]
fn rapid_switch_a_then_b_results_in_b() {
    let dir = tempfile::tempdir().expect("tempdir");
    let app_state = fresh_app_state(&dir);
    let db_read = app_state
        .db
        .read()
        .clone()
        .expect("db slot Some");
    // Create two providers.
    let p1 = db_read
        .with_conn(|conn| db_providers::create(conn, "custom", "Provider A", "http://localhost:8080", None))
        .expect("create A");
    let p2 = db_read
        .with_conn(|conn| db_providers::create(conn, "custom", "Provider B", "http://localhost:8081", None))
        .expect("create B");

    // Switch to A, then immediately switch to B (simulating rapid clicks).
    // begin_switch bumps the revision for each call; the DB write checks it.
    linguaray_lib::handle_switch_provider_core(&app_state, &p1.uuid).expect("switch A");
    linguaray_lib::handle_switch_provider_core(&app_state, &p2.uuid).expect("switch B");

    let selection = db_read
        .with_conn(|conn| db_providers::read_active_selection(conn))
        .expect("read");
    assert_eq!(
        selection.primary,
        Some(p2.uuid.clone()),
        "the LAST switch (B) must win in the DB (P1-3)"
    );
}

#[test]
fn stale_revision_db_write_is_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let app_state = fresh_app_state(&dir);
    let db_read = app_state
        .db
        .read()
        .clone()
        .expect("db slot Some");
    let p1 = db_read
        .with_conn(|conn| db_providers::create(conn, "custom", "A", "http://localhost:8080", None))
        .expect("create");
    let p2 = db_read
        .with_conn(|conn| db_providers::create(conn, "custom", "B", "http://localhost:8081", None))
        .expect("create");
    let _ = p2; // second provider is only here to make the table non-empty (realistic).

    // Manually simulate the stale-revision path: capture rev for A, bump past it
    // with another begin_switch, then try A's write with the stale rev. The guard
    // must reject it so a late/stale switch cannot overwrite a newer switch's commit.
    let rev_a = app_state.tray.lock().begin_switch();
    let _rev_b = app_state.tray.lock().begin_switch(); // now switch_revision > rev_a
    let _ = _rev_b;
    // db_set_active_primary with the stale rev should fail (return an error
    // indicating the revision is stale).
    let result = linguaray_lib::db_set_active_primary(&app_state, &p1.uuid, rev_a);
    assert!(result.is_err(), "stale revision DB write must be rejected");
    let err = result.unwrap_err();
    assert!(
        err.contains("stale switch revision"),
        "error must identify the stale revision (got: {err})"
    );
    let selection = db_read
        .with_conn(|conn| db_providers::read_active_selection(conn))
        .expect("read");
    assert_eq!(
        selection.primary, None,
        "the stale write must not have committed"
    );
}
