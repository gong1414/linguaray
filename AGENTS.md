# AGENTS.md — LinguaRay

This file is the working reference for automated agents and contributors. The
repository contains a Flutter desktop UI and a Rust runtime connected through
UniFFI. The supported release targets are macOS and Windows.

## Repository map

```text
apps/desktop/flutter/   Flutter desktop host, controllers, and platform code
crates/                 Rust core models and translation engines
packages/runtime/       Dart/Rust/Swift UniFFI bridge
packages/ui_flutter/    Reusable Flutter design system
assets/brand/           Canonical LinguaRay brand sources and generated assets
scripts/                Formatting and code-generation entry points
docs/                   Maintainer-facing architecture documentation
```

Internal package and FFI identifiers are implementation details, and they all
use `linguaray`-prefixed names (`linguaray_runtime`, `linguaray_ui`,
`linguaray-core`, …). User-facing names, identifiers, documentation, and
assets use `LinguaRay`. Do not reintroduce the retired prototype name in any
identifier, file name, channel string, or doc.

The workspace root also holds untracked leftovers from a retired React/Tauri
prototype (`src-tauri/`, `storybook-static/`, `node_modules/`, `dist/`,
`test-results/`, `s1b-shots/`, `.pnpm-store/`), kept out via `.gitignore` and
the local `.git/info/exclude`. They are not part of the product: skip them when
searching, never edit them, and do not treat them as a reference for current
behavior.

## Toolchain

- Flutter 3.47.1 / Dart 3.13.1
- current stable Rust with `rustfmt` and `clippy`
- Python 3 for generation scripts
- Xcode, Swift Format, and CocoaPods for macOS
- Visual Studio Desktop development with C++ for Windows

The root is a Dart Pub Workspace managed with Melos. The script catalog
(`analyze`, `test`, `codegen`, `format`, `format-check`, `fix`,
`dependency_validator`) lives under the `melos:` key in the root
`pubspec.yaml`, not in a `melos.yaml`. Resolve dependencies from the
repository root:

```bash
dart pub get
```

Do not add `path:` overrides between workspace packages. Use normal version
constraints so Pub resolves local workspace members.

## Architecture rules

- Widgets depend on controllers, not directly on platform plugins.
- Platform services expose typed results for shortcuts, selection, capture,
  permission state, windows, tray behavior, and secure storage.
- Translation, OCR, provider configuration, and persisted settings belong in
  the Rust runtime.
- Provider secrets stay in platform secure storage. Normal settings, logs,
  errors, and UI state must never contain secret values.
- Permission state is refreshed when the app becomes active and immediately
  before protected actions; do not cache startup permission results forever.
- Keep the quick translator dynamically sized and constrained to the active
  display's work area. Do not add a second onboarding window or duplicate title
  frame.

See `docs/ARCHITECTURE.md` for the full data flow.

## Development

```bash
cd apps/desktop/flutter
flutter run -d macos          # use windows on Windows
flutter run -d macos -t lib/widgetbook.dart
```

Use Flutter hot reload for normal interface work. Packaged installs are only
needed for final release smoke testing.

## Generated code

The JSON files under `apps/desktop/flutter/lib/src/i18n/` are the source of
truth for localization. Generated Dart and Swift bindings must not be edited by
hand.

After changing runtime interfaces or localization sources, run:

```bash
python3 scripts/codegen.py
```

This regenerates Dart and Swift UniFFI bindings, native headers, and macOS
localization output, then formats the generated sources.

## Brand assets

`assets/brand/linguaray/` is the canonical brand package. Follow its README and
generation tools. Do not manually redraw or independently alter generated
platform icons. The canonical spelling is always `LinguaRay` with capital `L`
and `R` and no space.

The Flutter `BrandLogo` painter mirrors the exact paths and colors of the
canonical SVG. If symbol geometry changes, update it in the same brand change
and refresh affected golden tests.

## Required checks

Run checks in proportion to the change. Before a cross-layer pull request, run:

```bash
dart run melos run analyze
dart run melos run test
dart run melos run dependency_validator
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
python3 scripts/check_uniffi_surface.py
python3 scripts/check_dart_reachability.py
```

Use `python3 scripts/format.py --check` to check Dart, Rust, and Swift formatting
together. UI changes need Widgetbook coverage and deliberate golden updates.
Platform changes need desktop integration coverage where practical.
Use `LinguaRayMaterialTheme` for all surfaces and `SettingsPage` for settings
layouts. Catalog fixtures must stay reachable only from the Widgetbook entry,
and tests must not be the only callers keeping a product library alive.

## Git practices

- Write clear, focused commit messages in English.
- Preserve third-party attribution and update notices when importing code.
- Do not commit credentials, private fixtures, build products, or local IDE
  state.
- Do not rewrite shared branch history unless the repository owner explicitly
  requests it.
