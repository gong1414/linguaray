# Refactor public surfaces

The following surfaces are compatibility boundaries during the refactor.
Moving implementation code is allowed; renaming, removing, or changing the
meaning of these surfaces is not.

## Dart packages

- `package:linguaray_application/linguaray_application.dart`: exported models,
  ports, and use cases.
- `package:linguaray_runtime/linguaray_runtime.dart`: generated UniFFI records,
  enums, objects, functions, errors, and method signatures.
- `package:linguaray_ui/linguaray_ui.dart`: the desktop Material theme and canonical brand mark.
- Existing public screen, controller, repository, and route types used by app
  tests or workspace packages.

Compatibility barrels should re-export moved declarations while oversized
modules are split. The unused internal UI exports were removed in the visual unification change
after migrating all workspace callers. Splitting `WorkspaceSettingsRepository`
still requires a separate API migration.

## Rust and UniFFI

- Public exports from `linguaray-core`, `linguaray-engine`, and
  `linguaray_runtime` retain their names and module paths.
- The `Runtime` capability accessors and all generated Dart and Swift method
  signatures remain stable.
- UniFFI API version/checksum agreement must continue to fail fast when native
  and generated bindings drift.

The committed [UniFFI surface baseline](UNIFFI_SURFACE.txt) records the contract
version and signature checksum for every exported function, constructor, and
method. Run `python3 scripts/check_uniffi_surface.py` after reorganizing runtime
implementation code. Updating that file is reserved for a focused API
migration and must be accompanied by regenerated Dart, Swift, and C bindings.

## Persistent and native contracts

- JSON field names, enum wire values, compatibility aliases, catalog preset
  identifiers, service identifiers, and the `v2` namespace are stable.
- Backup archive paths and validation rules are stable.
- Platform channel names and method payloads are stable until both native and
  Dart consumers are migrated in the same focused change.
- Window titles, supported settings paths, protocol commands, shortcut action
  identifiers, secure-storage identifiers, and application identifiers are
  stable.

## Generated sources

Generated Dart, Swift, native headers, and localization files are outputs, not
manual edit targets. Changes to runtime interfaces or localization JSON must go
through `python3 scripts/codegen.py`.
