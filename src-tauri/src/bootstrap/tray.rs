//! System tray (R2b Surface 04) — build/refresh/menu handling (moved verbatim
//! from `lib.rs` in refactor P3.1).

use std::sync::Arc;

use tauri::menu::MenuEvent;
use tauri::Emitter;
use tauri::Manager;

use crate::bootstrap::state::{require_database, AppState};
use crate::db::providers::{self as db_providers};
use crate::handle_switch_provider;

type DbErr = crate::db::DbError;

/// rev-5-4: build the tray for the FIRST time (registers `"main-tray"`).
/// Subsequent updates go through `refresh_tray` → `build_tray_menu` +
/// `tray.set_menu(...)` so we never register a duplicate tray id.
/// Called once from `setup()`. Built last so a tray-init failure does not
/// block DB/keystore/window setup; the caller logs and continues on `Err`.
pub(crate) fn build_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
    // P1-2: the tray data readers + menu builder are `async`. `build_tray` runs
    // exactly once from `setup()` (sync, on the main thread, before the runtime
    // serves commands), so a SINGLE `block_on` driving both awaits is safe here
    // — it cannot nest inside an async worker thread the way the tray refresh
    // path can. This is the ONLY legitimate `block_on` in the tray path.
    let (menu, status) = tauri::async_runtime::block_on(async {
        let menu = build_tray_menu(app).await?;
        let status = read_primary_status(app).await;
        Ok::<_, tauri::Error>((menu, status))
    })?;
    TrayIconBuilder::with_id("main-tray")
        .icon(
            app.default_window_icon()
                .cloned()
                .expect("default window icon"),
        )
        .menu(&menu)
        .tooltip(status)
        .show_menu_on_left_click(false)
        .on_menu_event(handle_tray_menu_event)
        .on_tray_icon_event(|tray, event| {
            // Double-click on the icon surfaces the main window for
            // discoverability (macOS left-click opens the menu by default;
            // DoubleClick is documented as Windows-only but harmless to match).
            if let TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } = event
            {
                let app = tray.app_handle();
                let _ = surface_main(app);
            }
        })
        .build(app)?;
    Ok(())
}

/// rev-5-4: build ONLY the menu (reusable by build_tray + refresh_tray). Returns
/// the full menu with the fresh provider list + status item text.
///
/// P1-2: `async fn` — awaits the async DB readers instead of nesting `block_on`.
async fn build_tray_menu(app: &tauri::AppHandle) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};

    // Quick actions group.
    let sel = MenuItem::with_id(
        app,
        "tray.translate-selection",
        "Translate Selection",
        true,
        None::<&str>,
    )?;
    let clip = MenuItem::with_id(
        app,
        "tray.translate-clipboard",
        "Translate Clipboard",
        true,
        None::<&str>,
    )?;
    let sep1 = PredefinedMenuItem::separator(app)?;

    // Switch Provider submenu: built from the db at menu-build time;
    // refresh_tray() rebuilds it after provider mutations.
    let enabled = read_enabled_providers(app).await;
    let switch_sub = build_switch_provider_submenu(app, &enabled)?;
    let provider_status = MenuItem::with_id(
        app,
        "tray.provider-status",
        read_primary_status(app).await,
        false,
        None::<&str>,
    )?;
    let sep2 = PredefinedMenuItem::separator(app)?;

    let ocr = MenuItem::with_id(app, "tray.ocr-capture", "OCR Translate", true, None::<&str>)?;
    let history = MenuItem::with_id(app, "tray.history", "History", true, None::<&str>)?;
    let sep3 = PredefinedMenuItem::separator(app)?;

    // Navigation + system group.
    let check_updates =
        MenuItem::with_id(app, "tray.check-updates", "Check for Updates…", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "tray.settings", "Settings", true, None::<&str>)?;
    let sep4 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "tray.quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &sel,
            &clip,
            &sep1,
            &switch_sub,
            &provider_status,
            &sep2,
            &ocr,
            &history,
            &sep3,
            &check_updates,
            &settings,
            &sep4,
            &quit,
        ],
    )?;
    Ok(menu)
}

/// Build the Switch Provider submenu from the given `(uuid, name)` pairs. Each
/// item id encodes the uuid: `tray.switch-<uuid>`. Returns a Submenu.
///
/// P1-2: the DB read is no longer performed here — the caller (an async fn)
/// reads the providers via [`read_enabled_providers`] and passes the slice in,
/// so this builder stays sync and `block_on`-free.
fn build_switch_provider_submenu(
    app: &tauri::AppHandle,
    enabled: &[(String, String)],
) -> tauri::Result<tauri::menu::Submenu<tauri::Wry>> {
    use tauri::menu::{MenuItem, SubmenuBuilder};
    let mut sub = SubmenuBuilder::new(app, "Switch Provider");
    for (uuid, name) in enabled {
        let item = MenuItem::with_id(app, format!("tray.switch-{uuid}"), name, true, None::<&str>)?;
        sub = sub.item(&item);
    }
    sub.build()
}

/// Read (uuid, name) for enabled providers. Best-effort: returns empty on db
/// error.
///
/// P1-2: this is an `async fn` that drives the blocking DB read via
/// `spawn_blocking().await`. It MUST NOT use `block_on(spawn_blocking(...))`
/// because it is awaited from async command handlers — nesting `block_on`
/// inside the async runtime risks a runtime panic ("Cannot start a runtime from
/// within a runtime"). The single legitimate `block_on` caller is `build_tray`,
/// which runs once in `setup()` (sync, before the runtime serves commands).
async fn read_enabled_providers(app: &tauri::AppHandle) -> Vec<(String, String)> {
    use tauri::Manager;
    let app_state = app.state::<Arc<AppState>>().inner().clone();
    match tauri::async_runtime::spawn_blocking(move || {
        let _gate = app_state.data_gate.read();
        let db = require_database(&app_state, &_gate)?;
        db.with_conn(|conn| {
            let list = db_providers::list(conn)?;
            Ok(list
                .into_iter()
                .filter(|p| p.enabled)
                .map(|p| (p.uuid, p.name))
                .collect::<Vec<_>>())
        })
        .map_err(|e: DbErr| e.to_string())
    })
    .await
    {
        Ok(Ok(v)) => v,
        _ => Vec::new(),
    }
}

/// Read the primary provider name for the status item. Falls back to
/// "No provider".
///
/// P1-2: `async fn` driving the blocking DB read via `spawn_blocking().await`
/// (see [`read_enabled_providers`]). No `block_on`.
async fn read_primary_status(app: &tauri::AppHandle) -> String {
    use tauri::Manager;
    let app_state = match app.try_state::<Arc<AppState>>() {
        Some(s) => s.inner().clone(),
        None => return "No provider".into(),
    };
    match tauri::async_runtime::spawn_blocking(move || {
        let _gate = app_state.data_gate.read();
        let db = match require_database(&app_state, &_gate) {
            Ok(d) => d,
            Err(_) => return "No provider".to_string(),
        };
        let selection = db.with_conn(|conn| db_providers::read_active_selection(conn));
        match selection {
            Ok(sel) => match sel.primary {
                Some(uuid) => {
                    let name = db
                        .with_conn(|conn| db_providers::get(conn, &uuid))
                        .ok()
                        .map(|p| p.name);
                    name.unwrap_or_else(|| "Unknown provider".into())
                }
                None => "No provider".into(),
            },
            Err(_) => "No provider".into(),
        }
    })
    .await
    {
        Ok(s) => s,
        Err(_) => "No provider".into(),
    }
}

/// Refresh the tray menu + status after a provider mutation. Called from the
/// eight provider mutation command handlers (P1-5) via `refresh_tray_if_available`.
///
/// rev-5-4: refresh the EXISTING `"main-tray"` in place — rebuild the menu +
/// re-set the status tooltip via `app.tray_by_id("main-tray")`. Rebuilding from
/// scratch via the setup-time builder would register a DUPLICATE tray icon
/// (Tauri panics on duplicate id). Instead, fetch the existing tray and update
/// its menu + tooltip. Errors are PROPAGATED so the wrapper can log them.
///
/// P1-2 (R2-A): if the tray does not exist yet, this is a NO-OP. The
/// setup-time first-build helper nests a single legitimate blocking drive of the
/// runtime, but that is safe ONLY in `setup()` (sync, on the main thread, before
/// the runtime serves commands). Calling it from here — an `async fn` awaited on
/// a runtime worker thread — would nest that blocking drive inside the async
/// runtime and risk a panic ("Cannot start a runtime from within a runtime").
/// The tray is built exactly once in `setup()`; a refresh finding no tray has
/// nothing to update, so it returns `Ok(())`. The tray will be present on the
/// next launch / setup run.
pub async fn refresh_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let menu = build_tray_menu(app).await?;
        tray.set_menu(Some(menu))?;
        tray.set_tooltip(Some(&read_primary_status(app).await))?;
        Ok(())
    } else {
        // P1-2: Do NOT reach for the setup-time tray builder here — it nests a
        // runtime blocking drive that is unsafe inside this async context. The
        // tray is built once in setup(); if it is absent, there is nothing to
        // refresh yet.
        log::debug!("refresh_tray: main-tray not found, skipping refresh");
        Ok(())
    }
}

/// rev-9-3: best-effort tray refresh after a provider mutation. Wraps
/// `refresh_tray` (which returns `tauri::Result<()>`) so a tray rebuild failure
/// (e.g. tray not yet built during startup) NEVER turns a successful provider
/// write into an error.
///
/// P1-2: `async fn` — awaits [`refresh_tray`]. Callers in async command
/// handlers `.await` this directly; the SYNC `handle_switch_provider` (runs in
/// `spawn_blocking`) detaches it via `tauri::async_runtime::spawn`.
pub async fn refresh_tray_if_available(app: &tauri::AppHandle) {
    if let Err(e) = refresh_tray(app).await {
        log::warn!("tray refresh failed: {e}");
    }
}

/// Show + focus the main (settings) window. Shared by the tray menu handlers,
/// the tray double-click and the single-instance relaunch. Returns the window
/// so callers can scope their `navigate` emits to it (not broadcast).
pub(crate) fn surface_main(app: &tauri::AppHandle) -> Option<tauri::WebviewWindow> {
    let w = app.get_webview_window("main")?;
    let _ = w.show();
    let _ = w.set_focus();
    Some(w)
}

/// Menu item handler. Each arm matches a `with_id` string from [`build_tray_menu`].
///
/// The translation entry points emit a `tray-action` event that the main window
/// forwards (its listener invokes the matching backend command). The
/// `tray.switch-<uuid>` arm runs the SYNC `handle_switch_provider` wrapper inside
/// a `spawn_blocking` (rev-18-1/rev-20-4: offload the SYNC SQLite I/O). Settings
/// shows the main window + emits a real `SettingsSection` value (rev-6-3).
fn handle_tray_menu_event(app: &tauri::AppHandle, event: MenuEvent) {
    let id = event.id().as_ref();
    if let Some(uuid) = id.strip_prefix("tray.switch-") {
        // P1-5 + rev-5-4: set this provider as the sole primary, then refresh
        // the tray. On failure the write tx rolled back (old primary preserved);
        // handle_switch_provider surfaces the error in the tray tooltip.
        let app_state = app.state::<Arc<AppState>>().inner().clone();
        // R2-B (P1-3 residual): allocate the switch revision in the SYNC menu
        // callback BEFORE spawn_blocking, so revision order = click order
        // regardless of OS thread scheduling. The pre-allocated `rev` is passed
        // into the spawned closure (the core no longer calls begin_switch itself).
        let rev = app_state.tray.lock().begin_switch();
        let app_clone = app.clone();
        let uuid_owned = uuid.to_string();
        tauri::async_runtime::spawn_blocking(move || {
            let _ = handle_switch_provider(&app_clone, &app_state, &uuid_owned, rev);
        });
        return;
    }
    match id {
        "tray.translate-selection" => {
            let _ = app.emit("tray-action", "translate-selection");
        }
        "tray.translate-clipboard" => {
            let _ = app.emit("tray-action", "translate-clipboard");
        }
        "tray.ocr-capture" => {
            let _ = app.emit("tray-action", "ocr-capture");
        }
        "tray.history" => {
            if let Some(w) = surface_main(app) {
                let _ = w.emit("navigate", "history");
            }
        }
        "tray.check-updates" => {
            // R5: open the settings Updater section (it runs a fresh check on
            // mount) rather than firing the network call from the menu thread.
            if let Some(w) = surface_main(app) {
                let _ = w.emit("navigate", "updater");
            }
        }
        "tray.settings" => {
            if let Some(w) = surface_main(app) {
                // rev-6-3: navigate value is a real SettingsSection union member,
                // NOT the generic "settings" string the type rejects.
                let _ = w.emit("navigate", "provider-center");
            }
        }
        "tray.quit" => {
            app.exit(0);
        }
        _ => {}
    }
}
