# Phase 4: Windows Parity + Cross-Platform Packaging — Implementation Plan (rev 5)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`) syntax for tracking.

**Status:** rev 5 — fixes the round-3 review's 4 plan blockers: (a) CSP tightened to IPC-only `connect-src` (no wildcard https/ws — provider calls are Rust-side), (b) AppManifest::commands pinned with exact build.rs code + permission IDs, (c) macOS signing script: `$RUNNER_TEMP` path + `codesign:` in partition list + `if: always()` cleanup, (d) Windows signing: `certificateThumbprint` via overlay config (no `certificatePath`).

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

## Task 3: Release bundle config — CSP + env-driven signing

**Files:** `src-tauri/tauri.conf.json`

- [ ] CSP (production): **NO wildcard `https:` / `ws:`** — provider HTTP calls go through Rust reqwest, NOT the WebView. The WebView only needs Tauri IPC. Production CSP: `"default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; connect-src 'self'"`. **`devCsp`**: add ONLY the Vite HMR origin explicitly: `connect-src 'self' ws://localhost:1420` (Vite dev server port) + `'unsafe-inline'` in script-src. Verify dev (HMR works) + the three windows work. [Tauri Capabilities](https://v2.tauri.app/security/capabilities/)
- [ ] Signing: drive purely via env vars (NO `${APPLE_SIGNING_IDENTITY}` in JSON). Keep `bundle.macOS.minimumSystemVersion: "11.0"`.
- [ ] Commit.

## Task 4: Capabilities (decided: per-window via AppManifest::commands)

**Files:** `src-tauri/build.rs`, `src-tauri/capabilities/`

- [ ] **Exact build.rs code** (no "verify API" TBD):
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
- [ ] Per-window capabilities with exact permission IDs:
  - `capabilities/main.json` — `"permissions": ["core:default", "store:default", "islandpot:allow-translate", "islandpot:allow-translate-default", "islandpot:allow-translate-clipboard", "islandpot:allow-list-engines", "islandpot:allow-set-key", "islandpot:allow-delete-key", "islandpot:allow-key-status", "islandpot:allow-get-settings", "islandpot:allow-set-setting", "islandpot:allow-lookup-dictionary", "islandpot:allow-a11y-status", "islandpot:allow-keystore-health", "islandpot:allow-archive-keystore", "islandpot:allow-reset-keystore"]`
  - `capabilities/popup.json` — `"permissions": ["core:default"]` (popup only listens to events; no commands)
  - `capabilities/input.json` — `"permissions": ["core:default", "islandpot:allow-translate-default", "islandpot:allow-get-settings"]`
- [ ] Drop `opener`, `global-shortcut` from window permissions. Verify each window resolves its scoped commands at runtime (exercise all three in dev).
- [ ] Commit.

## Task 5: GitHub Actions release workflow (correct signing + runner)

**Files:** `.github/workflows/release.yml`

- [ ] Matrix: `macos-latest` (arm64), **`macos-15-intel`** (NOT retired `macos-13`), `windows-latest` (x86_64-pc-windows-msvc).
- [ ] **macOS signing** (pinned, no TBD):
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
    security import "$CERT_PATH" -P "$APPLE_CERTIFICATE_PASSWORD" -k build.keychain -T /usr/bin/codesign
    security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$KEYCHAIN_PASSWORD" build.keychain
    pnpm tauri build --target ${{ matrix.target }}

- name: Cleanup keychain (macOS)
  if: always() && startsWith(matrix.os, 'macos')
  run: security delete-keychain build.keychain || true
```
Key fixes from round-3 review: `$RUNNER_TEMP` (not `mktemp` template); `codesign:` added to partition list; cleanup in `if: always()` step.
- [ ] **Windows signing** (Authenticode via `certificateThumbprint` overlay — NOT the `TAURI_SIGNING_PRIVATE_KEY*` updater key; NOT nonexistent `certificatePath`):
```yaml
- name: Import PFX + build (Windows)
  if: matrix.os == 'windows-latest'
  env:
    WINDOWS_CERTIFICATE_PFX: ${{ secrets.WINDOWS_CERTIFICATE_PFX }}
    WINDOWS_CERTIFICATE_PASSWORD: ${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }}
  run: |
    $certPath = "$env:RUNNER_TEMP\islandpot-cert.pfx"
    [System.Convert]::FromBase64String("$env:WINDOWS_CERTIFICATE_PFX") | Set-Content $certPath -AsByteStream
    $cert = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2($certPath, $env:WINDOWS_CERTIFICATE_PASSWORD)
    Import-PfxCertificate -CertStoreLocation Cert:\CurrentUser\My -FilePath $certPath -Password (ConvertTo-SecureString -String $env:WINDOWS_CERTIFICATE_PASSWORD -AsPlainText -Force)
    $thumbprint = $cert.Thumbprint
    # Pass the thumbprint to Tauri via an overlay config.
    pnpm tauri build --target ${{ matrix.target }} --config `"{"bundle":{"windows":{"certificateThumbprint":"$thumbprint","digestAlgorithm":"sha256","timestampUrl":"http://timestamp.digicert.com"}}}`"
```
If no Authenticode cert: build unsigned (document the SmartScreen warning). `TAURI_SIGNING_PRIVATE_KEY*` is the UPDATER signature only (separate plugin).
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
