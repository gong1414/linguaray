# Refactor validation record

This record captures the acceptance evidence for the behavior-preserving
refactor passes in this directory. It is not permission to update snapshots or
goldens when a future cleanup fails.

## Local macOS evidence

Validated on 2026-08-25 with Flutter 3.47.1 / Dart 3.13.1 and the current stable
Rust toolchain:

- `python3 scripts/codegen.py` completed with the repository Flutter SDK on
  `PATH`; regenerated Dart, Swift, and C bindings produced no committed binding
  diff.
- `python3 scripts/check_uniffi_surface.py` matched every committed UniFFI
  signature and checksum.
- `python3 scripts/format.py --check` passed for Dart, Rust, and Swift.
- `dart run melos run analyze`, `dart run melos run test`, and
  `dart run melos run dependency_validator` passed for all workspace packages.
- All application, runtime, design-system, repository, route, provider,
  platform-contract, and Golden tests passed without changing a Golden file.
- `cargo fmt --all -- --check`, strict workspace Clippy, and the full Cargo
  workspace test suite passed: 6 API-core, 4 core, 74 engine, 86 runtime, and 1
  Runtime integration test.
- `flutter build macos --debug` produced `LinguaRay.app` successfully.
- The macOS desktop integration smoke passed resident launch, shortcut
  registration, permission refresh, speech lifecycle, built-in dictionary,
  Quick Translate, Settings, and every settings destination.

## Cross-platform gate

Windows native compilation cannot run on the macOS workstation. The existing
`desktop-windows` CI job remains a required parity gate: it runs all workspace
tests, the real system-OCR fixture, the desktop integration smoke, and
`flutter build windows --debug`. The same matrix performs the corresponding
macOS checks. A refactor branch is not mergeable until both runners pass.

## Deliberately separate work

Dependency upgrades, public-port changes, Riverpod lifecycle migration, a full
typed-router migration, persisted-schema changes, true UniFFI exported-module
relocation, and platform-port architecture changes remain in
`MIGRATION_BACKLOG.md`.
