# Phase 4: Windows Parity + Cross-Platform Packaging — Implementation Plan (rev 2)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Status:** rev 2 — rewritten after the 2026-07-31 review found the icacls fallback, the shell-substitution `${APPLE_SIGNING_IDENTITY}`, the missing cert-import CI steps, and the absent CSP/Intel tasks. Those are all fixed here.

**Goal:** Make IslandPot build and run on **Windows** (fill the macOS-only stubs correctly — Win32 ACL API, not icacls) and produce **signed/notarized release bundles** for both platforms via a correct CI (cert import + real env-var signing), with **CSP** enabled and per-window minimal capabilities, and **Intel macOS** in the matrix.

**Architecture:** (1) Windows parity: real `SetNamedSecurityInfoW` DACL (current-user SID, full control incl. delete, PROTECTED_DACL_SECURITY_INFORMATION to block inheritance) — NOT icacls. `ReplaceFileW` (update) / `MoveFileExW` (first-create) already in Task 1. (2) Packaging: signing driven by env vars (no JSON `${...}`); CI imports the macOS certificate into a temp keychain + sets Windows signing vars. (3) Hardening: real CSP + split capabilities. (4) Intel + arm64 macOS.

**Tech Stack:** Rust 1.95 · Tauri 2 bundler · `windows-sys` (Win32 ACL: `Win32_Security_Authorization`, `Win32_Security`) · GitHub Actions (macos-latest arm64 + macos-13 Intel + windows-latest).

**Facts verified upfront:** Tauri signing env vars: `APPLE_SIGNING_IDENTITY`/`APPLE_CERTIFICATE`+`APPLE_CERTIFICATE_PASSWORD` (sign), `APPLE_ID`+`APPLE_PASSWORD`+`APPLE_TEAM_ID` (notarize); Windows: `TAURI_SIGNING_PRIVATE_KEY`+`TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (MSI/NSIS). `bundle.macOS.signingIdentity` is NOT shell-substituted from `${...}` — drive signing purely via env vars (omit the JSON field). Win32 ACL: `SetNamedSecurityInfoW` with a `TRUSTEE` for the current user SID + an explicit-ACE `EXPLICIT_ACCESS_W` (SET_ACCESS, all rights) + `PROTECTED_DACL_SECURITY_INFORMATION`.

---

## File Structure

**Modify:**
- `src-tauri/src/keystore.rs` — Windows `set_file_perms`: real `SetNamedSecurityInfoW` DACL (replace the no-op). `atomic_replace` already done in P1 Task 1.
- `src-tauri/Cargo.toml` — add windows-sys features: `Win32_Security_Authorization`, `Win32_Security`.
- `src-tauri/tauri.conf.json` — real CSP (`app.security.csp`); remove any `${APPLE_SIGNING_IDENTITY}` (drive via env); split capabilities.
- `src-tauri/capabilities/` — per-window capability files (main/popup/input) with only the permissions each window needs; drop blanket `core:default` where over-broad.

**Create/modify:**
- `.github/workflows/release.yml` — matrix (arm64 + Intel macOS + Windows); macOS cert import to temp keychain; Windows signing env; artifact upload.

---

## Task 1: Windows keystore permissions via Win32 ACL API

**Files:** `src-tauri/src/keystore.rs`, `src-tauri/Cargo.toml`

- [ ] **Step 1: windows-sys features.** Add `"Win32_Security_Authorization"` and `"Win32_Security"` to the `windows-sys` features list (alongside the existing `Win32_System_DataExchange`, `Win32_Foundation`, `Win32_Storage_FileSystem`).

- [ ] **Step 2: implement `set_file_perms` (Windows) via SetNamedSecurityInfoW.** Replace the no-op `#[cfg(not(target_os = "macos"))]` stub. The DACL grants the current user FULL control (including delete — required for `ReplaceFileW` updates) and blocks inheritance:
  - Get the current user SID via `GetUserNameW` → `LookupAccountNameW` → SID. (Or `OpenThreadToken`/`GetTokenInformation`→`TokenUser`; either is acceptable — pick the simpler.)
  - Build a `TRUSTEE_W` for that SID.
  - Build one `EXPLICIT_ACCESS_W`: `grfAccessMode = SET_ACCESS`, `grfAccessPermissions = GENERIC_ALL` (FILE_ALL_ACCESS), `grfInheritance = NO_INHERITANCE`.
  - `BuildExplicitAccessWithName` (or hand-assemble the `EXPLICIT_ACCESS_W` array of 1).
  - `SetEntriesInAclW(1, &ea, null, &new_dacl)` → produces a new DACL.
  - `SetNamedSecurityInfoW(path, SE_FILE_OBJECT, DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION, null, null, new_dacl, null)`. The `PROTECTED_DACL_SECURITY_INFORMATION` bit BLOCKS inheritance (§A requirement).
  - Free the DACL (`LocalFree`).
  Apply this to BOTH the keystore dir (on `new`) and the file (on `store_locked`).

- [ ] **Step 3: tests.** Add a Windows-only test (`#[cfg(target_os="windows")]`) that creates a keystore in a temp dir, writes a key, then verifies the ACL via `GetNamedSecurityInfoW` → the DACL has exactly one explicit ACE for the current user with full control AND the DACL is protected (no inherited ACEs). Run on a Windows runner (Task 5 CI) — can't run locally on macOS.

- [ ] **Step 4: cargo check (macOS) — Windows branch cfg-gated, passes.** Commit.
```bash
git add -A && git commit -m "feat(keystore): Windows file/dir ACL via SetNamedSecurityInfoW (current-user, protected)"
```

> **NOT icacls.** The rev-1 `icacls` fallback is removed: it doesn't expand `%USERNAME%`, `(R,W)` lacks delete (breaks ReplaceFileW), and shelling out on a security-critical path is wrong. Use the Win32 ACL API.

---

## Task 2: Release bundle config — real CSP + env-driven signing

**Files:** `src-tauri/tauri.conf.json`

- [ ] **Step 2a: CSP.** Set `app.security.csp` to a real policy. SolidJS + the providers we call (HTTPS only, loopback Ollama) need: default-src 'self'; connect-src 'self' https: http://localhost:* http://127.0.0.1:*; script-src 'self' (Vite injects inline in dev — for dev use the Tauri dev CSP or `unsafe-inline` dev-only; for release, 'self'). Start with:
```json
"security": {
  "csp": "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; connect-src 'self' https: http://localhost:* http://127.0.0.1:*"
}
```
Verify the popup/input windows still work (they only invoke commands + emit; no external connections). If a SolidJS hydration needs `unsafe-inline` script in release, loosen `script-src` minimally and document why.

- [ ] **Step 2b: signing — env-driven, NO JSON interpolation.** Do NOT add `${APPLE_SIGNING_IDENTITY}` to `bundle.macOS.signingIdentity` (Tauri does not shell-substitute it). Omit the field; signing is driven entirely by the env vars set in CI (`APPLE_SIGNING_IDENTITY`, etc.). `bundle.macOS.minimumSystemVersion: "11.0"` stays.

- [ ] **Step 2c: verify dev still launches.** `pnpm tauri dev` briefly → window opens (CSP doesn't break dev; Tauri relaxes CSP in dev). Commit.

---

## Task 3: Split capabilities per window

**Files:** `src-tauri/capabilities/`

- [ ] **Step 1: audit current `capabilities/default.json`** — it grants `windows: ["main","popup","input"]` with `core:default`, `opener:default`, `global-shortcut:default`, `store:default` to ALL three. Split:
  - `capabilities/main.json` — window `main`: store (settings), opener, global-shortcut (none needed on main actually), core window/event perms it uses.
  - `capabilities/popup.json` — window `popup`: only the core perms to hide on focus + listen to events. No store, no opener.
  - `capabilities/input.json` — window `input`: invoke translate_default + core.
  - Remove `opener` from any window that doesn't open URLs (none currently do). Remove `global-shortcut` from windows (it's a backend plugin, not a window perm).
- [ ] **Step 2: verify each window's commands still resolve** (capabilities scope which commands a window's JS can invoke — a missing perm = command rejected). `pnpm tauri dev` + exercise popup/input/main. Commit.

---

## Task 4: Verify Windows compiles (cross-check)

- [ ] **Step 1:** `rustup target add x86_64-pc-windows-gnu` (if not present) + `cargo check --target x86_64-pc-windows-gnu` to type-check the Windows cfg branches (ACL API, atomic_replace, clipboard sequence, MachineGuid). Fix any errors surfaced. If the gnu target trips on windows-sys (it sometimes does for ACL APIs), defer real validation to the Windows CI runner (Task 5) and note it. Commit any fixes.

---

## Task 5: GitHub Actions release workflow (correct signing)

**Files:** `.github/workflows/release.yml`

- [ ] **Step 1: matrix** — arm64 + Intel macOS + Windows:
```yaml
strategy:
  fail-fast: false
  matrix:
    include:
      - { os: macos-latest,   target: aarch64-apple-darwin }
      - { os: macos-13,       target: x86_64-apple-darwin }   # Intel
      - { os: windows-latest, target: x86_64-pc-windows-msvc }
```

- [ ] **Step 2: macOS signing + notarization (correct).** Import the certificate into a temp keychain and pass the identity + notarization creds as env to the build:
```yaml
- name: Import macOS signing cert
  if: startsWith(matrix.os, 'macos') && env.APPLE_CERTIFICATE != ''
  env:
    APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}      # base64 .p12
    APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
  run: |
    cert=$(mktemp /tmp/cert.p12)
    echo "$APPLE_CERTIFICATE" | base64 --decode > "$cert"
    security create-keychain -p "$KEYCHAIN_PW" build.keychain
    security default-keychain -s build.keychain
    security unlock-keychain -p "$KEYCHAIN_PW" build.keychain
    security import "$cert" -P "$APPLE_CERTIFICATE_PASSWORD" -k build.keychain -T /usr/bin/codesign
    security set-key-partition-list -S apple-tool:,apple: -s -k "$KEYCHAIN_PW" build.keychain
  env: { KEYCHAIN_PW: ${{ secrets.KEYCHAIN_PW }} }
- name: Build
  run: pnpm tauri build --target ${{ matrix.target }}
  env:
    APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
    APPLE_ID: ${{ secrets.APPLE_ID }}
    APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
    APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
    # Windows (ignored on mac):
    TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
    TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
```
> Key points: the cert is imported to a keychain (not just echoed); `APPLE_CERTIFICATE_PASSWORD` IS passed; the build reads the identity + notarization vars; empty secrets → unsigned dev bundle (still builds).

- [ ] **Step 3: Windows signing.** Set `TAURI_SIGNING_PRIVATE_KEY` + `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (already in the Build env above) so the MSI/NSIS bundle is signed (Smartscreen-relevant). Document that an EV cert needs hardware/attestation signing (out of CI scope); a standard cert is what CI can do.

- [ ] **Step 4: artifacts.** Upload `.app`/`.dmg`/`.msi`/`.exe` from each target. Lint the YAML. Commit.

---

## Task 6: README release docs + manual E2E + final review

- [ ] **Step 1:** README "Building/Releasing" section: dev, local bundle, signing env vars (mac + Windows) with the note they're optional, CI trigger (push `v*` tag), Intel+arm64. Commit.
- [ ] **Step 2:** Manual E2E on macOS release build (`pnpm tauri build` → run the .app): grant Accessibility, smoke-test selection (incl. AX-first on a rich-text app), input, clipboard, fallback, dict, keystore recovery. Confirm the DMG (may fail locally on hdiutil — env issue, .app is the artifact).
- [ ] **Step 3:** Final review (opus) — focus on Windows ACL correctness (the part we can't run locally), CSP not breaking the UI, capability split not dropping a needed perm, and CI cert steps. Merge to main.

---

## Self-Review

- **Review #11/#12 coverage:** icacls removed (Task 1 uses SetNamedSecurityInfoW); `${APPLE_SIGNING_IDENTITY}` removed (Task 2b, env-driven); CI cert-import added (Task 5 step 2, with APPLE_CERTIFICATE_PASSWORD); Windows signing vars added (Task 5 step 3); CSP task (Task 2a); Intel (Task 5 matrix); capability split (Task 3).
- **fsync (review P2):** noted but NOT in this plan — it's P2 (durability, not release-blocking); will be added in a hardening slice alongside the keystore envelope-size cap. The plan honestly defers it.
- **Placeholder scan:** Task 1's SID-acquisition offers two approaches (pick simpler) — not a TBD. Task 5's EV-cert is scoped out with a note. No lingering TODOs.
- **Honest risk:** Windows ACL code can't be fully validated locally (cfg-gated); Task 4 cross-check + Task 5 Windows runner are the real validation.

---

## Execution Handoff

Plan saved to `docs/superpowers/plans/2026-07-30-phase4-windows-parity-packaging.md` (rev 2). Subagent-Driven recommended once the P1 code fixes (separate plan) are reviewed + merged.
