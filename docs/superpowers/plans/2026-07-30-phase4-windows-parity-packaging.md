# Phase 4: Windows Parity + Cross-Platform Packaging — Implementation Plan (rev 3)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`) syntax for tracking.

**Status:** rev 3 — rewrites rev-2 after the 2026-07-31 round-2 review found it un-executable: (a) Windows `atomic_replace` is still a STUB (rev-2 falsely claimed it was done), (b) `TAURI_SIGNING_PRIVATE_KEY*` is the Tauri UPDATER key, not Windows Authenticode, (c) macOS CI YAML had two sibling `env:` blocks + invalid `mktemp` template, (d) `macos-13` runner retired 2025-12-04, (e) capabilities description was wrong (custom commands are global by default; per-window needs `AppManifest::commands`).

**Goal:** Windows builds (real `atomic_replace` + ACL), correct signing (macOS notarization via cert-import; Windows Authenticode via PFX/signtool — distinct from the updater key), and a clean CI matrix (arm64 + Intel macOS via `macos-15-intel`, Windows).

**Facts verified (round-2):** `atomic_replace` non-mac returns Err (keystore.rs:444 stub). Tauri updater keys (`TAURI_SIGNING_PRIVATE_KEY*`) sign the updater bundle, NOT Windows installers — Authenticode needs a PFX import + `signtool` (or Tauri's `signCommand`/`certificateThumbprint`). Tauri macOS signing: base64 cert → keychain import → `APPLE_SIGNING_IDENTITY` env. `macos-13` retired 2025-12-04. Custom `invoke_handler` commands are reachable from ALL local windows by default; per-window restriction requires defining them as app commands in a manifest + scoping per capability.

---

## Task 1: Windows atomic_replace (real, not the stub)

**Files:** `src-tauri/src/keystore.rs`, `src-tauri/Cargo.toml`

- [ ] Replace the `#[cfg(not(target_os = "macos"))] fn atomic_replace(...) -> Err("not implemented")` STUB with platform-specific real impls:
  - `#[cfg(target_os = "windows")]`: `MoveFileExW` (first-create, `MOVEFILE_REPLACE_EXISTING`) if dst absent, else `ReplaceFileW` (update). Needs `windows-sys` feature `Win32_Storage_FileSystem` (already present).
  - `#[cfg(not(any(target_os = "macos", target_os = "windows")))]`: keep the Err stub.
- [ ] Test (Windows runner, Task 5 CI): keystore write → keystore.json replaced atomically; a concurrent writer can't observe a half-written file. Can't run locally on macOS.
- [ ] Commit.

## Task 2: Windows file/dir ACL via SetNamedSecurityInfoW

**Files:** `src-tauri/src/keystore.rs`, `src-tauri/Cargo.toml`

- [ ] windows-sys features: add `Win32_Security_Authorization`, `Win32_Security`.
- [ ] Real `set_file_perms` (Windows): `SetNamedSecurityInfoW` with a DACL of one explicit ACE for the current-user SID (GENERIC_ALL, incl. delete for ReplaceFileW) + `PROTECTED_DACL_SECURITY_INFORMATION` (block inheritance). NOT icacls. Apply to dir (on `new`) + file (on `store_locked`).
- [ ] Windows-only test (CI): verify via `GetNamedSecurityInfoW` the DACL has exactly one explicit user ACE, full control, and is protected.
- [ ] Commit.

## Task 3: Release bundle config — CSP + env-driven signing

**Files:** `src-tauri/tauri.conf.json`

- [ ] CSP: `app.security.csp` = `"default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; connect-src 'self' https: http://localhost:* http://127.0.0.1:*"`. Verify dev + the three windows still work (Tauri relaxes CSP in dev).
- [ ] Signing: drive purely via env vars (NO `${APPLE_SIGNING_IDENTITY}` in JSON). Keep `bundle.macOS.minimumSystemVersion: "11.0"`.
- [ ] Commit.

## Task 4: Capabilities (correct mechanism)

**Files:** `src-tauri/capabilities/`

- [ ] Per the round-2 review: custom `invoke_handler` commands are callable from ALL local windows by default; restricting them per-window requires defining them as **app commands** in `tauri.conf.json` `app.commands` (or a manifest) and scoping per capability. For v1, two acceptable options (pick one, document):
  - **(a) Leave commands global** (v1 pragmatic), and instead harden via CSP + minimal PLUGIN permissions per window (drop `opener`/`global-shortcut` from windows that don't need them; keep `store` on main only). Document that command-level restriction is post-v1.
  - **(b) Define app commands + scope per capability** (the rigorous path) — more work.
  - Recommend (a) for v1 with a clear note; do (b) if time allows. Split `capabilities/default.json` into per-window files with the minimal plugin perms each.
- [ ] Commit.

## Task 5: GitHub Actions release workflow (correct signing + runner)

**Files:** `.github/workflows/release.yml`

- [ ] Matrix (current runners): `macos-latest` (arm64), **`macos-15-intel`** (NOT retired `macos-13`), `windows-latest` (x86_64-pc-windows-msvc).
- [ ] macOS signing+notarization (official structure, single env block):
```yaml
- name: Build (macOS)
  if: startsWith(matrix.os, 'macos')
  env:
    APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
    APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
    APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
    APPLE_ID: ${{ secrets.APPLE_ID }}
    APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
    APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
  run: pnpm tauri build --target ${{ matrix.target }}
```
  Tauri reads `APPLE_CERTIFICATE` (base64 p12) + `APPLE_CERTIFICATE_PASSWORD` and imports to a keychain itself when these are set (per Tauri docs) — verify against the resolved Tauri version's behavior; if Tauri does NOT auto-import, add the explicit `security create-keychain`/`import` step in the SAME job (one `env:` block, no `mktemp` template).
- [ ] Windows signing (Authenticode, NOT the updater key): if a code-signing PFX is available, configure Tauri's bundle signing via a `signCommand` (signtool) or `certificateThumbprint`/`certificatePath` in `tauri.conf.json` bundle.windows. The `TAURI_SIGNING_PRIVATE_KEY*` is separate (updater only) — document both. If no cert, the build still produces an unsigned installer (note the Smartscreen warning for users).
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
