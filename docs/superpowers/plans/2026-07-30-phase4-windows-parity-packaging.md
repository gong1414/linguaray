# Phase 4: Windows Parity + Cross-Platform Packaging — Implementation Plan (rev 8)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`) syntax for tracking.

**Status:** rev 8 — fixes round-7 review issues: (a) Task 2b HWND owner now reuses
the Tauri main window's HWND (`Window::hwnd()`, on the event-loop thread that
already pumps messages — required because the owner receives WM_DESTROYCLIPBOARD
even with eager rendering; dropped the message-only-window-on-an-async-worker +
OnceCell approach), (b) submit/handle-ownership sequence corrected — first-submit
failure frees BOTH handles with NO re-empty (nothing on clipboard; EmptyClipboard
after CloseClipboard is invalid); only second-submit failure re-empties WHILE OPEN
and frees only the unsubmitted handle, (c) windows-sys 0.59 feature map corrected
by grepping the crate source: CF_UNICODETEXT/CF_DIBV5 are in `Win32_System_Ole`
(was wrongly placed in WindowsAndMessaging), DataExchange already provides the
clipboard functions, WindowsAndMessaging is no longer needed, LCS_sRGB is in
`Win32_UI_ColorSystem`; added explicit checked i32/u32/usize conversions mirroring
the macOS preflight.

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

**Files:** `src-tauri/src/clipboard.rs`, `src-tauri/Cargo.toml`

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
      Tauri's event-loop thread (which already pumps messages), it is stable for the
      app lifetime, and `tauri::Window::hwnd() -> crate::Result<HWND>` is the public
      accessor (`#[cfg(windows)]`, tauri 2.11.5 `src/window/mod.rs:1668`). Obtain it
      from the `AppHandle`/`Window` passed into the restore path; do NOT create a
      message-only window on an arbitrary async-runtime worker and cache its HWND in
      a `OnceCell` (that thread has no message loop and may exit, leaving a stale
      owner). If the main window is unavailable, return Err (best-effort: no restore).

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

- [ ] **Submit sequence + handle ownership (RAII):**
      1. `OpenClipboard(hwnd_owner)` → on fail: `GlobalFree` BOTH pre-built handles
         (`h_text`, `h_dib`), return Err. (Clipboard was never opened; nothing to
         close or empty.)
      2. `EmptyClipboard()` → on fail: `GlobalFree` BOTH handles, `CloseClipboard()`,
         return Err. (Nothing was submitted, so no half-state.)
      3. `SetClipboardData(CF_UNICODETEXT, h_text)`:
         - **Success:** `h_text` ownership transfers to the system — do NOT free it.
         - **Failure:** `h_text` was NOT taken (the call failed), so `GlobalFree` it;
           `h_dib` is also still unsubmitted, so `GlobalFree` it too. There is **no
           half-state to clean up** (nothing was submitted), so do NOT call
           `EmptyClipboard` again. `CloseClipboard()`, return Err.
           (Prev rev was wrong here: it freed only `h_dib` and re-emptied — but on a
           first-submit failure nothing is on the clipboard to remove, and a call to
           `EmptyClipboard` AFTER `CloseClipboard` is invalid anyway — it requires
           the clipboard to still be open.)
      4. `SetClipboardData(CF_DIBV5, h_dib)`:
         - **Success:** `h_dib` ownership transfers to the system. Both formats are
           now live (both system-owned). `CloseClipboard()`, return Ok.
         - **Failure:** `h_text` WAS already taken by step 3 (system-owned — do NOT
           free it, or double-free). `h_dib` was not taken → `GlobalFree` it. Now
           there IS a half-state (only text is on the clipboard). Re-`EmptyClipboard()`
           to remove the orphaned text — **while the clipboard is still open** (the
           guard has not closed it yet). `CloseClipboard()`, return Err.
      - Use a small RAII guard (`struct ClipGuard { opened: bool }` impls Drop:
        `if opened { CloseClipboard() }`) so `CloseClipboard` always runs even on
        early-return / panic. The re-`EmptyClipboard` in step 4 runs BEFORE the guard
        drops (clipboard still open); steps 1-3 never need a re-empty because nothing
        is on the clipboard until a submit succeeds.

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
      `Win32_UI_WindowsAndMessaging` is NOT needed (we reuse the Tauri main-window
      HWND — no `CreateWindowExW`/`HWND_MESSAGE`); it was listed in the prev rev for
      the now-dropped message-only-window approach.
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

- [ ] Add `#[cfg(target_os = "windows")] fn restore_snapshot(...)` mirroring the
      macOS signature, replacing the sequential-arboard cfg-arm for Windows. Keep the
      non-mac/non-win sequential stub for unsupported targets.

- [ ] **Test (Windows CI, `#[cfg(target_os="windows")]` integration test):**
      1. Build a 2×2 all-red RGBA `ImageBlob` (R=255,G=0,B=0,A=255).
      2. `restore_snapshot(Some("hi"), Some(&img))`.
      3. `OpenClipboard(NULL)` (read path: NULL is fine for reading), assert
         `IsClipboardFormatAvailable(CF_UNICODETEXT)` AND
         `IsClipboardFormatAvailable(CF_DIBV5)`.
      4. `GetClipboardData(CF_UNICODETEXT)` → decode UTF-16, assert == "hi".
      5. `GetClipboardData(CF_DIBV5)` → lock, read BITMAPV5HEADER, assert width==2,
         height==2 (abs), bitcount==32, compression==BI_BITFIELDS, redMask==
         0x00FF0000, alphaMask==0xFF000000; read ALL 4 pixels, assert each BGRA ==
         (0,0,255,255) (red in BGRA). All-4-pixels (not just the first) catches a
         row-stride bug that would read padding bytes on rows 2; channel order +
         alpha are covered by the BGRA values.
      6. Negative test: invalid RGBA (len mismatch) returns Err AND clipboard is
         unchanged (write a marker first, assert it survives).

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
