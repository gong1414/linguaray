# Architecture

LinguaRay separates the desktop interface from translation capabilities so UI
work can be inspected and changed without rebuilding product logic.

```text
┌───────────────────────────────────────────────────────────┐
│ Flutter views                                             │
│ Material 3 · Widgetbook · immutable props · user intents  │
└───────────────────────────┬───────────────────────────────┘
                            │ Riverpod view models
┌───────────────────────────▼───────────────────────────────┐
│ Pure Dart application layer                               │
│ use cases · entities · repository and platform ports      │
└───────────────────────────┬───────────────────────────────┘
                            │ adapter implementations
┌───────────────────────────▼───────────────────────────────┐
│ Rust runtime / desktop platform                           │
│ translation · OCR · settings / windows · tray · shortcuts │
└───────────────────────────────────────────────────────────┘
```

## Layers

### Flutter application

`apps/desktop/flutter` contains routes, Riverpod view models, adapters, desktop
lifecycle code, and platform integrations. Widgets render immutable state and
send user intent to a view model. They do not import generated runtime models,
read platform permissions, or call plugins directly.

View models depend on use cases from `packages/application`, which is a pure
Dart package with no Flutter, FFI, networking, or platform-plugin dependency.
Its ports are implemented by adapters in the desktop app. This is the enforced
dependency direction:

```text
view → view model → use case → port ← runtime/platform adapter
```

LinguaRay launches with no visible window. The native tray menu and global
shortcuts are the primary entry points. The quick translator is a dynamically
sized utility surface that is mounted only for translation actions; Preferences
is the only persistent window. There is no main workbench, chat home, or
separate onboarding window.

Settings expose translation/OCR shortcuts and services, history, favourites,
glossaries, vocabulary, permissions, update checks, and local API integration.
The quick translator consumes those capabilities through controllers and
application ports; it does not own provider or persistence logic.

### Design system

`packages/ui_flutter` projects LinguaRay's brand onto Flutter's official
Material 3 components. Material 3 is the sole foundation for new screens;
product-specific translation widgets remain in the desktop application.
Widgetbook exposes product states independently of runtime and provider access.

### Platform services

Platform services normalize global shortcuts, selection extraction, screenshot
capture, windows, tray behavior, permissions, and secure storage. Permission
state is re-read when the application becomes active and immediately before a
protected action; it is not treated as immutable startup state.

### Rust runtime

The crates under `crates` implement shared models and translation engines.
`packages/runtime` exposes the runtime to Dart and Swift through generated
UniFFI bindings. Settings contain secret identifiers, while secret values stay
in platform secure storage.

macOS uses both Dart and Swift bindings where native UI or system services are
required. Runtime instances are deduplicated by data directory so the two
binding paths observe the same state.

## Data flow

1. A shortcut, tray action, or settings/quick-translation view produces a typed
   user intent.
2. A Riverpod view model invokes an application use case.
3. The use case coordinates one or more abstract ports using pure models.
4. A desktop adapter maps those requests to the Rust runtime or a platform
   service and maps generated types back at the boundary.
5. The view model publishes immutable loading, streaming, success, or
   recoverable-error state for Material widgets to render.

Application tests replace ports with in-memory fakes. Widget and golden tests
render views without initializing Rust or loading secrets. Integration tests
exercise the adapters and native window behavior separately.

External provider calls are replaced by local stubs in automated tests. System
OCR uses fixed, non-sensitive fixture images.

## Storage

LinguaRay uses its own `v2` configuration namespace and does not read or delete
data from the former Tauri application. Logs, UI state, and normal settings must
not contain provider secrets.

Settings, history, vocabulary, glossary files and backup exports use the same
`packages/runtime/rust/src/storage.rs` commit primitive. It writes to a unique
file beside the destination, flushes it, and replaces the destination without
first deleting the previous file. Failed staging and replacement leave the
previous committed file intact. A terminated writer may leave a `.tmp` file;
loaders and backup export ignore it. JSON schemas and data paths are unchanged.
This is a single-file guarantee, not a transaction across all runtime stores.
Backup restore retains its separate staged install and rollback protocol.
Directory synchronization after commit is best effort on Unix; power-loss
durability depends on the operating system, filesystem and hardware.

Versioned backup/restore and the shared proxy policy are documented in
[Data transfer and network policy](DATA_AND_NETWORK.md). Backups deliberately
exclude secure-storage values.

## Desktop visual contract

`packages/ui_flutter` exposes the canonical logo and `LinguaRayMaterialTheme`.
Every running surface and Widgetbook uses that theme directly; no alternate
widget theme or token provider is mounted. Settings pages share `SettingsPage`
for content backgrounds, heading baselines, action placement, and page insets.
Native window dimensions are content sizes; Windows frame extents are included
when constraining minimum sizes and fitting the window to a display work area.

`python3 scripts/check_dart_reachability.py` rejects libraries not reachable from
`main.dart` or the explicit Widgetbook/testing entry points. It also rejects
catalog imports in the production graph. This library-level check complements
the Dart analyzer and Rust Clippy; it does not infer whether every public member
is used by external consumers. The visual suite renders all settings destinations
and translation/OCR/provider/update states using production theme font names.
