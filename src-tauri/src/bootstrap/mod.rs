//! Startup decomposition (refactor P3.1).
//!
//! The former 1900-line `lib.rs` split by startup phase:
//! - [`state`] — the shared `Session` / `AppState` types + DB gate helpers.
//! - [`readiness`] — startup readiness reducer, hardened client/keystore
//!   builders, preset-endpoint fail-closed gate, keystore-recovery cleanup.
//! - [`hotkeys`] — the global-shortcut entry points (selection + input).
//! - [`tray`] — system-tray build/refresh/menu handling.
//! - [`setup`] — the Tauri `setup()` stages (session, data layer, shortcuts,
//!   kernel, tray, autoshow testability hooks).
//!
//! `lib.rs` re-exports the crate-facing names so `crate::AppState`,
//! `crate::refresh_tray_if_available`, … keep resolving unchanged.

pub mod hotkeys;
pub mod readiness;
pub mod setup;
pub mod state;
pub mod tray;
