<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/brand/linguaray/dist/readme/linguaray-readme-dark.svg">
    <source media="(prefers-color-scheme: light)" srcset="assets/brand/linguaray/dist/readme/linguaray-readme-light.svg">
    <img alt="LinguaRay" src="assets/brand/linguaray/dist/readme/linguaray-readme-light.svg" width="560">
  </picture>

  <p>A privacy-first translator that stays one shortcut away.</p>

  [![CI](https://github.com/gong1414/linguaray/actions/workflows/ci.yml/badge.svg)](https://github.com/gong1414/linguaray/actions/workflows/ci.yml)
  [![License: AGPL-3.0](https://img.shields.io/github/license/gong1414/linguaray)](LICENSE)
  [![Flutter](https://img.shields.io/badge/Flutter-3.47.1-02569B?logo=flutter)](https://flutter.dev/)
  [![Platforms](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-59636E)](#platform-support)

  **English** · [简体中文](README-ZH.md)
</div>

## About

LinguaRay is an open-source desktop translator for macOS and Windows. Select
text, type or paste, or capture a screen region, then translate without moving
your workflow into a browser tab.

<p align="center">
  <img src="docs/images/workbench.png" alt="LinguaRay workbench translating English into Chinese" width="860">
</p>

### Highlights

- **One shortcut away** — open the quick translator, translate selected text,
  or start a screen capture with configurable global shortcuts.
- **Text and image translation** — handle selections, clipboard input, and
  screenshot OCR through one focused interface.
- **Provider choice** — use operating-system services or configure compatible
  translation providers without coupling the interface to one vendor.
- **Private credentials** — API keys live in the operating system's secure
  storage and are never written to the regular settings file.
- **Native desktop behavior** — tray access, permission recovery, multi-display
  placement, and platform-aware window handling.
- **Inspectable UI** — important states are available in Widgetbook and covered
  by golden tests.

## Platform support

| Platform | Minimum | Status |
| --- | --- | --- |
| macOS | 13.0 | Supported |
| Windows | Windows 10 | Supported |

Linux is not part of the current release matrix.

## Download

Builds are published on the
[Releases](https://github.com/gong1414/linguaray/releases) page. Until the first
signed release is available, run LinguaRay from source using the steps below.

> Public CI artifacts are unsigned unless a maintainer supplies platform
> signing credentials.

## Build from source

### Prerequisites

- [Flutter 3.47.1](https://docs.flutter.dev/install/archive) with Dart 3.13.1
- the current stable [Rust toolchain](https://www.rust-lang.org/tools/install)
- Xcode and CocoaPods on macOS, or Visual Studio with the Desktop development
  with C++ workload on Windows

```bash
git clone https://github.com/gong1414/linguaray.git
cd linguaray
dart pub get
cd apps/desktop/flutter
flutter run -d macos        # use windows on Windows
```

Flutter hot reload is the normal way to adjust and inspect the interface. Run
the component catalog independently with:

```bash
cd apps/desktop/flutter
flutter run -d macos -t lib/widgetbook.dart
```

## Architecture

LinguaRay keeps interface code and capabilities separate:

```text
Flutter UI → controllers → platform services / UniFFI → Rust runtime
```

- `apps/desktop/flutter` contains the desktop host, routes, controllers, and
  platform integrations.
- `packages/ui_flutter` is the reusable Flutter design system.
- `packages/runtime` exposes the Rust runtime to Dart and Swift through UniFFI.
- `crates` contains translation, OCR, provider, settings, and shared core logic.

See [Architecture](docs/ARCHITECTURE.md) for boundaries and data flow, and
[Contributing](CONTRIBUTING.md) for the development and test workflow.

## Security and privacy

LinguaRay only sends content to the provider you choose for an action you
start. Provider secrets are stored through platform secure storage. Do not post
credentials, private text, or sensitive screenshots in public issues.

Report vulnerabilities using the process in [SECURITY.md](SECURITY.md).

## Contributing

Bug reports, focused feature proposals, documentation improvements, and code
contributions are welcome. Before opening a pull request, read
[CONTRIBUTING.md](CONTRIBUTING.md) and the
[Code of Conduct](CODE_OF_CONDUCT.md).

## Attribution and license

LinguaRay is derived from
[BeyondTranslate CE](https://github.com/beyondtranslate/beyondtranslate-ce).
Compatible internal package names are retained where they keep the native
bridge stable. Upstream and dependency notices are documented in
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

LinguaRay is licensed under the [GNU AGPL v3](LICENSE).
