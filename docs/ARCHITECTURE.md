# Architecture

LinguaRay separates the desktop interface from translation capabilities so UI
work can be inspected and changed without rebuilding product logic.

```text
┌─────────────────────────────────────────────────────────┐
│ Flutter UI                                              │
│ routes · widgets · Widgetbook · view state              │
└──────────────────────────┬──────────────────────────────┘
                           │ typed controllers
┌──────────────────────────▼──────────────────────────────┐
│ Application and platform services                       │
│ shortcuts · windows · tray · permissions · secure store │
└──────────────────────────┬──────────────────────────────┘
                           │ UniFFI
┌──────────────────────────▼──────────────────────────────┐
│ Rust runtime                                             │
│ translation · OCR · providers · settings · persistence   │
└─────────────────────────────────────────────────────────┘
```

## Layers

### Flutter application

`apps/desktop/flutter` contains routes, controllers, desktop lifecycle code,
and platform integrations. Widgets render state and send user intent to a
controller. They do not read platform permissions or call plugins directly.

The quick translator is a dynamically sized utility window. The workbench is
the persistent surface for input translation and settings. Startup guidance is
a route inside the workbench rather than a separate native onboarding window.

### Design system

`packages/ui_flutter` contains reusable tokens, themes, and controls.
Application-specific translation widgets remain in the desktop application.
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

1. A shortcut, tray action, or widget produces a typed user action.
2. A controller refreshes required platform state and requests selection,
   input, or a captured image.
3. The controller calls the Rust runtime with plain typed data.
4. Streaming or completed runtime results are mapped to UI state.
5. Widgets render loading, success, recoverable error, or permission states.

External provider calls are replaced by local stubs in automated tests. System
OCR uses fixed, non-sensitive fixture images.

## Storage

LinguaRay uses its own `v2` configuration namespace and does not read or delete
data from the former Tauri application. Logs, UI state, and normal settings must
not contain provider secrets.
