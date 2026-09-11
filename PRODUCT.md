# LinguaRay

<!-- impeccable:product-schema 1 -->

## Platform

Native Flutter desktop on macOS and Windows, backed by Rust and UniFFI.
This is not a web or mobile application.

## Product purpose

Translate selected or pasted text and recognize text from screen captures.
The app lives in the menu bar/system tray. Shortcuts open a dynamically sized
quick translator or OCR utility; Preferences is its persistent window.

## Capabilities and constraints

Settings manage translation and OCR services, providers, shortcuts, history,
favorites, glossaries, vocabulary, permissions, backups, integration and updates.
Keep working controllers, localization and native affordances. Refresh protected
permissions before actions. Provider secrets belong in platform secure storage.
Do not introduce a dashboard, home window, onboarding window or duplicate title bar.

## Brand commitments

The spelling is LinguaRay. Use the canonical assets in assets/brand/linguaray.
Use LinguaRayMaterialTheme and SettingsPage throughout the product.

## Evidence

AGENTS.md, docs/ARCHITECTURE.md, the existing feature implementations and
Widgetbook states establish the product scope. The user requested a complete
interface redesign with no old interface left in production, and project-local
Hallmark and Impeccable configuration. Audience demographics remain unspecified.
