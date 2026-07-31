# Phase 4: Windows Parity + Cross-Platform Packaging — Implementation Plan (rev 9)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`) syntax for tracking.

**Status:** rev 9 — fixes round-8 review issues (all in Task 2b, all from the
HWND decision not yet landing in the data flow): (a) **callchain + HWND threading**
— files list now includes `lib.rs`/`selection.rs`/tests; `OsClipboard` carries a
Windows-only owner HWND; `capture_selection` gains a Windows-only `owner` param;
`on_hotkey` resolves `app.get_window("main").hwnd()` and threads it down; the
windows-crate `HWND` (tauri's `windows` 0.61, newtype `HWND(pub *mut c_void)`)
converts to windows-sys's `HWND` (`*mut c_void`) via a single `.0` field access
(no isize round-trip); `ClipboardLike` trait and the pure FSM stay unchanged.
(b) **single-close RAII** — `ClipGuard` constructed right after a successful
`OpenClipboard` is the ONLY closer; no manual `CloseClipboard()` in any branch
(prev rev double-closed); each branch just `return`s. (c) **failure-injection
tests** — extract a `ClipOps` adapter; a fake forces open/empty/set1/set2 failures
and asserts exact `free`/`empty`/`close` counts + order + no-leak/no-double-free
via ownership bookkeeping (runs in `cargo test` on all platforms, no clipboard
needed); the real Windows CI test is success-only with a 4-color image that proves
stride + row direction.

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

**Files:** `src-tauri/src/clipboard.rs`, `src-tauri/src/selection.rs`,
`src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`, `src-tauri/tests/clipboard_win.rs`
(new), `src-tauri/tests/clipboard_win_fsm.rs` (new, failure-injection unit tests).

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
      in the actual data flow, not just be stated).** Today the path is
      `lib.rs::on_hotkey` → `selection::capture_selection(timeout)` → unit
      `OsClipboard` → `clipboard::restore_snapshot(text, image)`, with no
      `AppHandle`/`Window`/HWND anywhere. Changes:
      - **`selection.rs`**: `OsClipboard` gains a Windows-only owner field.
        `capture_selection` / `capture_selection_with_ax` gain a Windows-only
        `owner: windows_sys::Win32::Foundation::HWND` param threaded into
        `OsClipboard { #[cfg(windows)] owner }`:
        ```rust
        // selection.rs
        #[cfg(target_os = "windows")]
        type OwnerHwnd = windows_sys::Win32::Foundation::HWND; // *mut c_void
        struct OsClipboard {
            #[cfg(target_os = "windows")]
            owner: OwnerHwnd,
        }
        // macOS path keeps OsClipboard as a unit struct (no HWND) — the cfg keeps
        // the macOS build and its selection_engine unit tests unchanged.
        #[cfg(target_os = "windows")]
        pub fn capture_selection(timeout_ms: u64, owner: OwnerHwnd) -> Result<Capture, String> { … }
        #[cfg(not(target_os = "windows"))]
        pub fn capture_selection(timeout_ms: u64) -> Result<Capture, String> { … }
        ```
        `OsClipboard::restore_snapshot` passes `self.owner` to the Windows
        `clipboard::restore_snapshot`. The `ClipboardLike` trait itself is
        UNCHANGED — `OsClipboard` already implements it; only its concrete fields
        and the `restore_snapshot` body differ by cfg. `selection_engine::capture`
        (the pure FSM, unit-tested with a Fake) is untouched.
      - **`lib.rs::on_hotkey`** (the only caller of `capture_selection`):
        ```rust
        // Windows: resolve the main window's HWND on the event-loop-owned call and
        // pass it down. macOS/other: unchanged signature.
        #[cfg(target_os = "windows")]
        let cap = {
            let hwnd = app2.get_window("main")
                .ok_or_else(|| "main window unavailable".to_string())
                .and_then(|w| w.hwnd().map_err(|e| e.to_string()))?;
            selection::capture_selection(800, owner_from_tauri(hwnd))
        };
        #[cfg(not(target_os = "windows"))]
        let cap = selection::capture_selection(800);
        ```
      - **windows-crate HWND → windows-sys HWND conversion (round-8 review P1).**
        `tauri::Window::hwnd()` returns the HIGH-LEVEL `windows` crate's
        `HWND` (`windows::Win32::Foundation::HWND`, a newtype
        `pub struct HWND(pub *mut c_void)` — tauri 2.11.5 depends on `windows`
        0.61). Our code calls `windows-sys` 0.59, whose `HWND` is a type alias
        `*mut c_void`. Same pointee type, so the conversion is a single field
        access — NO `isize` round-trip, NO handle reinterpretation:
        ```rust
        // clipboard.rs (Windows)
        fn owner_from_tauri(h: windows::Win32::Foundation::HWND) -> windows_sys::Win32::Foundation::HWND {
            h.0 // *mut c_void  ==  windows-sys HWND
        }
        ```
        (We do NOT add a `windows` crate dependency to islandpot — only the one
        conversion site references the type, and it lives behind the cfg in the
        `lib.rs` call site where `w.hwnd()` already returns it. If avoiding the
        `windows` crate entirely is preferred, `w.hwnd().unwrap().0 as isize` then
        `isize as *mut c_void` is equivalent but adds a redundant cast chain; the
        `.0` direct access is cleaner and type-checked.)
      - **`translate_clipboard`** (the other selection caller, lib.rs:142) does NOT
        call `restore_snapshot` — it only reads text — so it needs no HWND change.
      - **`clipboard.rs`**: the Windows `restore_snapshot` signature becomes
        `restore_snapshot(owner: OwnerHwnd, text: Option<&str>, image: Option<&ImageBlob>)`.
        macOS signature is unchanged. The Win32 adapter (see FSM bullet below) is
        injected here so the failure paths are unit-testable.

- [ ] **Memory: `GlobalAlloc(GMEM_MOVEABLE, len)` for EACH blob.** Verified: "A
      memory object that is to be placed on the clipboard should be allocated by
      using GlobalAlloc with the GMEM_MOVEABLE flag." Use `GlobalAlloc(GMEM_MOVEABLE,
      len)` → `GlobalLock` → `copy_nonoverlapping` → `GlobalUnlock`. After a
      successful `SetClipboardData(h)`, **ownership transfers to the system** — do
      NOT `GlobalFree` a submitted handle (the system frees it on next empty). Only
      `GlobalFree` handles that were NOT successfully submitted.

- [ ] **Build BOTH blobs BEFORE `EmptyClipboard`.** (a) UTF-16 NUL-terminated text:
      `OsStr::encode_wide().chain(Some(0))`, byte-len = u16_count * 2. (b) CF_DIBV5
      blob: `BITMAPV5HEADER` + BGRA pixel buffer (see layout below). If either build
      fails, return Err — clipboard is untouched.

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

- [ ] **Submit sequence + handle ownership (single-close RAII discipline).**
      Round-8 review P1: the prev rev called `CloseClipboard()` manually in branches
      2-4 AND in `ClipGuard::drop` — a double close. Fix: **the guard is the ONLY
      closer.** Construct it immediately after a successful `OpenClipboard`; every
      branch from then on does its handle work and then `return`s — the guard's
      `Drop` closes exactly once, including on panic. No manual `CloseClipboard()`
      appears anywhere after the guard exists.
      ```text
      1. build h_text, h_dib (GMEM_MOVEABLE + GlobalLock/copy/Unlock)   [clipboard UNTOUCHED on fail → return Err]
      2. OpenClipboard(owner):
           fail → GlobalFree(h_text); GlobalFree(h_dib); return Err     [never opened, no guard]
           ok   → let _guard = ClipGuard::new();   // ONLY closer from here
      3. EmptyClipboard():
           fail → GlobalFree(h_text); GlobalFree(h_dib); return Err     [_guard drops → CloseClipboard once]
      4. SetClipboardData(CF_UNICODETEXT, h_text):
           fail → GlobalFree(h_text); GlobalFree(h_dib); return Err     [NOTHING submitted yet;
                                                                       no half-state; no re-empty;
                                                                       _guard drops → CloseClipboard once]
           ok   → h_text is now system-owned (do NOT free)
      5. SetClipboardData(CF_DIBV5, h_dib):
           ok   → return Ok(())   [both system-owned; _guard drops → CloseClipboard once]
           fail → h_text WAS taken (system-owned, do NOT free);
                  GlobalFree(h_dib);                              // half-state: only text is live
                  EmptyClipboard();                               // remove orphaned text — STILL OPEN (_guard alive)
                  return Err                                      [_guard drops → CloseClipboard once]
      ```
      `struct ClipGuard; impl Drop for ClipGuard { fn drop(&mut self) { unsafe { CloseClipboard(); } } }`
      — no `opened` flag needed (it's only ever constructed right after a successful
      open, so Drop always closes). The re-`EmptyClipboard` in step 5 runs while the
      guard is alive (clipboard still open); steps 3-4 never re-empty because nothing
      is on the clipboard until a submit succeeds. Handle ownership is trackable in
      the fake FSM test (next bullet) by counting `GlobalAlloc`/`GlobalFree` pairs.

- [ ] **windows-sys 0.59 features** — verified module paths by grepping the crate
      source (`~/.cargo/registry/.../windows-sys-0.59.0/src/`), NOT by guessing.
      The dep currently has `Win32_System_DataExchange` + `Win32_Foundation`. ADD:
      - `Win32_System_Memory` — `GlobalAlloc`, `GlobalLock`, `GlobalUnlock`,
        `GlobalFree`, `GMEM_MOVEABLE`
      - `Win32_System_Ole` — **`CF_UNICODETEXT`, `CF_DIBV5`** (these format
        constants live in Ole, NOT WindowsAndMessaging — the prev rev placed them
        wrong and the build would have failed)
      - `Win32_Graphics_Gdi` — `BITMAPV5HEADER`, `BI_BITFIELDS`, `LCS_GM_IMAGES`
      - `Win32_UI_ColorSystem` — `LCS_sRGB` (lives in ColorSystem, not Gdi)
      Already present, no action: `Win32_System_DataExchange` provides
      `OpenClipboard`/`EmptyClipboard`/`SetClipboardData`/`CloseClipboard`. NOTE:
      `Win32_UI_WindowsAndMessaging` is NOT needed by the APP (it reuses the Tauri
      main-window HWND — no `CreateWindowExW`/`HWND_MESSAGE`), but the real Windows
      CI test DOES use `CreateWindowExW`/`HWND_MESSAGE`/`PeekMessage`/`DispatchMessage`
      for its throwaway test owner, so add `Win32_UI_WindowsAndMessaging` as a
      `[dev-dependencies]`-style test-only feature (or accept it in the main dep —
      it's cheap). The app's production path never touches it.
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

- [ ] Add `#[cfg(target_os = "windows")] fn restore_snapshot(owner, text, image)`
      (owner-bearing signature per the callchain bullet), replacing the
      sequential-arboard cfg-arm for Windows. macOS signature unchanged. Keep a
      non-mac/non-win sequential stub for unsupported targets.

- [ ] **Extract an injectable Win32 adapter so the failure branches are testable
      without a real clipboard (round-8 review P1).** The submit logic is the
      danger zone; isolate it behind a trait the real impl and a fake both satisfy:
      ```rust
      #[cfg(target_os = "windows")]
      trait ClipOps {
          fn open(&mut self, owner: OwnerHwnd) -> Result<()>;
          fn empty(&mut self) -> Result<()>;
          fn set(&mut self, fmt: u32, h: GlobalHandle) -> Result<()>; // ownership transfers on Ok
          // GlobalAlloc/Lock/Free are also on the trait (or on a MemoryOps sibling)
          // so the fake can count alloc/free pairs to prove no double-free / no leak.
          fn alloc(&mut self, bytes: &[u8]) -> Result<GlobalHandle>;
          fn free(&mut self, h: GlobalHandle);
      }
      ```
      `restore_snapshot` takes a `&mut impl ClipOps` (the real impl wraps the
      `windows-sys` calls; a fake records calls). This mirrors how
      `selection_engine::capture` is already pure + tested via a `Fake` clipboard.
      The submit algorithm lives in a `fn submit<C: ClipOps>(c: &mut C, h_text, h_dib)
      -> Result<()>` that encodes ONLY the step-2..5 sequence above — no preflight,
      no conversion, pure ownership transitions. (Preflight + BGRA conversion stay
      in the non-generic `restore_snapshot` wrapper and get their own unit tests.)

- [ ] **Failure-injection unit tests (`tests/clipboard_win_fsm.rs`).** The fake
      needs no Win32 types (it just records calls against an in-memory ownership
      map), so these are NOT cfg-gated to Windows — they run in `cargo test` on ALL
      platforms (macOS dev, Linux CI, Windows CI), exactly like the existing
      `selection_engine` FSM tests that use a cross-platform `Fake` clipboard. The
      fake forces each API to fail and records the call sequence:
      - `open_fails`: OpenClipboard errs → exactly 2 `free` (h_text, h_dib), zero
        `empty`, zero `set`, zero `close` (guard never constructed). No leak.
      - `empty_fails`: EmptyClipboard errs → 2 `free`, zero `set`, exactly 1
        `close` (guard dropped). No re-empty.
      - `first_set_fails`: Set(CF_UNICODETEXT) errs → 2 `free` (both unsubmitted),
        zero `set` success, ZERO `empty` after the failure (nothing to remove),
        exactly 1 `close`. Proves no re-empty on first-submit failure.
      - `second_set_fails`: Set(CF_UNICODETEXT) ok, Set(CF_DIBV5) errs → exactly 1
        `free` (h_dib only — h_text is system-owned, freeing it would double-free),
        exactly 1 `empty` AFTER the failure (remove orphaned text) WHILE still open,
        exactly 1 `close`. Proves the half-state cleanup + ownership transfer.
      - `success`: both sets ok → zero `free` (both system-owned), zero re-`empty`,
        exactly 1 `close`.
      - **Ownership bookkeeping**: the fake tracks each `alloc`'d handle as
        "owned-by-us" and flips it to "owned-by-system" on a successful `set`.
        `free` on a system-owned handle panics (double-free detector). At end of
        every test, assert every handle is either system-owned or freed — no leaks,
        no double-frees. This is the core safety property the protocol must hold.
      - Close count is asserted `== 1` in every branch that opened (the guard is
        the only closer; these tests prove the single-close discipline directly).

- [ ] **Real Windows integration test (`tests/clipboard_win.rs`, Windows CI only).**
      This is the SUCCESS path only — the failure branches are covered by the fake
      FSM above (they can't be reliably forced against the real clipboard). It needs
      an owner HWND but has no Tauri window, so it creates a throwaway message-only
      window FOR THE TEST (not the app): `CreateWindowExW(0, "STATIC", …,
      HWND_MESSAGE, …)` on the test thread and runs a minimal `PeekMessage`/`Dispatch`
      loop around the assertion (so a `WM_DESTROYCLIPBOARD` from a concurrent app
      can't deadlock the test). This is acceptable in a test (short-lived, single
      thread) precisely because it is NOT the app's permanent owner.
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
