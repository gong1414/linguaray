Archived-on: 2026-08-14 · reason: superseded by linguaray-plugin-core-design / completed, see git history

# Phase 4: Windows Parity + Cross-Platform Packaging — Implementation Plan (rev 14)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`) syntax for tracking.

**Status:** rev 14 — fixes round-13 review (2 plan P1s, both test-coverage gaps where a
fake can't reach the real adapter/builder). (a) **`alloc` leak-safety is now testable
against the REAL helper** — factor Win32 memory ops behind an injectable `GlobalMemOps`
trait; the real `alloc_global<M: GlobalMemOps>` is unit-tested in `windows::tests` with a
`FakeGlobalMem { lock_fails: true }` that asserts `free` is called on the alloc'd handle
(a `ClipOps`-level fake sees alloc as one black box and couldn't catch a missing
`GlobalFree`). Handle is pinned as a `struct Handle(pub HGLOBAL)` newtype (`.0` unwraps in
set/free). FSM `first/second_alloc_fails` retained for the FSM's own alloc-loop. (b)
**`build_blobs` cardinality now tested** — `windows::tests` unit-tests all four Option
combos → asserts exact 0/1/1/2 entry COUNT, real `CF_UNICODETEXT`/`CF_DIBV5` ids, and ORDER
`[text, dib]` (the FSM is format-agnostic, so a swap would slip through without this). Real
Windows CI test expanded from text+image-only to all four cases incl. `(None,None)`
sentinel-clear (mirrors the macOS `restore_empty_to` real-pasteboard test). Small:
corrected the last stale "Windows-only param / signature unchanged" wording (third param
on ALL targets; only its type cfg's).

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

**Files:** `src-tauri/src/clipboard.rs` (macOS + non-mac/win paths; the empty-original
`clear()` fix already landed in code), `src-tauri/src/clipboard/fsm.rs` (new,
**always-compiled platform-neutral** submodule — `ClipOps` trait, `OpenClip` guard,
`restore_with` over a 0–N format list, `RestoreError`, in-module `#[cfg(test)]` fake
tests), `src-tauri/src/clipboard/windows.rs` (new, **`#[cfg(windows)]` only** —
`build_blobs` returning `Vec<(u32, Vec<u8>)>`, `Win32ClipOps` adapter), `src-tauri/src/selection.rs`
(`OsClipboard` gains a Windows-only owner; `capture_selection`/`capture_selection_with_ax`
take `owner` on ALL targets — `()` off-Windows), `src-tauri/src/lib.rs` (`on_hotkey`
resolves the main window HWND and threads it down), `src-tauri/Cargo.toml`,
`src-tauri/tests/clipboard_win.rs` (new, real Windows CI integration test only),
`src-tauri/tests/selection_engine.rs` (cfg the AX test's owner value —
`null_mut()` on Win / `()` else — see callchain bullet).

Windows mirrors the macOS compound write (one `EmptyClipboard`, then each format's
`SetClipboardData` in one open window). "All-or-nothing" here means: **conversion/
preflight failures never touch the clipboard**, and **`EmptyClipboard` + the
`SetClipboardData` calls are NOT a transaction** — if a later `SetClipboardData`
fails, we explicitly re-empty to remove the already-submitted formats. That re-empty
SUCCEEDS ⇒ clipboard left with NEITHER format; FAILS ⇒ earlier-submitted formats MAY
remain (surfaced as `RestoreError::SetPartial`, honest "may contain partial data").
The public `restore_snapshot` takes two `Option`s, but the FSM itself operates on a
0–2-element prepared-format list — so empty / text-only / image-only / both all work
(0 = clear-only, which is the §B empty-original case).

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
      - **`selection.rs`**: `OsClipboard` gains an owner field (cfg-gated to Windows).
        `capture_selection` / `capture_selection_with_ax` gain a THIRD `owner: OwnerHwnd`
        param on ALL targets (only the TYPE differs by cfg — raw HWND on Windows, `()`
        elsewhere), threaded into `OsClipboard`. The existing AX test helper's arity
        changes too — its call site's owner value is cfg'd (see test-migration note):
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
        // BOTH targets take an `owner` param so the SAME arity compiles everywhere; the
        // TYPE differs by cfg (raw HWND on Windows, () placeholder elsewhere). The existing
        // AX unit test (tests/selection_engine.rs:204) gets a THIRD arg — its owner value is
        // cfg'd (null_mut on Win / () else); see the Test migration bullet. macOS's
        // restore_snapshot ignores the extra arg via cfg.
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
        let owner: linguaray_lib::selection::OwnerHwnd = std::ptr::null_mut(); // unused: AX short-circuits
        #[cfg(not(target_os = "windows"))]
        let owner: linguaray_lib::selection::OwnerHwnd = ();
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

- [ ] **`Win32ClipOps::alloc` leak-safety (round-12 review P1 #4), testable against the
      REAL alloc helper (round-13 review P1: a fake in `fsm::tests` can't catch the real
      adapter forgetting to `GlobalFree`).** `GlobalAlloc` can succeed and `GlobalLock`
      fail; if `alloc` returns Err without freeing, the `HGLOBAL` leaks (restore_with can't
      free a handle it never received). Factor the Win32 memory ops behind an injectable
      low-level trait so the REAL alloc helper is unit-tested with an injected lock failure:
      ```rust
      // clipboard/windows.rs — the HGLOBAL is a tuple newtype so Handle is well-defined
      // (round-13 review: the prev sketch used Handle(raw) without defining Handle).
      pub(super) struct Handle(pub windows_sys::Win32::Foundation::HGLOBAL);  // .0 unwraps in set/free

      // Injectable low-level memory ops. The real impl calls GlobalAlloc/GlobalLock/etc;
      // a fake records calls and can force GlobalLock to "fail" (return null).
      trait GlobalMemOps {
          fn alloc(&mut self, flags: u32, bytes: usize) -> *mut core::ffi::c_void; // HGLOBAL; null = fail
          fn lock(&mut self, h: *mut core::ffi::c_void) -> *mut core::ffi::c_void;  // ptr; null = fail
          unsafe fn unlock(&mut self, h: *mut core::ffi::c_void) -> i32;
          unsafe fn free(&mut self, h: *mut core::ffi::c_void);
      }
      struct RealGlobalMem;   // impl GlobalMemOps via GlobalAlloc/GlobalLock/GlobalUnlock/GlobalFree
      #[cfg(test)]
      struct FakeGlobalMem { lock_fails: bool, log: Vec<&'static str> }  // records "alloc"/"lock"/"free"

      // The REAL alloc helper, generic over GlobalMemOps — so a Windows unit test injects
      // FakeGlobalMem { lock_fails: true } and asserts `free` was called on the leaked handle.
      // The ClipOps::alloc impl delegates here with RealGlobalMem.
      fn alloc_global<M: GlobalMemOps>(m: &mut M, bytes: &[u8]) -> Result<Handle, String> {
          let raw = m.alloc(GMEM_MOVEABLE, bytes.len());
          if raw.is_null() { return Err("GlobalAlloc failed".into()); }
          // RAII: owns `raw`, Drop calls m.free(raw). Disarm ONLY on success.
          struct Guard<'a, M: GlobalMemOps>(&'a mut M, *mut core::ffi::c_void);
          impl<M: GlobalMemOps> Drop for Guard<'_, M> { fn drop(&mut self) { unsafe { self.0.free(self.1) } } }
          let g = Guard(m, raw);
          let ptr = g.0.lock(raw);
          if ptr.is_null() { return Err("GlobalLock failed".into()); } // g.Drop frees raw ✓
          unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, bytes.len()) };
          let _ = unsafe { g.0.unlock(raw) };
          std::mem::forget(g);                        // disarm: ownership → caller via Handle
          Ok(Handle(raw))
      }
      impl ClipOps for Win32ClipOps {
          type Handle = Handle;
          fn alloc(&mut self, bytes: &[u8]) -> Result<Handle, String> { alloc_global(&mut RealGlobalMem, bytes) }
          fn free(&mut self, h: Handle) { unsafe { RealGlobalMem.free(h.0) } }
          fn set(&mut self, fmt: u32, h: Handle) -> Result<(), (Handle, String)> {
              // SetClipboardData(fmt, h.0); on failure return (h, err) so caller frees h.0
              …
          }
          …
      }
      ```
      **Two layers of testing** (round-13 review):
      - `fsm::tests` KEEP `first_alloc_fails` / `second_alloc_fails` — they test the FSM's
        alloc-loop ownership (restore_with), using a Fake whose `ClipOps::alloc` is a single
        black box. These are unchanged.
      - `windows::tests` (Windows-only unit test) ADD `real_alloc_helper_frees_on_lock_fail`:
        injects `FakeGlobalMem { lock_fails: true }` into `alloc_global`, asserts the helper
        returns Err AND `FakeGlobalMem.log` contains a `free` for the alloc'd handle — i.e.
        the REAL Win32 alloc path honors the postcondition (Err ⇒ no live HGLOBAL). This is
        the test the round-12 fake could NOT provide.

- [ ] **CF_DIBV5 pixel layout (BITMAPV5HEADER)** (Windows-only, lives in the
      `#[cfg(windows)]` blob builder — see three-layer split below):
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

- [ ] **Three-layer structure + correct module placement (round-11 review P1 #2 & #4:
      the FSM must cover 0–2 formats, and the Windows-specific blob builder must NOT
      live in the always-compiled platform-neutral module).** `build_blobs` uses
      `OsStr::encode_wide`, `BITMAPV5HEADER`, `BI_BITFIELDS`, `LCS_sRGB` — all
      Windows-only — so it stays in a `#[cfg(windows)]` module. The always-compiled
      `fsm.rs` holds ONLY the ownership state machine (abstract format ids) + fake tests.
      ```rust
      // === src/clipboard/fsm.rs — ALWAYS COMPILED, platform-neutral. NO Win32 types.
      // (module path crate::clipboard::fsm. restore_with AND ClipOps are pub(super) so the
      //  windows.rs adapter can impl the trait and call restore_with; OpenClip guard private.
      //  submit is folded into restore_with's loop — no separate submit fn.)
      pub(super) trait ClipOps {
          type Handle;
          fn open(&mut self) -> Result<(), String>;     // real adapter stores owner; no HWND arg
          fn close(&mut self);
          fn empty(&mut self) -> Result<(), String>;
          // set transfers Handle ownership to the system on Ok; on Err RETURNS the handle.
          fn set(&mut self, fmt: u32, h: Self::Handle) -> Result<(), (Self::Handle, String)>;
          // alloc POSTCONDITION (round-12 review P1 #4): on Err, NO app-owned handle is
          // left live — the adapter frees any partial allocation internally before
          // returning Err (e.g. GlobalAlloc succeeded but GlobalLock failed → GlobalFree
          // the HGLOBAL first). restore_with relies on this: it cannot free a handle the
          // adapter never handed back. The REAL adapter's leak-safety is unit-tested by
          // injecting a GlobalMemOps fake that forces lock-fail — see the Win32ClipOps::alloc
          // bullet (round-13: a ClipOps-level fake can't reach into the adapter's internals).
          fn alloc(&mut self, bytes: &[u8]) -> Result<Self::Handle, String>;
          fn free(&mut self, h: Self::Handle);
      }
      struct OpenClip<'a, C: ClipOps> { ops: &'a mut C }
      impl<'a, C: ClipOps> OpenClip<'a, C> {
          fn empty(&mut self) -> Result<(), String> { self.ops.empty() }
          fn set(&mut self, fmt: u32, h: C::Handle) -> Result<(), (C::Handle, String)> { self.ops.set(fmt, h) }
          fn free(&mut self, h: C::Handle) { self.ops.free(h) }
      }
      impl<C: ClipOps> Drop for OpenClip<'_, C> { fn drop(&mut self) { self.ops.close(); } }

      // A prepared format: (format-id, payload bytes). The PUBLIC wrapper decides how
      // many to produce: 0 (empty original → clear-only), 1 (text-only OR image-only),
      // or 2 (text+image). Format ids are plain u32 — the windows.rs builder fills in
      // CF_UNICODETEXT/CF_DIBV5, the fake fills in test ids. fsm.rs names neither.
      pub(super) fn restore_with<C: ClipOps>(
          c: &mut C,
          formats: &[(u32, Vec<u8>)],   // 0..=2 entries
      ) -> Result<(), RestoreError> {
          // 1. Allocate ALL payloads up front. Slot them in Option<C::Handle> so we can
          //    drain them one at a time (clean ownership, no take_or_clone pseudo-method).
          //    On any alloc failure, free the ones already held (reverse) and return.
          let mut handles: Vec<Option<C::Handle>> = Vec::with_capacity(formats.len());
          for (_, bytes) in formats {
              match c.alloc(bytes) {
                  Ok(h) => handles.push(Some(h)),
                  Err(e) => {
                      for slot in handles.into_iter().rev() { c.free(slot.unwrap()); }
                      return Err(RestoreError::Alloc(e));
                  }
              }
          }
          // 2. open. On FAILURE: free all allocated handles via `c` DIRECTLY (no guard
          //    exists yet → no second mutable borrow of C → borrow-checks). On success,
          //    hand `c` to the guard; from here all ops go through `clip` (single borrow).
          let mut clip = match c.open() {
              Ok(()) => OpenClip { ops: c },
              Err(e) => {
                  for slot in handles.into_iter().rev() { c.free(slot.unwrap()); }
                  return Err(RestoreError::Open(e));
              }
          };
          // 3. empty (always — 0 formats too: this clears the §B sentinel; round-11 P1 #1/#2).
          if let Err(e) = clip.empty() {
              for slot in handles.into_iter().rev().flatten() { clip.free(slot); }
              return Err(RestoreError::Empty(e));
          }
          // 4. submit each format in order, draining its slot. On a set FAILURE: the
          //    returned handle + all REMAINING un-drained handles are app-owned → free them
          //    via `clip`. Already-submitted handles are system-owned (NOT freed).
          //    - submitted == 0 at failure → nothing was on the clipboard (no re-empty).
          //    - submitted > 0 at failure → some formats ARE live → remedial empty; if THAT
          //      fails, surface SetPartial (partial data MAY remain — honest, round-11 small).
          let mut submitted = 0usize;
          for (i, (fmt, _)) in formats.iter().enumerate() {
              let h = handles[i].take().unwrap();
              match clip.set(*fmt, h) {
                  Ok(()) => { submitted += 1; }
                  Err((h_back, e)) => {
                      clip.free(h_back);
                      for slot in handles[i+1..].iter_mut().rev() { clip.free(slot.take().unwrap()); }
                      if submitted == 0 {
                          return Err(RestoreError::Set(e));
                      }
                      return match clip.empty() {
                          Ok(()) => Err(RestoreError::Set(e)),
                          Err(ce) => Err(RestoreError::SetPartial { cause: e, cleanup_err: ce }),
                      };
                  }
              }
          }
          Ok(())   // clip drops → close() exactly once on every path (incl. panic)
      }

      // thiserror is already a dep. Debug + thiserror::Error MUST be on the SAME derive
      // as the enum (round-12 review P1 #3: a commented-out derive after the enum does
      // nothing — map_err(|e| e.to_string()) and the fake assertions wouldn't compile).
      // Each variant gets a #[error(...)] so Display is real.
      #[derive(Debug, thiserror::Error)]
      pub(super) enum RestoreError {
          #[error("clipboard allocation failed: {0}")]           Alloc(String),
          #[error("clipboard open failed: {0}")]                 Open(String),
          #[error("clipboard empty failed: {0}")]                Empty(String),
          #[error("clipboard set failed: {0}")]                  Set(String),
          // a later set failed AND the remedial EmptyClipboard failed: earlier-submitted
          // formats MAY still be on the clipboard. cleanup_err is a String (always present
          // in this variant). The honest user-visible message names both failures + the
          // partial-data possibility — NOT silenced (prev rev did `let _ = clip.empty()`).
          #[error(
              "clipboard set failed: {cause}; cleanup also failed: {cleanup_err}; \
               clipboard may contain partial data"
          )]
          SetPartial { cause: String, cleanup_err: String },
      }

      // === src/clipboard/windows.rs — #[cfg(windows)] ONLY ===
      // Holds EVERYTHING Windows-specific: build_blobs + Win32ClipOps (both private to
      // the module) AND the public Windows restore_snapshot. This avoids two cfg mistakes
      // (round-12 review P1 #2): the parent must NOT call private build_blobs/Win32ClipOps
      // across the module boundary, and there must NOT be two `pub fn restore_snapshot`
      // definitions on Windows.
      //
      // build_blobs returns Vec<(u32, Vec<u8>)>: 0/1/2 entries based on text/image Options:
      //   - (None, None) → empty Vec  → restore_with clears only (§B empty-original fix)
      //   - (Some(t), None) → [(CF_UNICODETEXT, utf16_nul_bytes)]
      //   - (None, Some(img)) → [(CF_DIBV5, bitmapv5_bytes)]
      //   - (Some, Some) → both, in that order
      //   (uses OsStr::encode_wide, BITMAPV5HEADER, BI_BITFIELDS, LCS_sRGB — hence cfg(windows).)
      //
      // Win32ClipOps { owner }: impl ClipOps (open→OpenClipboard(self.owner),
      //   close→CloseClipboard, empty→EmptyClipboard, set→SetClipboardData(fmt,h),
      //   alloc→ see postcondition below, free→GlobalFree).
      //
      // The PUBLIC wrapper lives HERE (not in clipboard.rs):
      //   pub fn restore_snapshot(owner: OwnerHwnd, text: Option<&str>, image: Option<&ImageBlob>)
      //       -> Result<(), String>
      //   {
      //       let formats = build_blobs(text, image)?;              // private, same module
      //       let mut ops = Win32ClipOps { owner };                 // private, same module
      //       super::fsm::restore_with(&mut ops, &formats)          // platform-neutral FSM
      //           .map_err(|e| e.to_string())
      //   }
      //
      // === clipboard.rs cfg split (round-12 review P1 #2) — AVOID duplicate definitions ===
      // The existing `#[cfg(not(target_os = "macos"))] pub fn restore_snapshot` currently
      // compiles on Windows too. Narrow it and re-export so each target has exactly ONE
      // `restore_snapshot`:
      //   #[cfg(target_os = "macos")]   pub fn restore_snapshot(...) { … }           // current
      //   #[cfg(target_os = "windows")] mod windows;
      //   #[cfg(target_os = "windows")] pub use windows::restore_snapshot;          // re-export
      //   #[cfg(not(any(target_os = "macos", target_os = "windows")))]
      //   pub fn restore_snapshot(...) { … arboard sequential stub … }               // narrowed
      // (This is the FIRST commit of Task 2b — narrow the non-mac arm BEFORE adding the
      // Windows module, or Windows gets two `restore_snapshot` and fails to compile.)
      ```
      Why this shape: the always-compiled `fsm.rs` references NO Win32 symbol (format ids
      are plain `u32` params) and models 0–N formats generically (0 = clear-only, covering
      the §B empty-original case). `windows.rs` holds every Windows-only piece (blob builder
      + adapter). `restore_with` owns ALL handle lifetimes (alloc-all → open → empty →
      submit-loop) so the alloc-leak and cleanup-empty failures are fake-testable. `submit`
      is folded into `restore_with`'s loop (no separate borrow of `c`). `RestoreError` uses
      thiserror `Display`; `SetPartial.cleanup_err` is a `String` (always present in that
      variant). Honest semantics: the top-of-task "both absent" claim is corrected to
      "both absent IF the remedial empty succeeds; otherwise earlier-submitted formats may
      remain" (round-11 small fix).

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
      `empty` marks system-transferred handles freed). The FSM now takes a 0–N format
      list, so tests cover each cardinality:
      - **`zero_formats_clears`** (round-11 P1 #1/#2): empty Vec → `open`/`empty`/`close`,
        zero `alloc`/`set`/`free`. Proves the §B empty-original path clears (no sentinel).
      - **`one_format_success`** (round-11): 1 entry → 1 `alloc`, 1 `set`, zero re-`empty`,
        exactly 1 `close`, zero `free` (system-owned).
      - `two_formats_success`: 2 entries → 2 `alloc`, 2 `set`, zero re-`empty`, 1 `close`.
      - `open_fails` (2 fmts): → 2 `free` (pre-allocated), zero `empty`/`set`, zero `close`.
      - `empty_fails` (2 fmts): → 2 `free`, zero `set`, exactly 1 `close`. No re-empty.
      - `first_set_fails` (2 fmts): → 2 `free` (both unsubmitted), zero `empty` after,
        exactly 1 `close`.
      - `second_set_fails` (2 fmts): set#1 ok, set#2 errs → 1 `free` (h_dib; h_text
        system-owned → double-free detector guards it), 1 remedial `empty` WHILE OPEN,
        exactly 1 `close`.
      - **`first_alloc_fails`** (2 fmts): alloc#1 errs → zero `free` (none held), no `open`.
        RestoreError::Alloc.
      - **`second_alloc_fails`** (2 fmts): alloc#1 ok, alloc#2 errs → EXACTLY 1 `free`
        (h_text — the previously-leaked handle), no `open`. RestoreError::Alloc.
      - (NOT here: `alloc_lock_fails`. A ClipOps-level fake sees alloc as one black box and
        CANNOT catch the real adapter forgetting to GlobalFree on a GlobalLock failure —
        round-13 review P1. That leak-safety is tested in `windows::tests` by injecting a
        `GlobalMemOps` fake into the REAL `alloc_global` helper; see the Win32ClipOps::alloc
        bullet. The FSM fake here only models the alloc Ok/Err outcome, which is already
        covered by `first_alloc_fails` / `second_alloc_fails` above.)
      - **`cleanup_empty_fails`** (2 fmts): set#1 ok, set#2 errs, remedial `empty` errs →
        1 `free` (h_dib), result is `RestoreError::SetPartial { cleanup_err: <str> }`.
        The test asserts `err.to_string()` mentions the cleanup failure AND partial data
        (NOT silenced — prev rev did `let _ = clip.empty()`). **Ownership assertion is
        narrow here (round-11 small): assert only that the APP-OWNED handle (h_dib) was
        freed and no app handle leaked — do NOT assert all handles are gone, because the
        system-owned h_text may legitimately still exist on the clipboard after a failed
        cleanup empty.**
      - `close` count is `== 1` in every branch that opened (the `OpenClip` guard's `Drop`
        calls `ClipOps::close`, which the fake records). `RestoreError`'s `Display`
      (thiserror) is asserted via `to_string()` in the cross-platform fake tests — these
      do NOT exercise the Windows-only public `restore_snapshot` wrapper.

- [ ] **`windows::tests` — `build_blobs` cardinality/ORDER unit tests (round-13 review
      P1 #2: the cross-platform FSM fake tests take a pre-built format list and so cannot
      prove `build_blobs` maps the four Option combos correctly).** Private Windows-only
      unit tests (inside `windows.rs`, `#[cfg(test)]`) that call `build_blobs` directly and
      assert the exact entry COUNT, FORMAT IDS, and ORDER for each combo:
      - `(None, None)` → empty Vec (0 entries) — the §B empty-original case.
      - `(Some("hi"), None)` → exactly 1 entry: `(CF_UNICODETEXT, …)`. Decode the bytes as
        UTF-16 + NUL, assert == "hi".
      - `(None, Some(img))` → exactly 1 entry: `(CF_DIBV5, …)`. (Header/pixel content is
        validated by the real integration test below; here just assert the format id + count.)
      - `(Some("hi"), Some(img))` → exactly 2 entries, ORDER `[CF_UNICODETEXT, CF_DIBV5]`
        (text first). Asserting the ORDER here catches a swap that the FSM would silently
        accept (the FSM is format-id-agnostic).
      Format ids are the real `CF_UNICODETEXT`/`CF_DIBV5` constants (these tests compile only
      on Windows). This is the layer that proves the public `restore_snapshot` produces the
      right shape before any clipboard call.

- [ ] **Real Windows integration test (`tests/clipboard_win.rs`, Windows CI only).**
      The SUCCESS paths for ALL FOUR cardinalities (round-13 review P1 #2: the prev test
      only covered text+image). Failure branches are covered by the fake FSM above (they
      can't be reliably forced against the real clipboard). It needs an owner HWND but has
      no Tauri window, so it creates a throwaway message-only window FOR THE TEST (not the
      app). Lifecycle discipline (round-9 small fix — "short-lived" must have an actual
      cleanup step):
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
      Cases (each on a fresh EmptyClipboard):
      1. **`(None, None)` — sentinel cleared (round-13 P1 #2):** write a §B sentinel string
         to the clipboard first, then `restore_snapshot(owner, None, None)`, assert
         `IsClipboardFormatAvailable(CF_UNICODETEXT)` is FALSE and the clipboard has no
         data (the sentinel is gone). Mirrors the macOS `restore_empty_to` real-pasteboard test.
      2. **Text-only:** `restore_snapshot(owner, Some("hi"), None)` → read CF_UNICODETEXT == "hi",
         assert CF_DIBV5 absent.
      3. **Image-only:** `restore_snapshot(owner, None, Some(&img))` → read CF_DIBV5, assert
         header + pixels; assert CF_UNICODETEXT absent.
      4. **Text+image (4-color 2×2):** `restore_snapshot(owner, Some("hi"), Some(&img))` →
         read both; CF_UNICODETEXT == "hi"; CF_DIBV5 header (2×2, 32bpp, BI_BITFIELDS, masks)
         then assert EACH pixel position by its DISTINCT color:
         - `[(255,0,0,255), (0,255,0,255), (0,0,255,255), (255,255,0,255)]` —
           TL=red, TR=green, BL=blue, BR=yellow → BGRA per the masks. A wrong row stride OR a
           bottom-up flip scrambles these distinct values (the definitive stride + row-direction
           test; a uniform image couldn't do it).
      5. **Negative:** invalid RGBA (len mismatch) returns Err, clipboard unchanged (marker
         survives).

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
- [ ] Per-window capabilities with correct permission IDs (`allow-$command`, NO `linguaray:` prefix):
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
    CERT_PATH="$RUNNER_TEMP/linguaray-cert.p12"
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
    rm -f "$RUNNER_TEMP/linguaray-cert.p12"
```
- [ ] **Windows signing** (Authenticode via `certificateThumbprint` overlay config written to a FILE — no inline JSON quoting):
```yaml
- name: Import PFX + build (Windows)
  if: matrix.os == 'windows-latest'
  env:
    WINDOWS_CERTIFICATE_PFX: ${{ secrets.WINDOWS_CERTIFICATE_PFX }}
    WINDOWS_CERTIFICATE_PASSWORD: ${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }}
  run: |
    $certPath = "$env:RUNNER_TEMP\linguaray-cert.pfx"
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
  run: Remove-Item "$env:RUNNER_TEMP\linguaray-cert.pfx" -ErrorAction SilentlyContinue
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
