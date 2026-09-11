# LinguaRay desktop redesign review — 2026-09-11

The user requested a complete interface redesign and project-local Hallmark
and Impeccable setup. Navy was explicitly rejected. The implementation uses
neutral white, gray and graphite and retains the canonical brand artwork.

## Scope

The shared Material theme and settings frame cover all production destinations.
The former icon rail and second navigation row were replaced by a grouped
directory. Quick translation, OCR, general settings, service/provider settings,
history/favorites, glossary, vocabulary, permissions, backup, integration,
updates, about, common-language/translation-target forms and dictionary/glossary
dialogs consume the same design system. Working feature controllers remain in
place. Reachability checks confirm catalog fixtures stay out of production.

## Visual review

An independent reviewer inspected nine representative macOS captures, then
confirmed the wordmark fix and three added dialog captures. Final macOS visual
disposition: ship. No further visual fixes were requested. This is a review of
the inspected captures, not an assertion of native Windows verification.

## Validation

- Workspace Flutter tests: 244 passed, including 92 macOS golden states.
- Workspace static analysis: `dart run melos run analyze` passed.
- All 14 settings destinations: 1000×700 and 780×520, light/dark,
  macOS/Windows theme variants. These Windows-themed layouts ran on macOS.
- macOS debug production bundle built successfully.
- Dependency validation, formatting, Dart architecture, Dart library
  reachability and UniFFI surface checks passed.
- No new runtime interface or localization keys were introduced.

## Remaining platform evidence

Windows native goldens still describe the previous visual system. Do not copy
macOS PNGs over them or bypass comparison. On a Windows checkout, run from
`apps/desktop/flutter`:

```powershell
flutter test test/catalog_surface_golden_test.dart --update-goldens
flutter test
flutter build windows --debug
```

Review and commit the resulting Windows snapshots before merging or releasing
this redesign. No Windows build or native interaction result is claimed here.
The Impeccable HTML/CSS detector has no verdict on native Dart layouts.

## Follow-up residual audit

Checked the three production MaterialApp roots, all 14 settings routes and
feature dialog call sites. No alternate theme or references to the former
orange/warm palette or icon-rail navigation remain in production source.
The add-service dialog still used a bare dropdown and unlabeled input; it was
replaced by ServiceEditorView with labeled fields, scrolling and production
Widgetbook coverage. A behavior test verifies provider selection, trimmed
names and the preserved empty-name fallback. History and vocabulary edit
forms now scroll within constrained windows.

After this follow-up, the desktop suite passed 213 tests (including 92 macOS
goldens), desktop static analysis and the macOS production debug build passed.
The unchanged application/runtime/UI package suites previously passed 31 tests.
The 84 existing Windows native baselines remain outstanding as described above.
