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

## Native Windows evidence

[Windows CI capture](https://github.com/gong1414/linguaray/actions/runs/34597770235) succeeded
on the redesign commit. It generated 92 native Windows snapshots: 84 replacements
and eight new dialog states. File names match the complete macOS state set.
Representative settings, quick translation, OCR and service-dialog captures were
visually inspected after download. macOS images were not copied to Windows paths.
An independent finish reviewer inspected nine native Windows captures and returned
a ship verdict, with no clipping, overlap, missing glyphs, wordmark overflow or
inconsistent palette found. This static review does not establish native keyboard,
window or accessibility behavior.
Full Windows workspace tests, desktop integration and debug build are separate
required pull-request checks. The Impeccable HTML/CSS detector has no verdict on
native Dart layouts.

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
All 92 Windows baselines were subsequently captured as described above.
