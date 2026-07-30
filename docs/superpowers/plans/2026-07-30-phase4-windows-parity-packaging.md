# Phase 4: Windows Parity + Cross-Platform Packaging — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make IslandPot build and run on **Windows** (fill the macOS-only stubs) and produce **signed/notarized release bundles** for both platforms — the "first usable cut" that can actually ship to other users' machines.

**Architecture:** This phase has two halves. (1) **Windows parity**: the macOS-first code left Windows stubs in `keystore.rs` (atomic_replace, MachineGuid, DACL perms), `clipboard.rs` (sequence number already implemented for Windows), and `dict.rs` (Windows has no equivalent — return None, already done). These need real Windows implementations so the app compiles AND runs correctly on Windows. (2) **Packaging**: release build config (bundle targets, minimum macOS version, icons), code signing + notarization env wiring (CI-ready, not requiring the user to have certs locally), and a GitHub Actions workflow that builds both platforms.

**Tech Stack:** Rust 1.95 (cross-compile via GitHub runners, not local) · Tauri 2 bundler · `windows-sys` (already a dep) for the Windows keystore ops · GitHub Actions (macos-latest + windows-latest runners).

**Spec reference:** `docs/superpowers/specs/2026-07-30-islandpot-v1-design.md` — §A keystore (Windows: MachineGuid identity, Credential Manager → here self-encrypted file with DACL, ReplaceFileW for updates / MoveFileExW for first-create); §Privacy (HTTPS-only, no telemetry).

**Facts verified upfront (2026-07-30):** Tauri 2 macOS signing/notarization env vars: `APPLE_SIGNING_IDENTITY` / `APPLE_CERTIFICATE`+`APPLE_CERTIFICATE_PASSWORD` (signing), `APPLE_ID`+`APPLE_PASSWORD`+`APPLE_TEAM_ID` OR `APPLE_API_ISSUER`+`APPLE_API_KEY` (notarization); config key `bundle.macOS.signingIdentity`. Bundle command: `tauri bundle --bundles app,dmg`. `main.rs` already has `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`. Platform stubs present: keystore atomic_replace (non-mac errors), MachineGuid read (reg.exe), DACL perms (not set on Windows).

---

## File Structure

**Modify:**
- `src-tauri/src/keystore.rs` — implement Windows `atomic_replace` (ReplaceFileW update / MoveFileExW first-create), Windows file/dir perms (DACL via windows-sys).
- `src-tauri/tauri.conf.json` — add `bundle.macOS.minimumSystemVersion` + `bundle.macOS.signingIdentity` (env-driven); set `bundle.targets` explicitly per-platform.
- `src-tauri/Cargo.toml` — ensure `windows-sys` features cover `ReplaceFileW`/`MoveFileExW`/DACL APIs (`Win32_Storage_FileSystem`, `Win32_Security_Authorization`, `Win32_System_IO`).
- `README.md` — document build/release commands + signing env vars.
- `src-tauri/src/lib.rs` — (likely no change; verify the global-shortcut/clipboard/dict code already gates correctly for Windows).

**Create:**
- `.github/workflows/release.yml` — build + bundle + (optionally) sign/notarize on both macos-latest and windows-latest runners, upload artifacts.

---

## Task 1: Windows keystore — atomic_replace (ReplaceFileW / MoveFileExW)

**Files:** Modify `src-tauri/src/keystore.rs`, `src-tauri/Cargo.toml`

The current `atomic_replace` is macOS-only; the `#[cfg(not(target_os = "macos"))]` branch returns `Err("not implemented")`. Implement the Windows branch per spec §A (`ReplaceFileW` for updates — target must exist; `MoveFileExW` for first-create).

- [ ] **Step 1: Add windows-sys features.** In `src-tauri/Cargo.toml`, the `windows-sys` dep currently has `features = ["Win32_System_DataExchange", "Win32_Foundation"]` (for clipboard). ADD: `"Win32_Storage_FileSystem"` (ReplaceFileW/MoveFileExW live here). Result:
```toml
windows-sys = { version = "0.59", features = ["Win32_System_DataExchange", "Win32_Foundation", "Win32_Storage_FileSystem"] }
```

- [ ] **Step 2: Replace the Windows branch of `atomic_replace`.** Find the `#[cfg(not(target_os = "macos"))] fn atomic_replace(...)` stub in keystore.rs and replace with the real Windows impl:
```rust
#[cfg(target_os = "windows")]
fn atomic_replace(src: &std::path::Path, dst: &std::path::Path) -> Result<(), KeystoreError> {
    use std::os::windows::ffi::OsStrExt;
    fn wide(p: &std::path::Path) -> Vec<u16> {
        p.as_os_str().encode_wide().chain(std::iter::once(0)).collect()
    }
    unsafe {
        if dst.exists() {
            // Update path: ReplaceFileW (requires target to exist).
            let r = windows_sys::Win32::Storage::FileSystem::ReplaceFileW(
                wide(dst).as_ptr(), wide(src).as_ptr(), std::ptr::null(), 0, std::ptr::null_mut(), std::ptr::null_mut(),
            );
            if r == 0 {
                return Err(KeystoreError::Io(std::io::Error::last_os_error()));
            }
        } else {
            // First-create path: MoveFileExW with REPLACE_EXISTING (harmless if target absent).
            const MOVEFILE_REPLACE_EXISTING: u32 = 1;
            let r = windows_sys::Win32::Storage::FileSystem::MoveFileExW(
                wide(src).as_ptr(), wide(dst).as_ptr(), MOVEFILE_REPLACE_EXISTING,
            );
            if r == 0 {
                return Err(KeystoreError::Io(std::io::Error::last_os_error()));
            }
        }
    }
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn atomic_replace(_src: &std::path::Path, _dst: &std::path::Path) -> Result<(), KeystoreError> {
    Err(KeystoreError::Envelope("atomic_replace not implemented on this platform".into()))
}
```
VERIFY the exact signatures of `ReplaceFileW` and `MoveFileExW` in the resolved windows-sys 0.59 (`find ~/.cargo/registry/src -path '*windows-sys-0.59*' -name '*.rs'` → grep). `ReplaceFileW`'s last two params are `LPVOID` (may be `*mut c_void` / `*mut core::ffi::c_void`); pass null. `MoveFileExW` returns `BOOL` (0 = fail). The `MOVEFILE_REPLACE_EXISTING` constant — confirm its value in windows-sys (`Win32::Storage::FileSystem::MOVEFILE_REPLACE_EXISTING` may be exported as a const; if so, use the qualified name instead of the local `const`). Report which you used.

- [ ] **Step 3: cargo check on macOS.** The Windows branch is cfg-gated out of the macOS build, so `cargo check` should still pass (it just won't compile the Windows code locally). `cd /Users/daoyu/Code/projects/islandpot/src-tauri && cargo check` → Finished. (The real compile-validation happens in CI Task 5; we can't cross-compile windows easily here without the target installed. Optionally `rustup target add x86_64-pc-windows-gnu` + `cargo check --target ...` but that's heavy — defer to CI.)

- [ ] **Step 4: Commit.**
```bash
cd /Users/daoyu/Code/projects/islandpot && git checkout -b phase4 && git add src-tauri/src/keystore.rs src-tauri/Cargo.toml src-tauri/Cargo.lock && git -c user.name=daoyu -c user.email=daoyu@local commit -m "feat(keystore): Windows atomic_replace (ReplaceFileW/MoveFileExW)"
```
> Create `phase4` branch here.

---

## Task 2: Windows keystore — file/dir permissions (DACL)

**Files:** Modify `src-tauri/src/keystore.rs`, `src-tauri/Cargo.toml`

The macOS `set_dir_perms` (0700) / `set_file_perms_macos` (0600) have no-op Windows stubs. Per spec §A, Windows must set a DACL granting access to the **current user only** (no inheritance).

- [ ] **Step 1: Add windows-sys security features.** ADD to `windows-sys` features: `"Win32_Security_Authorization"`, `"Win32_Security"`. Result includes those + the Task 1 ones.

- [ ] **Step 2: Implement a Windows `set_user_only_perms` helper + wire it.** In keystore.rs, replace the Windows no-op `set_dir_perms`/`set_file_perms_macos` stubs. The robust-but-involved path is `SetNamedSecurityInfoW` with a DACL containing only the current user. A SIMPLER acceptable approach (verify it satisfies §A's intent "current user only"): use `icacls` via `Command` (shell out) — but that's fragile. PREFER the API: build a DACL with one explicit ACE for the current user (full control) and set `PROTECTED_DACL_SECURITY_INFORMATION` (blocks inheritance). This is ~40 lines of unsafe FFI.

Given the complexity and that this is the riskiest Windows task, **if a clean SetNamedSecurityInfoW implementation proves too involved for one task, fall back to a documented, reviewed simplification**: set the file ACL via PowerShell `icacls` invoked from Rust (`Command::new("icacls")`), restricting to the current user (`icacls "<file>" /inheritance:r /grant:r "%USERNAME%:(R,W)"`). This is less elegant but matches §A's intent (current-user-only, no inheritance) and is shippable. Note which you used; the API path is preferred for a security-sensitive file.

Add Windows branches:
```rust
#[cfg(target_os = "windows")]
fn set_dir_perms(dir: &std::path::Path) -> Result<(), KeystoreError> {
    set_user_only_acl(dir)
}
#[cfg(target_os = "windows")]
fn set_file_perms_macos(&self, p: &std::path::Path) -> Result<(), KeystoreError> {
    // (rename the method or add a Windows equivalent — see note)
    set_user_only_acl(p)
}
```
> NOTE: the method is named `set_file_perms_macos` but is called for both platforms in `store()`. Rename to `set_file_perms` (platform-internal) and have the macOS + Windows bodies differ by cfg. Do the rename as part of this task.

`set_user_only_acl` — implement via SetNamedSecurityInfoW OR icacls (your call per the above). Report which.

- [ ] **Step 3: cargo check (macOS) — Windows branch cfg-gated out, should still pass.** Commit.

```bash
git add src-tauri/src/keystore.rs src-tauri/Cargo.toml src-tauri/Cargo.lock && git -c user.name=daoyu -c user.email=daoyu@local commit -m "feat(keystore): Windows file/dir DACL (current-user-only)"
```

---

## Task 3: Release bundle config (tauri.conf.json)

**Files:** Modify `src-tauri/tauri.conf.json`

- [ ] **Step 1: Add `bundle.macOS`.** Set a minimum macOS version + env-driven signing identity. Replace the `"bundle"` block's contents to add a `"macOS"` sub-object (keep `active`, `targets`, `icon`):
```json
"bundle": {
  "active": true,
  "targets": "all",
  "icon": [ ... existing icons ... ],
  "macOS": {
    "minimumSystemVersion": "11.0",
    "signingIdentity": "${APPLE_SIGNING_IDENTITY}"
  }
}
```
The `${APPLE_SIGNING_IDENTITY}` is interpolated by Tauri from the env var at build time; if unset, it's left empty (unsigned dev build) — that's fine for local dev. `minimumSystemVersion: 11.0` (Big Sur) matches our use of modern APIs (Accessibility, CoreServices).

- [ ] **Step 2: Verify the JSON is valid + dev build still works.** `cd /Users/daoyu/Code/projects/islandpot && pnpm tauri dev` briefly (then Ctrl-C) — confirm it still launches (no config parse error). OR `cargo check` in src-tauri (the conf is consumed at build via generate_context). Then commit.
```bash
git add src-tauri/tauri.conf.json && git -c user.name=daoyu -c user.email=daoyu@local commit -m "build: bundle config (minimumSystemVersion 11, env-driven signing)"
```

---

## Task 4: Verify Windows compiles (best-effort cross-check)

**Files:** none (verification)

This is the catch for Tasks 1+2: the Windows code is cfg-gated and won't compile on macOS, so we can't fully validate it locally. Two options:
- **(a) Cross-check via rustup target** (preferred if feasible): `rustup target add x86_64-pc-windows-gnu` then `cd src-tauri && cargo check --target x86_64-pc-windows-gnu`. This compiles the Windows cfg branches. It may fail to link (no Windows libs) but `cargo check` doesn't link — it type-checks. If type errors surface in the Windows branches, fix them.
- **(b) Defer to CI** (Task 5): if the cross-target toolchain isn't readily available or the gnu target trips on windows-sys expectations, document that Windows compile-validation happens in the CI workflow and move on.

- [ ] **Step 1:** Attempt (a). If `cargo check --target x86_64-pc-windows-gnu` type-checks the Windows branches, great. If it surfaces errors (e.g. wrong API signature, missing feature flag), fix them in keystore.rs and re-check. If the target/toolchain isn't available or it's a rabbit hole (windows-sys gnu quirks), switch to (b) and note it.
- [ ] **Step 2:** If fixes were made, commit them. Otherwise no commit (just a verification step). Report the outcome.

---

## Task 5: GitHub Actions release workflow

**Files:** Create `.github/workflows/release.yml`

- [ ] **Step 1: Write the workflow.** Two jobs (matrix: macos-latest, windows-latest). Each: checkout, setup-node + setup pnpm, setup Rust (stable), `pnpm install`, `pnpm tauri build`. Upload the bundle artifacts. Signing/notarization via env secrets (only set if the repo has them; the build still produces unsigned artifacts otherwise — fine for the first cut, signing is opt-in via secrets).

```yaml
name: release
on:
  push:
    tags: ['v*']
  workflow_dispatch:

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: 22 }
      - uses: pnpm/action-setup@v4
        with: { version: 11 }
      - run: pnpm install --frozen-lockfile
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      # macOS signing/notarization — only if secrets present (opt-in).
      - name: macOS sign+notarize env
        if: matrix.os == 'macos-latest' && env.APPLE_CERTIFICATE != ''
        env:
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
        run: echo "signing enabled"
      - name: Build
        run: pnpm tauri build --target ${{ matrix.target }}
        env:
          # Tauri reads these if present; absent = unsigned dev bundle.
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
      - uses: actions/upload-artifact@v4
        with:
          name: islandpot-${{ matrix.os }}
          path: |
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.app
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.dmg
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.msi
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*.exe
```

> NOTE for implementer: the macOS runner is `macos-latest` which is arm64 (aarch64-apple-darwin) — confirms our local build target. Windows uses `x86_64-pc-windows-msvc` (the standard Tauri Windows target). Verify pnpm action-setup version matches the local pnpm (11). The `APPLE_CERTIFICATE` env gate in the "sign+notarize env" step is a placeholder check — the actual signing happens in the Build step via Tauri reading the env vars. If you want conditional signing only when secrets exist, the env vars being empty/unset already makes Tauri produce unsigned bundles, so the gate is cosmetic; simplify if desired.

- [ ] **Step 2: Lint the workflow YAML** (syntax only — can't run it without a push). `python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/release.yml'))"` (or `cat` and eyeball). Commit.
```bash
git add .github/workflows/release.yml && git -c user.name=daoyu -c user.email=daoyu@local commit -m "ci: release workflow (macos+windows build, optional signing)"
```

---

## Task 6: README release docs + manual E2E + final review

**Files:** Modify `README.md`

- [ ] **Step 1: Document build + release.** Add a "Building / Releasing" section to README: dev (`pnpm tauri dev`), local bundle (`pnpm tauri build`), the signing env vars (APPLE_SIGNING_IDENTITY etc.) with a note that they're optional (absent = unsigned), and the CI workflow (push a `v*` tag to trigger). Commit.
```bash
git add README.md && git -c user.name=daoyu -c user.email=daoyu@local commit -m "docs: build/release commands + signing env vars"
```

- [ ] **Step 2: Manual E2E (macOS, local).** `pnpm tauri build` → confirm it produces `islandpot.app` + `islandpot.dmg` (the DMG failed earlier due to hdiutil in this env; verify the .app builds). Grant Accessibility, smoke-test the v1 translation core (selection/input/clipboard + fallback) one more time on the release build.

- [ ] **Step 3: Final review** (opus) of the phase — focus on Windows-correctness (the cfg-gated code we couldn't fully test locally) + bundle config sanity. Then merge to main.

---

## Self-Review (run after writing; fix inline)

- **Spec coverage:** §A Windows keystore (MachineGuid identity — ALREADY implemented in Phase 1's `read_windows_machine_guid`; atomic_replace → Task 1; DACL perms → Task 2). §Privacy unchanged. **Gap:** Windows `clipboard::sequence()` uses `GetClipboardSequenceNumber` — verify it's implemented (Phase 2a Task 3 added it under `#[cfg(target_os = "windows")]`). Windows `dict::lookup` returns None (no macOS equivalent — acceptable, already done). Windows `simulate_copy` uses Ctrl+C (Phase 2a — verify the cfg branch).
- **Placeholder scan:** Task 2 has an explicit either/or (API vs icacls) with a stated preference — not a TBD. Task 4 is an explicit verification-with-fallback. Task 5's "sign+notarize env" step is flagged cosmetic. No lingering TODOs.
- **Honest risk:** the Windows code (Tasks 1-2) is cfg-gated and can't be fully validated on macOS — Task 4 mitigates with a cross-target check, and Task 5's CI gives the real validation. This is the inherent risk of cross-platform dev on a single OS; documented, not hidden.
- **Consistency:** the `set_file_perms_macos` rename in Task 2 must update its call site in `store()`. Watch for that.

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-07-30-phase4-windows-parity-packaging.md`. Two execution options:

**1. Subagent-Driven (recommended)** — fresh subagent per task, two-stage review.

**2. Inline Execution** — batch with checkpoints.

**Which approach?**
