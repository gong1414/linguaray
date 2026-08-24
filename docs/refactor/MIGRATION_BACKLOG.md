# Separate migration backlog

The items below are intentionally excluded from behavior-preserving cleanup.
Each changes a public contract, persisted representation, dependency baseline,
or architecture and must be reviewed as its own migration.

## Dart and Flutter dependency upgrade

The current resolver reports newer incompatible releases, including analyzer,
build tooling, `flutter_lints`, Melos, slang, platform interop packages, and
Win32. Upgrade coherent toolchain groups rather than changing all constraints
in a cleanup commit.

Acceptance checks: dependency diff review, generated-code diff review, all
workspace checks, macOS and Windows debug builds, integration smoke tests, and
deliberate golden review. Major lint upgrades need a separate mechanical-fix
commit before behavior changes.

The pinned Flutter/Dart toolchain, Riverpod, `go_router`, UniFFI, and the
`uniffi-dart` Git `main` dependency are all part of this migration boundary.
Replace the moving Git reference with an immutable revision as a focused
reproducibility task rather than hiding it in cleanup.

## Riverpod lifecycle migration

Changing the current `Notifier` providers and their scheduled `reload` calls
to `AsyncNotifier` changes loading, cancellation, disposal, error, and listener
timing. Specify those states first, migrate one domain at a time, and add
race/disposal tests plus window reopen tests.

## Typed router migration

The cleanup centralizes settings destinations but deliberately keeps the
current public URLs and untyped `go_router` graph. A full typed-router move
changes generated APIs, redirect semantics, deep-link parsing, and test setup;
perform it in a separate migration with a route compatibility matrix.

## Persisted appearance schema cleanup

The Rust settings schema still accepts the historical `appearance.theme`
field while active UI behavior uses `themeMode`. Removing or renaming the field
requires backup/restore fixtures for old settings, an explicit schema migration,
and round-trip tests across Dart and Swift consumers.

## Public Dart API cleanup

Removing generic legacy exports from `package:linguaray_ui/linguaray_ui.dart`
or splitting `WorkspaceSettingsRepository` changes public package contracts.
First inventory downstream imports, add narrower replacement barrels or ports,
publish a deprecation window, and update package-level API tests.

## Multi-window architecture

LinguaRay deliberately uses one stable Flutter host for mutually exclusive
surfaces. Moving to independent engine-backed windows changes lifecycle,
focus, sizing, tray behavior, resource ownership, and integration testing. Do
this only when the chosen Flutter API is supported on both release targets and
after a written window-state specification and rollback plan exist.

## UniFFI exported-module relocation

UniFFI includes the Rust module location of exported `impl` blocks in method
signature checksums. The behavior-preserving split uses lexical `include!`
fragments, so exports still belong to `runtime`. Moving those implementations
into true Rust child modules changes every affected binding checksum even when
names and parameter types are identical. Treat that relocation as an API
migration: regenerate every binding, review the full checksum baseline, update
all bundled native libraries atomically, and run upgrade tests against an
installed build using the previous bindings.

## Native channel replacement

Speech, protocol, system proxy, and macOS presentation channels remain
compatibility contracts. Replacing them with FFI or a different plugin layer
requires simultaneous Dart/macOS/Windows changes, payload versioning, and
target-specific integration coverage.

Move `nativeapi`, `file_selector`, tray behavior, window events, and file
dialogs behind typed application ports as separate capability migrations.
Window, tray, and file-dialog migrations each need their own desktop
integration coverage; do not combine them into one architecture change.
