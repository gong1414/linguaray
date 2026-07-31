# Phase 4: Windows Parity + Cross-Platform Packaging — Implementation Plan (rev 3)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`) syntax for tracking.

**Status:** rev 4 — addresses the round-2 review's remaining plan issues: (a) `Win32_Storage_FileSystem` feature is NOT currently present (must add), (b) macOS signing steps pinned to the full keychain-import flow (no TBD), (c) Windows signing clarified as Authenticode via `certificateThumbprint` (NOT the `TAURI_SIGNING_PRIVATE_KEY*` updater key, NOT nonexistent `certificatePath`), (d) capabilities decided: `AppManifest::commands` per-window (no either/or), (e) CSP needs an explicit `devCsp` for Vite HMR (Tauri does NOT auto-relax in dev).

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

- [ ] CSP (production): `app.security.csp` = `"default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; connect-src 'self' https: http://localhost:* http://127.0.0.1:*"`. **ALSO set `app.security.devCsp`** — round-3 review: Tauri does NOT auto-relax the production CSP in dev when devCsp is unset; the production CSP would block Vite's HMR WebSocket. Set `devCsp` to the production policy PLUS `connect-src ws: wss:` (Vite HMR) + `'unsafe-inline'` script (Vite dev injects inline). Verify dev (HMR works) + the three windows work.
- [ ] Signing: drive purely via env vars (NO `${APPLE_SIGNING_IDENTITY}` in JSON). Keep `bundle.macOS.minimumSystemVersion: "11.0"`.
- [ ] Commit.

## Task 4: Capabilities (decided: per-window via AppManifest::commands)

**Files:** `src-tauri/build.rs` (AppManifest::commands), `src-tauri/capabilities/`

- [ ] Round-3 decision (no more either/or): use `build.rs` `tauri_build::try_build` with an `ApplicationInfo`/manifest that declares our custom commands via `AppManifest::commands`, then scope each window's capability to only the commands it needs:
  - `main`: translate, translate_default, translate_clipboard, list_engines, set_key, delete_key, key_status, get_settings, set_setting, lookup_dictionary, a11y_status, keystore_health, archive_keystore, reset_keystore
  - `popup`: (only listens to `popup-state` events — no commands; the popup hides via its own window API, not a command)
  - `input`: translate_default, get_settings
  - Drop `opener`/`global-shortcut` from window permissions (they're backend plugins, not window perms). Keep `store` only where settings are touched (main).
- [ ] Verify the build.rs AppManifest approach is valid for the resolved Tauri 2.x (the `tauri_build::try_build` with `Attributes::new().app_manifest(...)` pattern). If `AppManifest::commands` isn't the right API name, use the equivalent in the resolved version. Each window's JS must still resolve its scoped commands — a missing scope = command rejected at runtime, so exercise all three windows in dev.
- [ ] Commit.

## Task 5: GitHub Actions release workflow (correct signing + runner)

**Files:** `.github/workflows/release.yml`

- [ ] Matrix (current runners): `macos-latest` (arm64), **`macos-15-intel`** (NOT retired `macos-13`), `windows-latest` (x86_64-pc-windows-msvc).
- [ ] macOS signing+notarization — the FULL pinned steps (no "if Tauri doesn't auto-import" TBD):
```yaml
- name: Import signing cert + build (macOS)
  if: startsWith(matrix.os, 'macos')
  env:
    APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}            # base64 .p12
    APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
    KEYCHAIN_PASSWORD: ${{ secrets.KEYCHAIN_PASSWORD }}
    APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }} # "Developer ID Application: ..."
    APPLE_ID: ${{ secrets.APPLE_ID }}
    APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}                  # app-specific
    APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
  run: |
    # Decode the p12, create a temp keychain, import the cert, allow codesign.
    CERT_PATH=$(mktemp islandpot-cert.XXXXXX.p12)
    echo "$APPLE_CERTIFICATE" | base64 --decode > "$CERT_PATH"
    security create-keychain -p "$KEYCHAIN_PASSWORD" build.keychain
    security default-keychain -s build.keychain
    security unlock-keychain -p "$KEYCHAIN_PASSWORD" build.keychain
    security import "$CERT_PATH" -P "$APPLE_CERTIFICATE_PASSWORD" -k build.keychain -T /usr/bin/codesign
    security set-key-partition-list -S apple-tool:,apple: -s -k "$KEYCHAIN_PASSWORD" build.keychain
    rm -f "$CERT_PATH"
    # Tauri reads APPLE_SIGNING_IDENTITY + the notarization creds from env.
    pnpm tauri build --target ${{ matrix.target }}
    # Cleanup.
    security delete-keychain build.keychain || true
```
- [ ] Windows signing (Authenticode, NOT the Tauri updater key): import a PFX code-signing cert in CI, then configure Tauri's `bundle.windows` with **`certificateThumbprint`** (the imported cert's thumbprint) + `digestAlgorithm` (sha256) + `timestampUrl` (e.g. http://timestamp.digicert.com). `WindowsConfig` exposes `certificateThumbprint`, `certificatePath`/`certificatePassword` (alternative to thumbprint), `digestAlgorithm`, `timestampUrl`, and `signCommand` (fully custom). Pick: **PFX import → `certificateThumbprint`** (the documented path). `TAURI_SIGNING_PRIVATE_KEY*` is the UPDATER signature (separate plugin) — do NOT conflate. If no Authenticode cert, the installer builds unsigned (users see a SmartScreen warning — document it).
- [ ] Artifacts upload. Lint YAML. Commit.

## Task 6: README + E2E + final review

- [ ] README release docs (mac + Windows signing, distinct from updater key; Intel+arm64; CI tag trigger). Commit.
- [ ] Manual E2E on macOS release .app (Accessibility grant; selection AX-first + copy-fallback; input; clipboard; fallback; dict; keystore recovery; latest-wins). DMG may fail on hdiutil locally (env issue).
- [ ] Final review (opus): Windows ACL/atomic_replace correctness (can't run locally — verify the code + CI is set up to run it), CSP not breaking UI, capabilities decision documented, CI signing distinct from updater key. Merge.

---

## Self-Review
- **Round-2 fixes covered:** stub atomic_replace (Task 1); Authenticode-not-updater-key (Task 5); single-`env:` macOS CI (Task 5); `macos-15-intel` (Task 5); capabilities mechanism corrected (Task 4).
- **fsync (P2):** deferred to a hardening slice (honest).
- **No false claims:** this rev explicitly notes the stub exists and Task 1 replaces it.

## Execution Handoff
Subagent-Driven. **Execute only after the round-2 code fixes (Part A) are re-reviewed + approved.**
