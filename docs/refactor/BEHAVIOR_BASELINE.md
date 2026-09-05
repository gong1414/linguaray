# Refactor behavior baseline

This document defines the product behavior that cleanup-only refactors must
preserve. A change to any item below is a product change and belongs in a
separate migration or feature pull request.

## Process and surfaces

- LinguaRay starts as a resident desktop application without opening a product
  window during a normal launch.
- The tray menu and global shortcuts are the primary entry points.
- Exactly one Flutter surface is mounted at a time: Preferences, Quick
  Translate, or OCR.
- Preferences is the only persistent window. Quick Translate remains a compact,
  dynamically sized utility surface constrained to the active display work
  area.
- On macOS, hiding the menu-bar icon exposes the Dock icon as the fallback entry
  point. On Windows, the notification-area icon remains visible.

## Translation and OCR

- Translation supports explicit source/target selection, automatic source
  detection, configured target rules, multiple enabled services, streaming LLM
  results, retry, copy, favorite, speech, dictionary lookup, glossary feedback,
  and vocabulary saving.
- OCR supports screen capture, silent capture, files, clipboard images, and a
  dedicated OCR surface.
- Permission state is refreshed when the app becomes active and immediately
  before protected operations.
- Cancelled captures, denied permissions, empty input, unavailable clipboard
  content, provider failures, and empty results remain recoverable UI states.

## Settings and integrations

- Settings routes retain their current paths under `/settings/`.
- Provider secrets remain in platform secure storage. Persisted settings,
  backups, logs, errors, and UI state never contain secret values.
- The provider catalog remains sourced from Rust and filtered by platform
  capability.
- Launch-at-login changes are reflected in both the platform login item and
  persisted settings, with rollback when the platform rejects the change.
- The local API server, proxy policy, protocol links, update checks, backup, and
  restore retain the behavior documented in `DATA_AND_NETWORK.md` and
  `INTEGRATIONS.md`.

## Persistence

- Runtime data remains in LinguaRay's `v2` namespace.
- The application does not read, migrate, or delete data from the retired
  prototype.
- Settings, history, glossary, and vocabulary data keep their current file
  formats and compatibility behavior.
- Backup and restore keep their current inclusion, validation, rollback, and
  secret-exclusion rules.

## Refactor acceptance

A cleanup-only pull request must satisfy all checks relevant to its layer. UI
refactors must not update golden files unless a separately approved visual
change is included. Runtime-interface refactors must regenerate bindings and
show no public signature change.
