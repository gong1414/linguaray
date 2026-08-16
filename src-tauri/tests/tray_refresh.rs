//! Task A2 (P1-2): verify the tray refresh path does not nest `block_on`
//! inside an async command context. The tray data readers + refresh functions
//! must be fully async (`spawn_blocking().await`), NOT
//! `block_on(spawn_blocking(...))` — nesting `block_on` inside an async runtime
//! worker thread risks a runtime panic ("Cannot start a runtime from within a
//! runtime").
//!
//! This is a structural test: the nested-block_on is a runtime fragility that
//! cannot be reliably triggered from a unit test, but the structural absence of
//! `block_on` in the tray refresh path is the load-bearing guarantee.

#[test]
fn tray_refresh_path_has_no_block_on() {
    let src = include_str!("../src/bootstrap/tray.rs");
    // The four load-bearing functions in the tray refresh path. They are kept
    // async (their names are unchanged); each MUST be free of `block_on`.
    for fn_name in [
        "fn read_enabled_providers",
        "fn read_primary_status",
        "fn refresh_tray",
        "fn refresh_tray_if_available",
    ] {
        let start = src
            .find(fn_name)
            .unwrap_or_else(|| panic!("`{fn_name}` not found in lib.rs"));
        let body = &src[start..];
        // The function body ends at the next column-0 closing brace (`\n}`).
        let end = body[1..].find("\n}").map(|i| i + 1).unwrap_or(body.len());
        let fn_body = &body[..end.min(body.len())];
        assert!(
            !fn_body.contains("block_on"),
            "`{fn_name}` must NOT call block_on — nested block_on in an async \
             command context risks a runtime panic (P1-2)"
        );
    }
}

/// R2-A (P1-2 transitive): `refresh_tray` is `async fn`. Its else-branch used to
/// call `build_tray`, which internally calls `block_on`. That forms an
/// async→sync→block_on chain that can panic on a tokio worker thread
/// ("Cannot start a runtime from within a runtime"). The structural absence of a
/// `build_tray` CALL in `refresh_tray`'s body is the load-bearing guarantee — the
/// tray is built exactly once in `setup()` (the only safe `block_on` site); an
/// async refresh finding no tray is a no-op, not a "build it now" trigger.
///
/// Precision note: the existing-tray branch legitimately calls
/// `build_tray_menu` — a SEPARATE, async, block_on-free helper. A naive
/// `contains("build_tray")` would false-flag that helper (it shares the
/// `build_tray` prefix). So we mask `build_tray_menu` out first, then assert no
/// `build_tray` token remains — this targets the dangerous sync function only.
#[test]
fn refresh_tray_does_not_call_build_tray() {
    let src = include_str!("../src/bootstrap/tray.rs");
    let start = src
        .find("pub async fn refresh_tray")
        .or_else(|| src.find("async fn refresh_tray"))
        .expect("refresh_tray not found");
    let body = &src[start..];
    let end = body[1..].find("\n}").map(|i| i + 1).unwrap_or(body.len());
    let fn_body = &body[..end.min(body.len())];
    // Mask the safe async helper so its `build_tray` prefix doesn't false-flag.
    let masked = fn_body.replace("build_tray_menu", "<safe-async-menu-builder>");
    assert!(
        !masked.contains("build_tray"),
        "refresh_tray must NOT call build_tray — it contains block_on, unsafe in \
         async context (P1-2 transitive)"
    );
}
