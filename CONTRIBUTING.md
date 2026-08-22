# Contributing to LinguaRay

Thank you for helping improve LinguaRay. Small, focused contributions are the
easiest to review and maintain.

## Before you start

- Use [GitHub Discussions](https://github.com/gong1414/linguaray/discussions)
  for usage questions and early design conversations.
- Search existing issues before reporting a bug or proposing a feature.
- Open an issue before a large architectural or platform change so the scope
  can be agreed before implementation begins.
- Never include API keys, private text, or sensitive screenshots in an issue,
  test fixture, log, or pull request.

By participating, you agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Development setup

LinguaRay uses Flutter 3.47.1, Dart 3.13.1, and the current stable Rust
toolchain. macOS development also requires Xcode and CocoaPods; Windows
development requires Visual Studio with the Desktop development with C++
workload.

```bash
git clone https://github.com/gong1414/linguaray.git
cd linguaray
dart pub get
cd apps/desktop/flutter
flutter run -d macos        # use windows on Windows
```

Use the component catalog for isolated interface work:

```bash
cd apps/desktop/flutter
flutter run -d macos -t lib/widgetbook.dart
```

## Repository boundaries

- Widgets depend on controllers, not directly on platform plugins.
- Platform integrations return typed results and structured errors.
- Translation, OCR, provider configuration, and persisted settings belong in
  the Rust runtime.
- API secrets must remain in platform secure storage; regular settings contain
  only secret identifiers.
- Generated UniFFI and localization files are committed, but their source files
  remain the source of truth.

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for more detail.

## Code generation

After changing Rust interfaces or localization sources, regenerate bindings:

```bash
python3 scripts/codegen.py
```

Do not manually edit files under generated binding directories. For brand
assets, follow [the brand asset guide](assets/brand/linguaray/README.md) and do
not hand-edit individual generated PNG or icon files.

## Tests and checks

Run the checks relevant to your change before opening a pull request. For a
cross-layer change, run the full set:

```bash
dart run melos run analyze
dart run melos run test
dart run melos run dependency_validator
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

Interface changes should include or deliberately update Widgetbook states and
golden tests. Platform changes should include an integration test where
automation is practical and a short manual smoke-test note otherwise.

## Pull requests

1. Create a branch from `main`.
2. Keep commits focused and write commit messages in English.
3. Update tests and user-facing documentation with the implementation.
4. Complete the pull request checklist and describe manual platform testing.
5. Address review feedback without force-pushing away discussion context unless
   a maintainer asks for a clean rebase.

By contributing code, you agree that your contribution is licensed under the
project's [GNU AGPL v3](LICENSE).
