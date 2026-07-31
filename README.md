# LinguaRay

A cross-platform, open-source translation / OCR / TTS desktop tool — an
actively-maintained successor to [pot-desktop](https://github.com/pot-app/pot-desktop)
(which stopped updating), and the open answer to its author's later closed-source
paid version ([manggo](https://manggo.pylogmon.cn/)).

**Status:** v1 in progress. Heads-down build before open-source.

## What makes it different

The headline feature is a **cc-switch-style AI provider catalog**: the user picks
a preset provider (OpenAI / Anthropic / Gemini / local Ollama / API 中转站 …),
fills in an API key, and it works — instead of the generic "OpenAI-compatible
form where you hand-edit base_url + model" that every other translation tool
uses. Adding a provider = adding one line of config, not reverse-engineering code.

Three legs, decided in the design grilling:

1. **AI-native** — the LLM is the default translation engine (auto language detect).
   Not a chat app — just translation, done well.
2. **Privacy / local-first** — local LLM (Ollama) is first-class. No telemetry.
   Users supply their own keys. API keys are stored in a self-encrypted,
   machine-bound keystore (AES-256-GCM + Argon2id).
3. **Continuously-maintained open source** — will never go closed/paid.

### Current capabilities (v1)

- AI provider catalog (preset-based, fill-key-and-use)
- Self-encrypted keystore (machine-bound, fail-closed)
- Selection translate (Alt+Space) with hybrid AX-first + sentinel-copy-fallback capture
- Input translate (Ctrl+Space)
- User-initiated clipboard translate
- Cursor-anchored popup with latest-wins generation token
- Built-in Google traditional engine as AI-fallback (§G classified fallback)
- Compound clipboard restore (text + image, single platform-level write)
- Windows + macOS cross-platform parity (keystore atomic replace + ACL, compound clipboard)
- CSP-hardened WebView, per-window least-privilege capabilities

### Planned (v1.x, before public open-source)

- PaddleOCR screenshot/OCR translate · TTS · external invocation
- Long-text segmentation/chunking
- Additional traditional engines (DeepL / 百度 / 有道 / …)
- Dictionary lookup UI + fallback-engine selector UI
- Plugin/WASM extensibility

## Tech stack

- **Tauri 2** + **Rust** backend, **SolidJS** + TypeScript frontend
- **Platforms:** Windows + macOS (Linux out of scope for v1)
- v1 has **no plugin system** — engines are built-in. Plugin/WASM extensibility is
  deferred to post-v1.

## Develop

Requirements: **Node 24+**, **pnpm 11.18.0** (pinned in `packageManager`), a working
Rust toolchain (stable), Xcode CLT (macOS) or the MSVC toolchain (Windows).

```bash
pnpm install
pnpm tauri dev      # launch the dev window
pnpm tauri build    # production bundle
```

### Toolchain notes

- **Node 24** is required — pnpm 11.18.0 needs Node ≥ 22.13. The CI workflows use
  `actions/setup-node@v4` with `node-version: 24`.
- **pnpm version** is pinned via the `packageManager` field in `package.json`
  (`pnpm@11.18.0`). The `pnpm/action-setup` action reads this automatically — do NOT
  pass a conflicting `version:` in the workflow.
- **Rust stable** with the target triple for your platform:
  - macOS arm64: `aarch64-apple-darwin`
  - macOS Intel: `x86_64-apple-darwin`
  - Windows x64: `x86_64-pc-windows-msvc`

## Project layout

```
src/                        # SolidJS frontend (main settings, popup, input)
src-tauri/src/
  lib.rs                    # Tauri commands + hotkey wiring
  clipboard/                # OS clipboard abstraction + compound restore
    mod.rs                  #   macOS (objc2) + non-mac/non-win (arboard)
    fsm.rs                  #   platform-neutral ownership state machine (always compiled)
    windows.rs              #   Win32 adapter (cfg(windows)): build_blobs + Win32ClipOps
  selection.rs              # §B hybrid selection capture wiring
  selection_engine.rs       # §B sentinel state machine (pure, unit-testable)
  keystore.rs               # self-encrypted JSON keystore (AES-256-GCM, Argon2id, machine-bound)
  providers.rs              # AI provider catalog (the core differentiator)
  service.rs                # §G classified fallback orchestration
  wire.rs                   # provider wire contract (HTTP, error classification)
  a11y.rs                   # macOS Accessibility AX-first read
```

## CI workflows

### `windows-check.yml` — continuous integration gate

Runs on every push and pull request to `main`. Windows runner only — validates
that the Windows-specific code compiles and tests pass on the real platform
(macOS code is tested locally). Steps: `cargo check`, `cargo clippy -D warnings`,
`cargo test` (including the real-clipboard `#[ignore]` tests via `--ignored`),
frontend build + `tsc --noEmit`.

### `release.yml` — release workflow (dual mode)

**Trigger 1: `workflow_dispatch` → UNSIGNED dry-run.**

- Runs the full 3-platform matrix (macos-latest arm64, macos-15-intel,
  windows-latest x64).
- Builds **unsigned** bundles. No secrets required.
- Uploads artifacts tagged `linguaray-UNSIGNED-*`.
- Does **NOT** create a GitHub Release.
- **UNSIGNED artifacts must NOT be distributed to end users** — they lack macOS
  codesign/notarization (Gatekeeper will block them) and Windows Authenticode
  (SmartScreen will warn). Use this mode only to validate the build matrix.

**Trigger 2: `push tag v*` → SIGNED official release.**

- Same 3-platform matrix, but with signing.
- **Fail-closed**: if any signing secret is missing, the build **fails immediately**
  with a clear error — it NEVER silently produces an unsigned bundle on a tag release.
- macOS: cert import → keychain → codesign → notarization → stapling.
- Windows: PFX import → Authenticode (`certificateThumbprint` overlay).
- A GitHub Release is created **only after all 3 platforms succeed**.

## Release: signing secrets

### macOS (7 secrets)

| Secret | Purpose |
|--------|---------|
| `APPLE_CERTIFICATE` | Base64-encoded `.p12` Developer ID Application certificate |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the `.p12` file |
| `KEYCHAIN_PASSWORD` | Password for the temporary CI keychain |
| `APPLE_SIGNING_IDENTITY` | The signing identity name (e.g. `Developer ID Application: Your Name (TEAM_ID)`) — verified via `grep -F` after import |
| `APPLE_ID` | Apple ID for notarization |
| `APPLE_PASSWORD` | App-specific password for notarization |
| `APPLE_TEAM_ID` | Apple Developer Team ID |

### Windows (2 secrets)

| Secret | Purpose |
|--------|---------|
| `WINDOWS_CERTIFICATE_PFX` | Base64-encoded PFX Authenticode certificate |
| `WINDOWS_CERTIFICATE_PASSWORD` | Password for the PFX file |

### Tauri updater key (separate, not for release signing)

`TAURI_SIGNING_PRIVATE_KEY` / `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` sign the
**updater bundle** (the auto-update mechanism), NOT the installer. They are a
completely separate key pair from the Apple/Windows signing certificates above.
Do NOT conflate them.

## Release: artifact types

| Platform | Target | Artifacts |
|----------|--------|-----------|
| macOS arm64 | `aarch64-apple-darwin` | `.dmg` (disk image), `.app` (inside the DMG) |
| macOS Intel | `x86_64-apple-darwin` | `.dmg`, `.app` |
| Windows x64 | `x86_64-pc-windows-msvc` | `.msi` (installer), `.exe` (NSIS installer) |

## Release: commands and verification

### Triggering a release

```bash
# Unsigned dry-run (validates the build matrix, no Release created):
# GitHub → Actions → "release" → "Run workflow"

# Signed official release (creates a GitHub Release with signed artifacts):
git tag v0.1.0
git push origin v0.1.0
```

### Verifying signatures

**macOS** (after downloading the `.dmg`):

```bash
# Check codesign:
codesign -dv --verbose=4 /Applications/LinguaRay.app

# Check notarization (returns "accepted" for notarized apps):
spctl --assess --verbose=4 /Applications/LinguaRay.app
xcrun stapler validate /Applications/LinguaRay.app
```

**Windows** (after installing the `.msi` or running the `.exe`):

```powershell
# Check Authenticode signature:
Get-AuthenticodeSignature "C:\Program Files\LinguaRay\LinguaRay.exe"
# Status should be "Valid"
```

## Roadmap (solo, ~1hr/day, must-ship)

**v1 — translation core:**
- **Phase 0 — foundation** ✅ Tauri 2 + SolidJS scaffold, translate contract wired
- **Phase 1 — AI provider catalog + keystore + unified pipeline** ✅ (the headline feature)
- **Phase 2 — selection/input translate + user-initiated clipboard translate + cursor-anchored popup** ✅
- **Phase 3 — built-in traditional engines** (Google ✅; DeepL/百度/有道/… follow the pattern) + system dict + §G fallback chain ✅
- **Phase 4 — cross-platform parity + packaging** — Windows keystore (atomic replace + ACL), Windows compound clipboard restore, CSP hardening, per-window capabilities, release/signing CI ✅ (fallback selector UI + dictionary lookup UI pending for v1.x)

**v1.x (before public open-source release):**
- PaddleOCR screenshot/OCR translate · TTS · external invocation
- polish, then open-source.

## License

MIT.

## Third-party

- macOS selection capture uses a vendored AX-only read from
  [`get-selected-text`](https://github.com/yetone/get-selected-text) (MIT).
  The upstream's AppleScript copy-fallback is deliberately excluded.
