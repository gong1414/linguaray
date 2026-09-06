# Desktop platform parity checklist

Use this checklist for refactors that touch shared UI, controllers, runtime
interfaces, windows, permissions, shortcuts, capture, or native registration.

| Area | macOS | Windows |
| --- | --- | --- |
| Normal launch | Resident process; no product window | Resident process; no product window |
| Reopen entry point | Menu-bar icon or Dock fallback | Notification-area icon |
| Preferences | Native title frame | Custom Windows title controls |
| Quick Translate | Compact, focus-aware, display-constrained surface | Compact, focus-aware, display-constrained surface |
| OCR | Screen/file/clipboard workflows | Screen/file/clipboard workflows |
| System translation | Available when the OS service supports it | Not advertised |
| System dictionary | Available | Not advertised |
| System OCR | Available | Available |
| Accessibility permission | Refreshed and requested before protected selection actions | Not required |
| Screen recording permission | Refreshed and requested before protected capture actions | Not required |
| Launch at login | Platform login item mirrors persisted setting | Platform login item mirrors persisted setting |
| Tray fallback | Dock icon prevents loss of all entry points | Tray icon is always retained |
| Secrets | Platform secure storage | Platform secure storage |

## Required parity flows

For both targets, cover opening every settings destination, switching between
Preferences/Quick Translate/OCR, editing shortcuts, translating typed and
clipboard text, capture cancellation, OCR success and empty results, provider
configuration, history/favorites, glossary/vocabulary, backup/restore, proxy
changes, update checks, and protocol-triggered actions.

macOS-native source changes require a macOS build and smoke test. Windows-native
or shared platform changes require a Windows CI build and the corresponding
integration tests. A host-only test run is not sufficient evidence for a
cross-platform native refactor.
