# LinguaRay

A cross-platform, open-source translation / OCR / TTS desktop tool — an
actively-maintained successor to [pot-desktop](https://github.com/pot-app/pot-desktop)
(which stopped updating), and the open answer to its author's later closed-source
paid version ([manggo](https://manggo.pylogmon.cn/)).

**Status:** v1 feature-complete. Internal testing before public open-source release.

## Install / 安装

Once the repository goes public, installers live on the Releases page
(`.dmg` on macOS, `.msi` on Windows).

LinguaRay 的首个公开版本将以**未签名**形式分发 — 这是开源桌面软件零成本分发的
常见做法（pot-desktop、Upscayl、ChatBox 等均如此）。只有**首次打开**会出现系统
警告，处理方式如下：

- **macOS**「无法打开"LinguaRay"，因为无法验证开发者」/
  *"LinguaRay" can't be opened because it is from an unidentified developer*:
  系统设置 → 隐私与安全性 → 点「**仍要打开**」；或对 App 右键 →「打开」→ 再点「打开」。
  System Settings → Privacy & Security → **Open Anyway**; or right-click the app → Open → Open.
- **macOS**「"LinguaRay"已损坏，无法打开」/
  *"LinguaRay" is damaged and can't be opened*:
  打开终端执行（Open Terminal.app and run）:
  ```bash
  sudo xattr -d com.apple.quarantine /Applications/LinguaRay.app
  ```
- **Windows** SmartScreen「Windows 已保护你的电脑」/
  *Windows protected your PC*:
  点「**更多信息**」→「**仍要运行**」。Click **More info** → **Run anyway**.

更新包完整性将由独立的 minisign 签名保证（见 Roadmap），与安装器是否签名无关。
Signed installers (Gatekeeper/SmartScreen with zero warnings) are an optional
later step once certificates are acquired — see Roadmap.

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

## Current capabilities

### Translation
- AI provider catalog (21 official presets; only `ready` rows are
  fill-key-and-use — Azure / Custom / Doubao require extra setup)
- Selection translate (Alt+Space) with hybrid AX-first + sentinel-copy-fallback capture
- Input translate (Ctrl+Space) with draft autosave/restore
- User-initiated clipboard translate
- Cursor-anchored popup with latest-wins generation token (multi-monitor Retina-safe)
- Built-in Google traditional engine as AI-fallback (§G classified fallback)
- Configurable custom shortcuts (conflict detection, one-click reset)

### Privacy & Security
- Self-encrypted keystore (machine-bound, AES-256-GCM + Argon2id, fail-closed)
- CSP-hardened WebView, per-window least-privilege capabilities
- Compound clipboard restore (text + image, single platform-level write)

### Knowledge
- History (AES-256-GCM encrypted, NFKC casefold search, cursor pagination,
  favorites, streaming export to CSV/JSON)
- Vocabulary (encrypted CRUD, CSV/JSON/AnkiConnect export)
- Dictionary (StarDict offline packages + macOS system dictionary, atomic install)

### System
- System tray (Switch Provider submenu, Active-pulse + Error red-dot states)
- OCR (macOS Vision / Windows.Media.Ocr, region capture)
- TTS (macOS NSSpeechSynthesizer / Windows SpeechSynthesizer)
- External invocation API
- Update checker
- Onboarding flow
- Windows + macOS cross-platform parity

## Planned (v1.x)

- Long-text segmentation/chunking
- Additional traditional engines (DeepL / 百度 / 有道 / …)
- Dictionary select-word → definition popup (backend ready, needs product UI entry point)
- MDX dictionary format support
- Plugin/WASM extensibility

## Tech stack

- **Tauri 2** + **Rust** backend, **React 19** + **Ant Design X / Ant Design 6** + TypeScript frontend
- **Platforms:** Windows + macOS (Linux out of scope for v1)
- v1 official capabilities and protocol drivers are **in-tree plugins**.
  Third-party / WASM loading remains post-v1.

## Develop

Requirements: **Node 24+**, **pnpm 11.18.0** (pinned in `packageManager`), a working
Rust toolchain (stable), Xcode CLT (macOS) or the MSVC toolchain (Windows).
macOS production-style local bundles also require an **Apple Development**
code-signing identity so Accessibility and Screen Recording grants remain
valid across rebuilds. LinguaRay supports macOS 10.15 and later.

```bash
pnpm install
pnpm dev:app        # launch the desktop app in development mode
pnpm build:local    # local production bundle (updater disabled)
```

### Toolchain notes

- **Node 24** is required — pnpm 11.18.0 needs Node ≥ 22.13. The CI workflows use
  `actions/setup-node@v4` with `node-version: 24`.
- **pnpm version** is pinned via the `packageManager` field in `package.json`
  (`pnpm@11.18.0`). CI pins the same version explicitly; update both together.
- **Rust stable** with the target triple for your platform:
  - macOS arm64: `aarch64-apple-darwin`
  - macOS Intel: `x86_64-apple-darwin`
  - Windows x64: `x86_64-pc-windows-msvc`

### Running tests

```bash
# Rust (backend)
cargo test --manifest-path src-tauri/Cargo.toml --features xproc-test-helper

# Frontend gates
pnpm test
pnpm typecheck
pnpm lint
pnpm build

# Storybook + browser checks
pnpm build-storybook
pnpm test:visual

# Rust lint
cargo clippy --manifest-path src-tauri/Cargo.toml --features xproc-test-helper --all-targets -- -D warnings
```

`pnpm test-storybook` runs against a served Storybook build; the dedicated
`storybook-tests.yml` workflow builds it, starts `http-server`, installs Chromium,
and runs the test runner in CI.

## CI workflows

### `windows-check.yml` — continuous integration gate

Runs on every push and pull request to `main`. Windows runner only — validates
that the Windows-specific code compiles and tests pass on the real platform
(macOS code is tested locally). Steps: `cargo clippy -D warnings`,
`cargo test` (integration tests + the real-clipboard `#[ignore]` tests via
`--ignored`), frontend build + `tsc --noEmit`.

### `release.yml` — release workflow

**Trigger 1: `workflow_dispatch` → UNSIGNED dry-run.**

- Runs the full 3-platform matrix (macos-latest arm64, macos-15-intel,
  windows-latest x64), including the minisign updater artifacts.
- Requires the `TAURI_SIGNING_PRIVATE_KEY` secrets (see below).
- Uploads artifacts tagged `linguaray-UNSIGNED-*`. Does **NOT** create a
  GitHub Release.
- Dry-run artifacts are for build-matrix validation only — they are not
  release assets. The official distribution channel is the tag path below.

**Trigger 2: `push tag v*` + repository variable `OS_SIGNING=true` → OFFICIAL SIGNED release.**

- macOS: cert import → keychain → codesign → notarization → stapling.
- Windows: PFX import → Authenticode (`certificateThumbprint` overlay) +
  signature verification.
- Fail-closed: missing ANY of the 9 OS-signing secrets fails the build —
  a signing-enabled repo never silently produces unsigned installers.
- Tag releases without stable OS signing are rejected on macOS. TCC binds
  Accessibility and Screen Recording grants to the signing identity; shipping
  an ad-hoc update would make an enabled grant appear to disappear.

## Release: signing secrets

### Tauri updater key (REQUIRED — all modes)

| Secret | Purpose |
|--------|---------|
| `TAURI_SIGNING_PRIVATE_KEY` | Minisign private key (`pnpm tauri signer generate`) — signs the auto-update payloads |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password for that key (empty if generated without one) |

These sign the **updater bundle**, NOT the installer. The matching public key
is compiled into the app (`plugins.updater.pubkey` in `tauri.conf.json`).
Losing the private key means no more updates for released clients — back it
up. Do NOT conflate with the OS-signing certificates below.

### OS signing (REQUIRED for official macOS releases)

Enables Gatekeeper/SmartScreen-free installs. macOS (7 secrets):
`APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `KEYCHAIN_PASSWORD`,
`APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`.
Windows (2): `WINDOWS_CERTIFICATE_PFX`, `WINDOWS_CERTIFICATE_PASSWORD`.
All 9 are required together in signed mode (fail-closed). See the tables
further down for their exact meanings.

### macOS OS-signing secrets (reference)

| Secret | Purpose |
|--------|---------|
| `APPLE_CERTIFICATE` | Base64-encoded `.p12` Developer ID Application certificate |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the `.p12` file |
| `KEYCHAIN_PASSWORD` | Password for the temporary CI keychain |
| `APPLE_SIGNING_IDENTITY` | The signing identity name (e.g. `Developer ID Application: Your Name (TEAM_ID)`) — verified via `grep -F` after import |
| `APPLE_ID` | Apple ID for notarization |
| `APPLE_PASSWORD` | App-specific password for notarization |
| `APPLE_TEAM_ID` | Apple Developer Team ID |

### Windows OS-signing secrets (reference)

| Secret | Purpose |
|--------|---------|
| `WINDOWS_CERTIFICATE_PFX` | Base64-encoded PFX Authenticode certificate |
| `WINDOWS_CERTIFICATE_PASSWORD` | Password for the PFX |

## Release: artifact types

| Platform | Target | Download assets | Updater payload (in `latest.json`) |
|----------|--------|-----------------|------------------------------------|
| macOS arm64 | `aarch64-apple-darwin` | `.dmg` | `*_aarch64.app.tar.gz` + `.sig` |
| macOS Intel | `x86_64-apple-darwin` | `.dmg` | `*_x64.app.tar.gz` + `.sig` |
| Windows x64 | `x86_64-pc-windows-msvc` | `.msi`, `*_x64-setup.exe` (NSIS per-user) | `*_x64-setup.exe` + `.sig` |

## Release: runbook

```bash
# 1. Bump version in src-tauri/tauri.conf.json, commit to main.
# 2. Tag + push (tag MUST equal the config version):
git tag v0.1.0
git push origin v0.1.0
# 3. CI publishes a PRERELEASE with installers + updater payloads + latest.json.
# 4. Verify the artifacts (download .dmg/.msi, launch, check Settings → Updater
#    reports "up to date"), then promote the release to stable in the GitHub UI.
#    Promotion is what serves latest.json to auto-updating clients.
```

Unsigned dry-run (matrix validation, no Release created):
GitHub → Actions → "release" → "Run workflow".

### Verifying signatures (signed mode only)

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

## Roadmap

**v1 — core:**
- ✅ Foundation (Tauri 2 + React scaffold, translate contract)
- ✅ AI provider catalog + keystore + unified pipeline (the headline feature)
- ✅ Selection/input/clipboard translate + cursor-anchored popup
- ✅ Built-in Google engine + §G classified fallback chain
- ✅ Cross-platform parity (Windows keystore, compound clipboard, CSP, capabilities)
- ✅ Shortcuts settings (conflict detection, reset defaults)
- ✅ History (encrypted, searchable, exportable)
- ✅ Vocabulary (encrypted CRUD, AnkiConnect export)
- ✅ Dictionary (StarDict + macOS system)
- ✅ OCR (region capture + image paste)
- ✅ TTS (list/speak/stop)
- ✅ External invocation API
- ✅ Update checker
- ✅ Onboarding
- ✅ Release CI (dual-mode: unsigned dry-run / signed tag)

**v1.x (before public open-source):**
- In-app auto-update (minisign-signed updater artifacts + `latest.json`)
- Long-text segmentation
- More traditional engines (DeepL / 百度 / 有道)
- Dictionary select-word popup
- MDX format support
- Signed installers (optional — eliminates first-launch warnings; requires
  Apple Developer Program / Authenticode certificates)
- Polish + open-source

## License

MIT.

## Third-party

- macOS selection capture uses a vendored AX-only read from
  [`get-selected-text`](https://github.com/yetone/get-selected-text) (MIT).
  The upstream's AppleScript copy-fallback is deliberately excluded.
