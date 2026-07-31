# Phase 4: Windows Parity + Cross-Platform Packaging — Implementation Plan (rev 7)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`) syntax for tracking.

**Status:** rev 7 — fixes round-6 review issues: (a) Task 2b Windows compound-clipboard now spells the full Win32 protocol (non-NULL HWND owner via message-only window, GMEM_MOVEABLE + ownership transfer, RAII CloseClipboard, explicit re-EmptyClipboard half-state cleanup on second-SetClipboardData failure, BGRA pixel layout + masks, required windows-sys features), (b) macOS signing identity is verified with `grep -F` against `APPLE_SIGNING_IDENTITY` (fail-fast, not just listed), (c) removed the stray code fence + duplicated unsigned-cert note, (d) "all-or-nothing" qualified to mean conversion-phase only (clearContents/writeObjects are not themselves a transaction).

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

- [ ] **HWND owner — MUST be non-NULL.** `EmptyClipboard` assigns ownership to the
      window that has the clipboard open. If `OpenClipboard(NULL)` was used,
      `EmptyClipboard` succeeds but sets the owner to NULL, which makes
      `SetClipboardData` fail. (Verified: EmptyClipboard Remarks — "If the
      application specifies a NULL window handle when opening the clipboard,
      EmptyClipboard succeeds but sets the clipboard owner to NULL. Note that this
      causes SetClipboardData to fail.") Tauri's Rust backend has no UI window on
      the capture thread, so create a **message-only window** as the owner:
      `CreateWindowExW(0, "STATIC", "", 0, 0,0,0,0, HWND_MESSAGE, null, hinst, null)`
      (HWND_MESSAGE = -3). Cache it in a `static OnceCell<isize>` keyed lazily on
      first restore; it only needs to exist (we use eager rendering, NOT delayed
      rendering, so no WndProc / message pump is required — we hand real handles to
      `SetClipboardData`, never NULL).

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

- [ ] **Submit sequence + half-state cleanup (RAII):**
      1. `OpenClipboard(hwnd_owner)` → must succeed; on fail, `GlobalFree` both
         pre-built handles, return Err.
      2. `EmptyClipboard()` → on fail, close + free both, return Err.
      3. `SetClipboardData(CF_UNICODETEXT, h_text)` → on fail: free `h_dib`, close,
         re-`EmptyClipboard()` (the text handle was NOT taken since the call failed),
         return Err. (Half-state: only text submitted ⇒ re-empty to leave neither.)
      4. `SetClipboardData(CF_DIBV5, h_dib)` → on fail: **`h_text` was already taken
         by step 3 (do NOT free it — it's system-owned).** Re-`EmptyClipboard()` to
         remove the orphaned text, `GlobalFree(h_dib)` (not taken), close, return Err.
      5. Success: both submitted → both now system-owned. Just `CloseClipboard()`.
      - Use a small RAII guard (`struct ClipGuard { hwnd, opened: bool }` impls Drop:
        `if opened { CloseClipboard() }`) so `CloseClipboard` always runs even on
        early-return / panic. The re-`EmptyClipboard` in the failure branches is the
        explicit de-half-state step (the RAII close handles only CloseClipboard).

- [ ] **windows-sys features** (add to the existing `windows-sys` dep): currently
      `Win32_System_DataExchange` + `Win32_Foundation`. ADD:
      - `Win32_System_Memory` (GlobalAlloc/GlobalLock/GlobalUnlock/GlobalFree,
        GMEM_MOVEABLE)
      - `Win32_Graphics_Gdi` (BITMAPV5HEADER, BI_BITFIELDS, LCS_sRGB, LCS_GM_IMAGES)
      - `Win32_UI_WindowsAndMessaging` (CreateWindowExW, HWND_MESSAGE,
        OpenClipboard, EmptyClipboard, SetClipboardData, CloseClipboard,
        CF_UNICODETEXT, CF_DIBV5)
      (Verify each symbol's module via `windows-docs-rs` during impl; the
      `Win32_Security_Authorization`/`Win32_Security` from Task 2 are separate.)

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
         0x00FF0000, alphaMask==0xFF000000; read first pixel, assert BGRA ==
         (0,0,255,255) (red in BGRA). This covers channel order, stride, alpha.
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
