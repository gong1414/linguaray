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
    let src = include_str!("../src/lib.rs");
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
