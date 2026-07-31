# Phase 4: Windows Parity + Cross-Platform Packaging — Implementation Plan (rev 11)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`) syntax for tracking.

**Status:** rev 11 — fixes round-10 review issues (three P1s, all plan-compilability
/ correctness in Task 2b). (a) **AX test owner is cfg'd** — a single `()` does NOT
typecheck on Windows (`OwnerHwnd` is `*mut c_void` there), so the AX test cfgs the
owner value (`null_mut()` on Win / `()` else); the AX-Some path short-circuits
before the owner is used, so `null_mut` is safe there. The on_hotkey example's
`Err(e)` is now logged (no unused-variable warning). (b) **FSM references NO Win32
symbol** — `submit` takes `text_fmt`/`dib_fmt` as params and `ClipOps` supplies them
via `text_format()`/`dib_format()` (prev rev hard-coded CF_UNICODETEXT/CF_DIBV5,
which don't exist on macOS); module path is `crate::clipboard::fsm` and `submit`/
`restore_with` are `pub(super)`. (c) **three-layer split** — `build_blobs` (pure
bytes), generic `restore_with<C>` (does BOTH allocs + submit, so the
second-alloc-fail leak is testable), non-generic `restore_snapshot`; the remedial
`empty` failure is surfaced as `RestoreError::SetPartial` (honest "may contain text
only") instead of `let _ =`. Added `first_alloc_fails`, `second_alloc_fails`,
`cleanup_empty_fails` fake tests.

**Goal:** Windows builds (real `atomic_replace` + ACL), correct signing (macOS notarization via cert-import; Windows Authenticode via PFX/signtool — distinct from the updater key), and a clean CI matrix (arm64 + Intel macOS via `macos-15-intel`, Windows).

**Facts verified (round-2):** `atomic_replace` non-mac returns Err (keystore.rs:444 stub). Tauri updater keys (`TAURI_SIGNING_PRIVATE_KEY*`) sign the updater bundle, NOT Windows installers — Authenticode needs a PFX import + `signtool` (or Tauri's `signCommand`/`certificateThumbprint`). Tauri macOS signing: base64 cert → keychain import → `APPLE_SIGNING_IDENTITY` env. `macos-13` retired 2025-12-04. Custom `invoke_handler` commands are reachable from ALL local windows by default; per-window restriction requires defining them as app commands in a manifest + scoping per capability.

---

## Task 1: Windows atomic_replace (real, not the stub)

**Files:** `src-tauri/src/keystore.rs`, `src-tauri/Cargo.toml`

- [ ] Replace the `#[cfg(not(target_os = "macos"))] fn atomic_replace(...) -> Err("not implemented")` STUB with platform-specific real impls:
  - `#[cfg(target_os = "windows")]`: `MoveFileExW` (first-create, `MOVEFILE_REPLACE_EXISTING`) if dst absent, else `ReplaceFileW` (update). ADD `windows-sys` feature `Win32_Storage_FileSystem` (NOT currently in Cargo.toml — the windows-sys dep currently has only `Win32_System_DataExchange` + `Win32_Foundation`; must add `Win32_Storage_FileSystem` for ReplaceFileW/MoveFileExW).
  - `#[cfg(not(any(target_os = "macos", target_os = "windows")))]`: keep the Err stub.
- [ ] Tests (Windows runner, Task 5 CI): (a) first-create (MoveFileExW path, dst absent) — keystore.json created; (b) update-replace (ReplaceFileW path, dst exists) — content replaced atomically, no half-write observable; (c) failure path — canonical keystore.json stays intact if the tmp write fails. Can't run locally on macOS.

## Task 2: Windows file/dir ACL via SetNamedSecurityInfoW

**Files:** `src-tauri/src/keystore.rs`, `src-tauri/Cargo.toml`

- [ ] windows-sys features: add `Win32_Security_Authorization`, `Win32_Security`.
- [ ] Real `set_file_perms` (Windows): `SetNamedSecurityInfoW` with a DACL of one explicit ACE for the current-user SID (GENERIC_ALL, incl. delete for ReplaceFileW) + `PROTECTED_DACL_SECURITY_INFORMATION` (block inheritance). NOT icacls. Apply to dir (on `new`) + file (on `store_locked`).
- [ ] Windows-only test (CI): verify via `GetNamedSecurityInfoW` the DACL has exactly one explicit user ACE, full control, and is protected.
- [ ] Commit.

## Task 2b: Windows compound clipboard restore (§B image promise)

**Files:** `src-tauri/src/clipboard.rs`, `src-tauri/src/clipboard/fsm.rs` (new,
always-compiled private submodule — `ClipOps` trait, `OpenClip` guard, `submit`,
in-module `#[cfg(test)]` fake tests), `src-tauri/src/selection.rs`,
`src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`, `src-tauri/tests/clipboard_win.rs`
(new, real Windows CI integration test only), `src-tauri/tests/selection_engine.rs`
(update the one AX test call to pass `owner: ()`).

Windows mirrors the macOS compound write (one `EmptyClipboard`, both formats in one
open window). "All-or-nothing" here means: **conversion-phase failures never touch
the clipboard**; `EmptyClipboard` + the two `SetClipboardData` calls are NOT a
transaction — if the second `SetClipboardData` fails we must explicitly re-empty so
the clipboard is left with NEITHER format, matching the macOS single-item semantics
(both present, or both absent).

**Win32 protocol (verified against MS docs, refs at end of task):**

- [ ] **HWND owner — MUST be non-NULL, AND it must be owned by a thread that runs a
      message loop.** `EmptyClipboard` assigns ownership to the window that has the
      clipboard open. If `OpenClipboard(NULL)` was used, `EmptyClipboard` succeeds
      but sets the owner to NULL, which makes `SetClipboardData` fail. (Verified:
      EmptyClipboard Remarks — "If the application specifies a NULL window handle
      when opening the clipboard, EmptyClipboard succeeds but sets the clipboard
      owner to NULL. Note that this causes SetClipboardData to fail.") **Beyond the
      non-NULL requirement:** the owner receives `WM_DESTROYCLIPBOARD` whenever ANY
      other app later empties the clipboard (verified: WM_DESTROYCLIPBOARD — "Sent
      to the clipboard owner when a call to the EmptyClipboard function empties the
      clipboard. A window receives this message through its WindowProc function.").
      This applies even with eager rendering (we never pass NULL to
      SetClipboardData). Window messages are delivered on the owning thread, so the
      owner HWND MUST belong to a thread with a running message pump — otherwise a
      cross-thread SendMessage from the emptying app can block.
      **Decision: reuse the existing Tauri main window's HWND.** It is created on
      Tauri's event-loop thread (which already pumps messages) and is stable for the
      app lifetime. If the main window is unavailable, return Err (best-effort: no
      restore). Do NOT create a message-only window on an arbitrary async-runtime
      worker and cache its HWND in a `OnceCell` (that thread has no message loop and
      may exit, leaving a stale owner).

- [ ] **Callchain + HWND threading (round-8 review P1: the HWND decision must land
      in the actual data flow; round-9 review P1: the example must compile).**
      Today the path is `lib.rs::on_hotkey` → `selection::capture_selection(timeout)`
      → unit `OsClipboard` → `clipboard::restore_snapshot(text, image)`, with no
      `AppHandle`/`Window`/HWND anywhere. Changes:
      - **`selection.rs`**: `OsClipboard` gains a Windows-only owner field.
        `capture_selection` / `capture_selection_with_ax` gain a Windows-only
        `owner: windows_sys::Win32::Foundation::HWND` param threaded into
        `OsClipboard { #[cfg(windows)] owner }`. **Keep the cross-platform AX test
        helper's signature unchanged** (see test-migration note below):
        ```rust
        // selection.rs
        #[cfg(target_os = "windows")]
        pub type OwnerHwnd = windows_sys::Win32::Foundation::HWND; // *mut c_void
        #[cfg(not(target_os = "windows"))]
        pub type OwnerHwnd = (); // unit placeholder so the cross-platform test path compiles

        struct OsClipboard {
            #[cfg(target_os = "windows")]
            owner: OwnerHwnd,
        }
        // Both targets take `owner` so the SAME signature compiles everywhere; on
        // non-Windows it's a unit () and ignored. This keeps the AX unit test
        // (tests/selection_engine.rs:204 `capture_selection_with_ax(ax, 1)`) working
        // WITHOUT a signature change — pass `()` as the owner on non-Windows. The
        // macOS `restore_snapshot` ignores the extra arg via cfg.
        #[cfg(target_os = "windows")]
        pub fn capture_selection(timeout_ms: u64, owner: OwnerHwnd) -> Result<Capture, String> { … }
        #[cfg(not(target_os = "windows"))]
        pub fn capture_selection(timeout_ms: u64, _owner: OwnerHwnd) -> Result<Capture, String> { … }
        // capture_selection_with_ax mirrors this (takes owner, threads into OsClipboard).
        ```
        `OsClipboard::restore_snapshot` passes `self.owner` to the Windows
        `clipboard::restore_snapshot`. The `ClipboardLike` trait itself is
        UNCHANGED — `OsClipboard` already implements it; only its concrete fields
        and the `restore_snapshot` body differ by cfg. `selection_engine::capture`
        (the pure FSM, unit-tested with a Fake) is untouched.
      - **`lib.rs::on_hotkey`** — the `async move {}` block returns `()`, so `?` is
        NOT usable there (round-9 review P1). Use a `match` to fold the HWND-resolution
        error into the `captured: Result<_, String>` (no `owner_from_tauri` helper —
        the `.0` field access is inlined at the call site, so no `windows`-crate path
        is ever named in our code):
        ```rust
        // selection capture under the mutex, in on_hotkey's async block:
        let (x, y, captured) = {
            let _g = state.gen.selection_lock();
            let pos = cursor::position();
            // Windows: owner HWND from the main webview window; `.hwnd()` is
            // #[cfg(windows)] and returns windows-crate HWND (newtype HWND(*mut c_void));
            // `.0` is the raw *mut c_void == windows-sys HWND. Non-Windows: pass ().
            #[cfg(target_os = "windows")]
            let owner = match app2
                .get_webview_window("main")
                .ok_or_else(|| "main window unavailable".to_string())
                .and_then(|w| w.hwnd().map(|h| h.0).map_err(|e| e.to_string()))
            {
                Ok(h) => h,
                Err(e) => {
                    // best-effort: log + bail this trigger (no valid owner → no restore).
                    // `e` MUST be used (else unused-variable); log it, do not silently drop.
                    log::warn!("clipboard restore skipped: no owner HWND ({e})");
                    return;
                }
            };
            #[cfg(not(target_os = "windows"))]
            let owner = ();
            let cap = selection::capture_selection(800, owner);
            (pos.0, pos.1, cap)
        };
        ```
        (On Windows, a HWND-resolution failure logs `e` and `return`s — best-effort,
        no restore, since we have no valid owner. This keeps the block `()->()` clean
        and uses `e` so there's no unused-variable warning.)
      - **windows-crate HWND → windows-sys HWND (round-8/9 review).** `WebviewWindow::hwnd()`
        (tauri 2.11.5 `src/webview/webview_window.rs:1847`, `#[cfg(windows)]`) returns the
        HIGH-LEVEL `windows` crate's `HWND` — `windows::Win32::Foundation::HWND`, a newtype
        `pub struct HWND(pub *mut c_void)` (tauri 2.11.5 depends on `windows` 0.61). Our code
        calls `windows-sys` 0.59, whose `HWND` is `*mut c_void`. Same pointee type, so the
        conversion is the inline `.0` access shown above. We do NOT name the `windows` crate
        anywhere, do NOT add it as a dependency, and do NOT introduce an `owner_from_tauri`
        helper (round-9: that helper would have to live in clipboard.rs and name the `windows`
        crate type, which we don't depend on).
      - **`translate_clipboard`** (lib.rs:142) does NOT call `restore_snapshot` — it only
        reads text — so its `capture_selection`/clipboard call needs no owner threading.
      - **`clipboard.rs`**: the Windows `restore_snapshot` signature becomes
        `pub fn restore_snapshot(owner: OwnerHwnd, text, image) -> Result<(), String>` — a
        NON-generic public wrapper (round-9 review P1: do not make the public fn generic).
        It builds the text/DIB blobs (preflight + BGRA), constructs the real `Win32ClipOps`
        adapter (which stores `owner`), and calls the INTERNAL generic `restore_with(ops, …)`.
        See the FSM bullet for the split. macOS `restore_snapshot` signature unchanged.
      - **Test migration (round-10 review P1: a single `()` owner does NOT compile on
        Windows — `OwnerHwnd` is `*mut c_void` there, so `()` is a type mismatch on Win CI).**
        `tests/selection_engine.rs:204` calls `capture_selection_with_ax(|| Some("ax-text".into()), 1)`.
        The AX-Some path short-circuits BEFORE the owner is ever used, so the owner value is
        irrelevant — but it must still be the right TYPE per target. cfg the value:
        ```rust
        // tests/selection_engine.rs — ax_first_short_circuits_copy_fallback
        #[cfg(target_os = "windows")]
        let owner: islandpot_lib::selection::OwnerHwnd = std::ptr::null_mut(); // unused: AX short-circuits
        #[cfg(not(target_os = "windows"))]
        let owner: islandpot_lib::selection::OwnerHwnd = ();
        let res = capture_selection_with_ax(|| Some("ax-text".into()), 1, owner).unwrap();
        ```
        (The alternative — a Windows-only extra param — was rejected because it would split
        the cross-platform AX test signature. The cfg'd value keeps one call site, type-correct
        on both targets. Note: on Windows, `null_mut()` is fine BECAUSE this test never reaches
        the clipboard — the AX reader returns Some, so `restore_snapshot` is never called.)

- [ ] **Memory: `GlobalAlloc(GMEM_MOVEABLE, len)` for EACH blob.** Verified: "A
      memory object that is to be placed on the clipboard should be allocated by
      using GlobalAlloc with the GMEM_MOVEABLE flag." Use `GlobalAlloc(GMEM_MOVEABLE,
      len)` → `GlobalLock` → `copy_nonoverlapping` → `GlobalUnlock`. After a
      successful `SetClipboardData(h)`, **ownership transfers to the system** — do
      NOT `GlobalFree` a submitted handle (the system frees it on next empty). Only
      `GlobalFree` handles that were NOT successfully submitted.

- [ ] **Build BOTH byte blobs BEFORE any clipboard/alloc call (LAYER A:
      `build_blobs`).** (a) UTF-16 NUL-terminated text: `OsStr::encode_wide().chain(Some(0))`,
      byte-len = u16_count * 2. (b) CF_DIBV5 blob: `BITMAPV5HEADER` + BGRA pixel buffer
      (see layout below). Returns `(Vec<u8>, Vec<u8>)` — pure bytes, no handle, no
      clipboard touch. If either build fails, return Err — clipboard is untouched. The
      `GlobalAlloc` of these bytes happens later in LAYER B (`restore_with`), which is
      why the second-alloc-failure path can free the first handle.

- [ ] **CF_DIBV5 pixel layout (BITMAPV5HEADER):**
      - `bV5Size` = `size_of::<BITMAPV5HEADER>()` (124)
      - `bV5Width` = width; `bV5Height` = **-(height as i32)** (negative ⇒ top-down,
        origin upper-left — matches our RGBA row-major source, no vertical flip)
      - `bV5Planes` = 1; `bV5BitCount` = 32
      - `bV5Compression` = `BI_BITFIELDS` (3) — required to honor the masks below
      - `bV5SizeImage` = width*height*4
      - `bV5RedMask` = 0x00FF0000, `bV5GreenMask` = 0x0000FF00,
        `bV5BlueMask` = 0x000000FF, `bV5AlphaMask` = 0xFF000000 (BGRA byte order in
        memory: B,G,R,A per pixel — Windows native, NOT RGBA)
      - `bV5CSType` = `LCS_sRGB`; `bV5Intent` = `LCS_GM_IMAGES`; rest zeroed
      - **Pixel conversion:** for each source RGBA pixel `(r,g,b,a)` emit BGRA bytes
        `(b,g,r,a)`. Row stride = width*4 (no padding at 32bpp). This channel swap +
        the alpha mask are what the test asserts (see test).

- [ ] **Three-layer structure (round-10 review P1): separate conversion, the alloc+
      submit state machine, and the public entry point).** The previous design started
      `submit` from pre-built handles, so it could NOT test the second `GlobalAlloc`/
      `GlobalLock` failing while the first handle was live, and the cleanup `empty`
      was `let _ =`-silenced. Split into three layers:
      ```rust
      // === clipboard::fsm — ALWAYS COMPILED, platform-neutral (no Win32 types at all) ===
      // (module path: src/clipboard/fsm.rs → reached as `super::fsm` from clipboard.rs,
      //  or `crate::clipboard::fsm` from elsewhere. `submit`/`restore_with` are
      //  `pub(super)` so clipboard.rs can call them; the trait + guard are private.)
      pub(super) trait ClipOps {
          type Handle;
          fn open(&mut self) -> Result<(), String>;     // real adapter stores owner; no HWND arg
          fn close(&mut self);
          fn empty(&mut self) -> Result<(), String>;
          // set transfers Handle ownership to the system on Ok; on Err RETURNS the handle.
          fn set(&mut self, fmt: u32, h: Self::Handle) -> Result<(), (Self::Handle, String)>;
          fn alloc(&mut self, bytes: &[u8]) -> Result<Self::Handle, String>;
          fn free(&mut self, h: Self::Handle);
          // format ids are NOT Win32 consts here — the adapter supplies them:
          fn text_format(&self) -> u32;   // real: CF_UNICODETEXT; fake: a test id
          fn dib_format(&self) -> u32;    // real: CF_DIBV5;     fake: a test id
      }
      struct OpenClip<'a, C: ClipOps> { ops: &'a mut C }
      impl<'a, C: ClipOps> OpenClip<'a, C> {
          fn empty(&mut self) -> Result<(), String> { self.ops.empty() }
          fn set(&mut self, fmt: u32, h: C::Handle) -> Result<(), (C::Handle, String)> { self.ops.set(fmt, h) }
          fn free(&mut self, h: C::Handle) { self.ops.free(h) }
      }
      impl<C: ClipOps> Drop for OpenClip<'_, C> { fn drop(&mut self) { self.ops.close(); } }

      // LAYER A: build_blobs — pure conversion, returns Vec<u8> (no handle, no clipboard).
      //   fn build_blobs(text, img) -> Result<(Vec<u8> text_utf16_nul, Vec<u8> dibv5), String>
      //   (preflight + BGRA layout; unit-tested directly, no ClipOps.)

      // LAYER B: restore_with — the alloc+submit state machine, generic over ClipOps.
      //   Covers the failure paths the prev rev missed: second-alloc fail, cleanup-empty fail.
      pub(super) fn restore_with<C: ClipOps>(
          c: &mut C, text_bytes: &[u8], dib_bytes: &[u8],
      ) -> Result<(), RestoreError> {
          // allocate text; if it fails, nothing to free yet.
          let h_text = c.alloc(text_bytes).map_err(RestoreError::Alloc)?;
          // allocate dib; on FAIL free h_text (round-10 P1: first handle leaked in prev rev).
          let h_dib = match c.alloc(dib_bytes) {
              Ok(h) => h,
              Err(e) => { c.free(h_text); return Err(RestoreError::Alloc(e)); }
          };
          // submit (open/empty/set/set); on failure both handles are already freed
          // inside submit per the ownership rules, so no double-free here. Pass through
          // whatever submit returns — including SetPartial if the remedial empty failed.
          submit(c, c.text_format(), c.dib_format(), h_text, h_dib)
      }

      // LAYER B helper: submit (open/empty/set/set) — formats are PARAMETERS (round-10
      // P1: the prev rev hard-coded CF_UNICODETEXT/CF_DIBV5, which don't exist on macOS).
      fn submit<C: ClipOps>(
          c: &mut C, text_fmt: u32, dib_fmt: u32,
          h_text: C::Handle, h_dib: C::Handle,
      ) -> Result<(), RestoreError> {
          let mut clip = match c.open() {                 // fail: free both, no guard, no close
              Ok(()) => OpenClip { ops: c },
              Err(e) => { c.free(h_text); c.free(h_dib); return Err(RestoreError::Open(e)); }
          };
          if let Err(e) = clip.empty() {                  // fail: free both via guard, no re-empty
              clip.free(h_text); clip.free(h_dib); return Err(RestoreError::Empty(e));
          }
          if let Err((h, e)) = clip.set(text_fmt, h_text) {  // text fail: free both, no half-state
              clip.free(h); clip.free(h_dib); return Err(RestoreError::Set(e));
          }
          // h_text now system-owned
          match clip.set(dib_fmt, h_dib) {                // dib ok: done; fail: re-empty + free h_dib
              Ok(()) => Ok(()),
              Err((h, e)) => {
                  // re-empty to remove orphaned text — BUT report if it fails (round-10 P1:
                  // prev rev did `let _ = clip.empty()` and still claimed "both absent").
                  match clip.empty() {
                      Ok(()) => { clip.free(h); Err(RestoreError::Set(e)) }
                      // cleanup failed: text MAY remain. Surface BOTH errors honestly.
                      Err(ce) => { clip.free(h); Err(RestoreError::SetPartial { cause: e, cleanup_err: Some(ce) }) }
                  }
              }
          }
          // clip drops → close() exactly once on every path (incl. panic)
      }
      pub(super) enum RestoreError {
          Alloc(String), Open(String), Empty(String), Set(String),
          // second set failed AND the remedial EmptyClipboard failed — text MAY be on the
          // clipboard. The public wrapper maps this to an error string that says so.
          SetPartial { cause: String, cleanup_err: Option<String> },
      }

      // === LAYER C: public, NON-generic, Windows-only ===
      // #[cfg(windows)] pub fn restore_snapshot(owner, text, image) -> Result<(), String> {
      //     let (t, d) = build_blobs(text, image)?;                 // LAYER A
      //     let mut ops = Win32ClipOps { owner };                   // real adapter (#[cfg(windows)])
      //     restore_with(&mut ops, &t, &d)                          // LAYER B
      //         .map_err(|e| e.to_string())                         // incl. the honest SetPartial text
      // }
      ```
      Why three layers: LAYER A is pure bytes → unit-tested without any clipboard/adapter.
      LAYER B is the only generic code, owns ALL handle lifetimes (both allocs + submit),
      and is where the leak/cleanup failures live — fake-testable. LAYER C is thin,
      non-generic, Windows-only. `RestoreError::SetPartial` carries the cleanup error so
      the user-visible message can honestly state "clipboard may contain text only" rather
      than silently `let _ =` it. Format ids are adapter-supplied (`text_format`/`dib_format`)
      or passed as params (`submit`), so the always-compiled module references NO Win32 symbol.

- [ ] **windows-sys 0.59 features** — verified module paths by grepping the crate
      source (`~/.cargo/registry/.../windows-sys-0.59.0/src/`), NOT by guessing.
      The dep currently has `Win32_System_DataExchange` + `Win32_Foundation`. ADD:
      - `Win32_System_Memory` — `GlobalAlloc`, `GlobalLock`, `GlobalUnlock`,
        `GlobalFree`, `GMEM_MOVEABLE`
      - `Win32_System_Ole` — **`CF_UNICODETEXT`, `CF_DIBV5`** (these format
        constants live in Ole, NOT WindowsAndMessaging)
      - `Win32_Graphics_Gdi` — `BITMAPV5HEADER`, `BI_BITFIELDS`, `LCS_GM_IMAGES`
      - `Win32_UI_ColorSystem` — `LCS_sRGB` (lives in ColorSystem, not Gdi)
      Already present, no action: `Win32_System_DataExchange` provides
      `OpenClipboard`/`EmptyClipboard`/`SetClipboardData`/`CloseClipboard`. NOTE:
      `Win32_UI_WindowsAndMessaging` is NOT needed by the APP (it reuses the Tauri
      main-window HWND — no `CreateWindowExW`/`HWND_MESSAGE`), but the real Windows
      CI test DOES use `CreateWindowExW`/`HWND_MESSAGE`/`PeekMessage`/`DispatchMessage`
      for its throwaway test owner, so add it as a test-only feature.
      (Task 2's `Win32_Security_Authorization`/`Win32_Security` are separate.)

- [ ] **Checked conversions for the FFI boundary** (mirror the macOS preflight
      added in round-7). The Win32 functions take `i32`/`u32` sizes and `isize`
      handles; a wrapping `as` cast would be UB at the boundary:
      - Reject zero width/height.
      - `i32::try_from(width)` / `i32::try_from(height)` (BITMAPV5HEADER fields are
        `LONG`/i32; bV5Height is then negated for top-down — guard against
        i32::MIN which can't be negated).
      - `bV5SizeImage` (u32): `width.checked_mul(height)?.checked_mul(4)` →
        `u32::try_from`.
      - Text blob length (usize → `usize` for GlobalAlloc is fine, but the UTF-16
        byte count must be `u16_count.checked_mul(2)` then `usize::try_from`).
      - Image byte count: `usize::try_from(total_u64)` as in the macOS preflight.
      Any conversion failure returns Err before any Win32 call (clipboard untouched).

- [ ] **Failure-injection unit tests (`fsm::tests`, run on ALL platforms).** The
      `FakeClip` forces each API to fail and the test asserts the exact call sequence.
      `FakeClip` impls `ClipOps` with `type Handle = u32`, records
      `open/empty/set/alloc/free/close`, tracks ownership ("ours" → "system" on a
      successful `set`; `free` on a system-owned handle panics = double-free detector;
      `empty` marks system-transferred handles freed). Round-10 adds the alloc +
      cleanup-empty paths:
      - `open_fails`: → 2 `free` (both pre-allocated by restore_with), zero `empty`,
        zero `set`, zero `close` (guard never built). No leak.
      - `empty_fails`: → 2 `free`, zero `set`, exactly 1 `close`. No re-empty.
      - `first_set_fails` (text): → 2 `free` (both unsubmitted — set returns h_text),
        zero `empty` after, exactly 1 `close`.
      - `second_set_fails` (dib): text set ok, dib set errs → 1 `free` (h_dib; h_text
        system-owned → double-free detector guards it), 1 remedial `empty` WHILE OPEN,
        exactly 1 `close`.
      - `success`: both sets ok → zero `free`, zero re-`empty`, exactly 1 `close`.
      - **`first_alloc_fails`** (round-10): restore_with's first `alloc(text)` errs →
        zero `free` (no handle yet), zero of everything else, no `open`. RestoreError::Alloc.
      - **`second_alloc_fails`** (round-10): first `alloc` ok, second `alloc(dib)` errs →
        EXACTLY 1 `free` (h_text, the previously-leaked handle), zero `open`/`empty`/`set`.
        RestoreError::Alloc. This is the leak the prev rev could not catch.
      - **`cleanup_empty_fails`** (round-10): text set ok, dib set errs, AND the remedial
        `empty` errs → 1 `free` (h_dib), result is `RestoreError::SetPartial { cleanup_err: Some(_) }`,
        and the test asserts the PUBLIC message contains "may contain text only" — i.e. the
        cleanup failure is NOT silenced (prev rev did `let _ = clip.empty()`).
      - Ownership bookkeeping asserts no leaks / no double-frees at end of EVERY test.
      - `close` count is `== 1` in every branch that opened (the `OpenClip` guard is the
        only closer; its `Drop` calls `ClipOps::close`, which the fake records).

- [ ] **Real Windows integration test (`tests/clipboard_win.rs`, Windows CI only).**
      This is the SUCCESS path only — the failure branches are covered by the fake
      FSM above (they can't be reliably forced against the real clipboard). It needs
      an owner HWND but has no Tauri window, so it creates a throwaway message-only
      window FOR THE TEST (not the app). Lifecycle discipline (round-9 small fix —
      "short-lived" must have an actual cleanup step):
      - Create the owner on the TEST thread: `CreateWindowExW(0, "STATIC", …,
        HWND_MESSAGE, …)`.
      - Wrap it in an RAII guard whose `Drop` calls `DestroyWindow(hwnd)` — and
        `DestroyWindow` must be called from the SAME thread that created the window
        (MS requirement). So the test is single-threaded: create → run assertions →
        destroy, all on one thread.
      - ORDERING: finish ALL clipboard reads, any re-empty, and a final
        `PeekMessage`/`DispatchMessage` pump (to drain a queued
        `WM_DESTROYCLIPBOARD` from a concurrent app so the senders don't block)
        BEFORE the guard drops and calls `DestroyWindow`. Destroying while pending
        cross-thread sends are outstanding is undefined; pump first, then destroy.
      - This is acceptable in a test (single short-lived thread) precisely because it
        is NOT the app's permanent owner (the app reuses the Tauri main window).
      1. Build a **4-color 2×2 RGBA** image (not uniform red — round-8 review: a
         uniform image can't prove row stride or top-down vs bottom-up order):
         `[(255,0,0,255), (0,255,0,255),  (0,0,255,255), (255,255,0,255)]` —
         TL=red, TR=green, BL=blue, BR=yellow.
      2. `restore_snapshot(owner, Some("hi"), Some(&img))`.
      3. Read back: assert CF_UNICODETEXT == "hi".
      4. Read CF_DIBV5, assert header (2×2, 32bpp, BI_BITFIELDS, masks), then assert
         EACH pixel position by its DISTINCT color:
         - pos(0,0)=red→BGRA (0,0,255,255), pos(1,0)=green→(0,255,0,255),
           pos(0,1)=blue→(255,0,0,255), pos(1,1)=yellow→(0,255,255,255).
         A wrong row stride OR a bottom-up flip would scramble these distinct
         values — this is the definitive stride + row-direction test (the uniform
         all-red image could not do this).
      5. Negative: invalid RGBA (len mismatch) returns Err, clipboard unchanged
         (marker survives).

- [ ] Commit.

**Refs:** [EmptyClipboard](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-emptyclipboard)
(NULL owner ⇒ SetClipboardData fails), [Clipboard Operations](https://learn.microsoft.com/en-us/windows/win32/dataxchg/clipboard-operations)
(GMEM_MOVEABLE; ownership transfer; system frees CF_UNICODETEXT/CF_DIBV5 via
GlobalFree), [BITMAPV5HEADER](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapv5header)
(negative height ⇒ top-down; BI_BITFIELDS honors masks), [GlobalAlloc](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-globalalloc).

## Task 3: Release bundle config — CSP + env-driven signing

**Files:** `src-tauri/tauri.conf.json`

- [ ] CSP (production): NO wildcard `https:` / `ws:`. Provider HTTP goes through Rust reqwest, NOT the WebView. The WebView needs Tauri IPC (`ipc:` + `http://ipc.localhost`). Production: `"default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; connect-src 'self' ipc: http://ipc.localhost"`. **`devCsp`**: same + `ws://localhost:1420` for Vite HMR. [Tauri CSP](https://v2.tauri.app/security/csp/)
- [ ] Signing: drive purely via env vars (NO `${APPLE_SIGNING_IDENTITY}` in JSON). Keep `bundle.macOS.minimumSystemVersion: "11.0"`.
- [ ] Commit.

## Task 4: Capabilities (decided: per-window via AppManifest::commands)

**Files:** `src-tauri/build.rs`, `src-tauri/capabilities/`

- [ ] **Exact build.rs code**:
```rust
fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(
                tauri_build::AppManifest::new()
                    .commands(&[
                        "translate", "translate_default", "translate_clipboard",
                        "list_engines", "set_key", "delete_key", "key_status",
                        "get_settings", "set_setting", "lookup_dictionary",
                        "a11y_status", "keystore_health", "archive_keystore", "reset_keystore",
                    ]),
            ),
    )
    .expect("failed to run tauri build");
}
```
- [ ] Per-window capabilities with correct permission IDs (`allow-$command`, NO `islandpot:` prefix):
  - `capabilities/main.json` — `"permissions": ["core:default", "allow-translate", "allow-translate-default", "allow-translate-clipboard", "allow-list-engines", "allow-set-key", "allow-delete-key", "allow-key-status", "allow-get-settings", "allow-set-setting", "allow-lookup-dictionary", "allow-a11y-status", "allow-keystore-health", "allow-archive-keystore", "allow-reset-keystore"]`
  - `capabilities/popup.json` — `"permissions": ["core:default", "core:window:allow-hide"]` (popup hides via window API, not a custom command; `core:default` does NOT include `allow-hide`)
  - `capabilities/input.json` — `"permissions": ["core:default", "allow-translate-default"]` (input only calls translate_default; NOT get_settings)
  - Drop `store:default` (frontend doesn't call the store plugin directly — Rust does). Drop `opener`/`global-shortcut` (backend plugins).
- [ ] Verify each window resolves its scoped commands at runtime (exercise all three in dev).
- [ ] Commit.

## Task 5: GitHub Actions release workflow (correct signing + runner)

**Files:** `.github/workflows/release.yml`

- [ ] Matrix: `macos-latest` (arm64), **`macos-15-intel`** (NOT retired `macos-13`), `windows-latest` (x86_64-pc-windows-msvc).
- [ ] **macOS signing** (pinned, full official flow):
```yaml
- name: Import signing cert + build (macOS)
  id: mac-build
  if: startsWith(matrix.os, 'macos')
  env:
    APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
    APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
    KEYCHAIN_PASSWORD: ${{ secrets.KEYCHAIN_PASSWORD }}
    APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
    APPLE_ID: ${{ secrets.APPLE_ID }}
    APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
    APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
  run: |
    CERT_PATH="$RUNNER_TEMP/islandpot-cert.p12"
    echo "$APPLE_CERTIFICATE" | base64 --decode > "$CERT_PATH"
    security create-keychain -p "$KEYCHAIN_PASSWORD" build.keychain
    security default-keychain -s build.keychain
    security unlock-keychain -p "$KEYCHAIN_PASSWORD" build.keychain
    # Auto-lock after 3600s (official recommendation).
    security set-keychain-settings -t 3600 -u build.keychain
    security import "$CERT_PATH" -P "$APPLE_CERTIFICATE_PASSWORD" -k build.keychain -T /usr/bin/codesign
    security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$KEYCHAIN_PASSWORD" build.keychain
    # Verify the imported identity EXACTLY matches APPLE_SIGNING_IDENTITY — fail
    # the build here (clear error) rather than later with a vague codesign failure.
    # find-identity prints "  <SHA1> \"<Common Name>\""; grep -F anchors on the
    # expected identity string so a mismatch exits non-zero.
    security find-identity -v -p codesigning build.keychain > "$RUNNER_TEMP/identities.txt"
    cat "$RUNNER_TEMP/identities.txt"
    grep -F -- "$APPLE_SIGNING_IDENTITY" "$RUNNER_TEMP/identities.txt" \
      || { echo "::error::APPLE_SIGNING_IDENTITY '$APPLE_SIGNING_IDENTITY' not found among imported codesigning identities"; exit 1; }
    pnpm tauri build --target ${{ matrix.target }}

- name: Cleanup keychain + cert (macOS)
  if: always() && startsWith(matrix.os, 'macos')
  run: |
    security delete-keychain build.keychain || true
    rm -f "$RUNNER_TEMP/islandpot-cert.p12"
```
- [ ] **Windows signing** (Authenticode via `certificateThumbprint` overlay config written to a FILE — no inline JSON quoting):
```yaml
- name: Import PFX + build (Windows)
  if: matrix.os == 'windows-latest'
  env:
    WINDOWS_CERTIFICATE_PFX: ${{ secrets.WINDOWS_CERTIFICATE_PFX }}
    WINDOWS_CERTIFICATE_PASSWORD: ${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }}
  run: |
    $certPath = "$env:RUNNER_TEMP\islandpot-cert.pfx"
    [System.Convert]::FromBase64String("$env:WINDOWS_CERTIFICATE_PFX") | Set-Content $certPath -AsByteStream
    $imported = Import-PfxCertificate -CertStoreLocation Cert:\CurrentUser\My -FilePath $certPath -Password (ConvertTo-SecureString -String $env:WINDOWS_CERTIFICATE_PASSWORD -AsPlainText -Force)
    $thumbprint = $imported.Thumbprint
    # Write overlay config to a FILE (no inline JSON quoting issues).
    $config = @{ bundle = @{ windows = @{ certificateThumbprint = $thumbprint; digestAlgorithm = "sha256"; timestampUrl = "http://timestamp.digicert.com" } } }
    $configPath = "$env:RUNNER_TEMP\tauri-windows-signing.conf.json"
    $config | ConvertTo-Json -Depth 5 | Set-Content $configPath
    pnpm tauri build --target ${{ matrix.target }} --config $configPath

- name: Cleanup cert (Windows)
  if: always() && matrix.os == 'windows-latest'
  run: Remove-Item "$env:RUNNER_TEMP\islandpot-cert.pfx" -ErrorAction SilentlyContinue
```

> `TAURI_SIGNING_PRIVATE_KEY*` is the UPDATER signature only (separate plugin) —
> do NOT conflate it with Authenticode. If no Authenticode cert is configured,
> build unsigned and document the resulting SmartScreen warning in the release notes.

- [ ] Artifacts upload. Lint YAML. Commit.

## Task 6: README + E2E + final review

- [ ] README release docs (mac + Windows signing, distinct from updater key; Intel+arm64; CI tag trigger). Commit.
- [ ] Manual E2E on macOS release .app (Accessibility grant; selection AX-first + copy-fallback; input; clipboard; fallback; dict; keystore recovery; latest-wins). DMG may fail on hdiutil locally (env issue).
- [ ] Final review (opus): Windows ACL/atomic_replace correctness (can't run locally — verify the code + CI is set up to run it), CSP not breaking UI, capabilities decision documented, CI signing distinct from updater key. Merge.

---

## Self-Review
- **Round-3 fixes:** CSP tightened (no wildcard https/ws); AppManifest pinned with exact code + permission IDs; macOS signing `$RUNNER_TEMP` + `codesign:` + `if: always()` cleanup; Windows `certificateThumbprint` overlay (no `certificatePath`).
- **fsync (P2):** deferred to a hardening slice (honest).
- **No false claims:** stub atomic_replace still noted in Task 1.

## Execution Handoff
Subagent-Driven. **Execute only after the round-3 code fixes are re-reviewed + approved.**
