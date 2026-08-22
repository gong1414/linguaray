# Modernization & rebrand refactor — tracking doc

Approved 2026-08-22. Every pass lands as its own reviewable commit and
references this file. The hard rule for all passes: **user-visible behavior
stays identical** unless the pass explicitly says otherwise.

## Baseline (recorded before any pass, on this date)

| Gate | Result |
| --- | --- |
| `dart run melos run analyze` (fatal-infos, 3 packages) | pass |
| `dart run melos run test` (app + runtime + ui_flutter) | pass |
| `dart run melos run dependency_validator` | pass |
| `python3 scripts/format.py --check` (dart/rust/swift) | pass (198 Dart files, 0 changed) |
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | pass |
| `cargo test --locked --workspace` | pass (all suites green) |
| Golden image fingerprint | `daf1ff4e2549814d6206bdb0403e5dc5f8070cf3` over 29 PNGs |

Re-check the fingerprint after every pass that touches Dart UI code:

```bash
find apps/desktop/flutter/test packages/ui_flutter/test -name "*.png" -type f \
  | sort | xargs shasum | shasum
```

## Local toolchain notes (this machine)

- Flutter/Dart SDK lives at `~/.cache/linguaray/flutter-3.47.1` and is not on
  `PATH` by default; prepend its `bin` for every Dart command.
- `cargo` is a real binary while `cargo-fmt`/`cargo-clippy` resolve to rustup
  proxies that mis-forward args (`rustup: unexpected argument '--all'`).
  Prepend `~/.rustup/toolchains/stable-aarch64-apple-darwin/bin` to `PATH`.
- Melos `exec:`-style scripts shell out to a bare `melos` binary and fail when
  melos is only available via `dart run melos`. The `test`/`fix` scripts were
  rewritten in the `run:` style in P0; keep that style for new scripts.

## Passes

| Pass | Scope | Acceptance |
| --- | --- | --- |
| P0 | Baseline + CI gap + this doc | CI green; baseline table above |
| P1 | Remove uniffi demo fns `greet`/`add`/`version`, `main.dart` smoke call, demo test assertions, README demo | cargo + melos gates green |
| P2 | Full rename of every internal identifier (crates, Dart packages, uniffi namespace/symbols, Swift framework/plugin, method channels, Linux binary/deb names, CI `-p`, scripts, docs, AGENTS.md) to the `linguaray` brand — the legacy prototype name (visible in git history before this pass) must be gone | all gates + `flutter build macos --debug`; **`git grep -iE 'beyond[-_]?translate'` returns zero tracked hits**; golden fingerprint unchanged |
| P3 | Delete ~17 unreferenced Flutter files, commented blocks, unused members, `dio`, `negative_vertical_margin`; gate `widget_showcase` behind `kDebugMode`; dedupe `getAppDirectory` | analyze clean without the removed `unused_element` ignore; golden fingerprint unchanged |
| P4 | Drop unused Rust workspace deps (`bytes`, `mockito`, `wasm-bindgen`, engine `getrandom`) and 11 never-gated engine features | cargo gates green; Cargo.lock diff only shrinks |
| P5 | `{Key? key}` → `super.key`; deprecated `unfocus()` calls → current API; drop file-wide ignores | analyze --fatal-infos clean with ignores removed; fingerprint unchanged |
| P6 | Rewrite React/storybook/TanStack-era comments and README mapping tables; fix stale `kAppBuildNumber`; drop dead `kIsWeb` branches | analyze + format green; no runtime change |
| P7 | Merge duplicated `_CompareToggle`; extract shared golden harness into `ui_flutter` testing export; share sidebar drag logic; unify `fromId` mappers | golden fingerprint **byte-identical**; sidebar/metrics tests pass as-is |
| P8 | Split `runtime.rs` (3061): the inline 765-line `mod tests` moved verbatim to `runtime/tests.rs` (hub now 2294 lines). The impl-block file splits and the `system.rs`/`glossary.rs` splits are deferred: the Mimosa pre-commit hook requires every `.rs` write to go through Write/Edit (no scripted bulk moves), which makes large text moves disproportionate — follow up in a fresh session using Write-tool transcription | cargo fmt/clippy/test green; diff is a pure move; no pub API change. One first-run flake of the real-Vision OCR test did not reproduce (5 subsequent passes) |
| P9 | Split the ui example's 1018-line main.dart: the whole `_AtomsState` gallery body moves to `atoms_sections.part.dart` (main.dart now 187 lines). The workbench/mini-translator page splits are deferred: their build helpers call the `@protected` `setState`, so they cannot move into extensions/part mixins without first being redesigned as standalone widgets with explicit inputs — that is widget-level design work, not a mechanical move | flutter test green incl. example analyze; fingerprint unchanged |
| P10 | `build_from_engine_config` no longer serializes the config to YAML and reparses it: engine's `from_config` is now public and the runtime passes the structured config straight through. New tests: direct-vs-YAML-path parity, and a ProviderConfigEntry ↔ ProviderConfig round-trip preserving id/type/fields | new cargo tests + existing suites green (59 runtime tests) |

## Deferred as separate migration tasks (spec first, not in this effort)

M1 flutter_lints ^2→^6 · M2 dialog-stack consolidation · M3 text-entry-stack
consolidation · M4 slang-native i18n interpolation · M5 dormant
feature-flag surface decision (kept this round per owner) · M6 native menu
picker unification · M7 ChangeNotifier listener modernization · M8
remote.rs hand-mirrored core models automation.
