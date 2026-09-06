# linguaray_runtime

A Flutter FFI plugin that exposes a small Rust crate to Dart and Swift via
[`uniffi-rs`](https://mozilla.github.io/uniffi-rs/) and
[`uniffi-dart`](https://github.com/Uniffi-Dart/uniffi-dart).

## Layout

```
packages/runtime/
├── rust/                         Cargo crate (workspace member)
│   ├── Cargo.toml
│   ├── build.rs                  Generates the uniffi scaffolding
│   ├── uniffi.toml               Configures the Dart binding (package_name, cdylib_name)
│   └── src/
│       ├── api.udl               UniFFI namespace declaration
│       ├── lib.rs                #[uniffi::export] surface (runtime objects + echo round-trips)
│       └── bin/
│           ├── uniffi-bindgen.rs       Wraps uniffi_bindgen_main() (Swift / Kotlin / ...)
│           └── uniffi-bindgen-dart.rs  Calls uniffi_dart::gen::generate_dart_bindings(...)
├── lib/
│   ├── linguaray_runtime.dart          Public re-export
│   └── src/generated/            Committed Dart binding (uniffi-dart)
├── swift/Generated/              Committed Swift binding (uniffi-rs)
├── macos/linguaray_runtime/            Swift Package consumed by Flutter's macOS SPM
│   ├── Package.swift
│   └── Sources/
│       ├── linguaray_runtime/          LinguarayRuntimePlugin.swift + Generated/
│       └── linguaray_runtimeFFI/       C header + module.modulemap (mirror of swift/)
├── hook/build.dart               Native-assets build hook
├── test/linguaray_runtime_test.dart    Smoke test
└── example/                      Minimal Flutter app
```

## Exposed API

The Rust source ([`rust/src/lib.rs`](rust/src/lib.rs)) and
[`rust/src/runtime.rs`](rust/src/runtime.rs) export:

- `Runtime` and its handle objects (settings, translation, dictionary, LLM,
  OCR, glossary, history, permission, text extractor, API server) — the real
  product surface.
- One `echo_*` function per `linguaray-core` model type. These exist
  only to force UniFFI metadata for the remote (hand-mirrored) core types;
  they are exercised by the round-trip tests in `rust/test/`.

## Regenerating the bindings

```bash
python3 scripts/generate/runtime_bindings.py
```

This runs `cargo build --release`, then both bindgen binaries against the
host `liblinguaray_runtime.<dylib|so|dll>`, dropping the result into:

- `lib/src/generated/linguaray_runtime.dart`
- `swift/Generated/{linguaray_runtime.swift, linguaray_runtimeFFI.h, linguaray_runtimeFFI.modulemap}`

Both directories are committed so consumers don't need a Rust toolchain to
read the API surface.

## Loading the native library

`hook/build.dart` runs `cargo build --release --target <triple>` for the
target Flutter is building for, then registers a `CodeAsset(package:
"linguaray_runtime", name: "uniffi:linguaray_runtime")` so Dart's
`@Native(assetId: "package:linguaray_runtime/uniffi:linguaray_runtime")` annotations
resolve at runtime.

## Calling the Swift binding from native macOS code

`hook/build.dart` only registers the cdylib for **Dart's** native_assets
system. To also call the UniFFI Swift binding from native code (e.g. from
an `AppDelegate.swift` or a share extension), the plugin ships a Swift
Package at [`macos/linguaray_runtime/`](macos/linguaray_runtime/) that Flutter's macOS
SPM tooling auto-discovers via `pluginClass: LinguarayRuntimePlugin` in
[`pubspec.yaml`](pubspec.yaml).

In the host app you only need to import the module - **no Xcode project
edits, no bridging header, no Ruby script**:

```swift
import Cocoa
import FlutterMacOS
import linguaray_runtime

class AppDelegate: FlutterAppDelegate {
  override func applicationDidFinishLaunching(_ notification: Notification) {
    super.applicationDidFinishLaunching(notification)
    RegisterGeneratedPlugins(registry: self)

    // The plugin has dlopen'd the bundled runtime; the Swift binding is
    // ready to use after RegisterGeneratedPlugins.
  }
}
```

### How the SPM package works

- `Package.swift` declares two targets:
  - `linguaray_runtimeFFI` is the C umbrella (header + modulemap) for the uniffi
    C ABI, mirrored from `swift/Generated/` by `scripts/generate/runtime_bindings.py`.
  - `linguaray_runtime` re-exports the generated Swift binding plus a tiny
    `FlutterPlugin` stub. It depends on Flutter's `FlutterFramework`
    package (auto-injected by the macOS toolchain) and uses
    `-Xlinker -undefined -Xlinker dynamic_lookup` so unresolved symbols are
    looked up at runtime.
- `LinguarayRuntimePlugin.register(with:)` runs during plugin
  auto-registration and `dlopen`s
  `Frameworks/linguaray_runtime.framework/linguaray_runtime`.
  Call the Swift binding after `RegisterGeneratedPlugins(...)` so the bundled
  runtime has already been loaded.

## Caveats

1. **Native assets are experimental.** Enable them once per machine:
   ```bash
   flutter config --enable-native-assets
   ```
2. **Desktop Rust targets** must be installed for the platform and
   architecture you build. For example:
   ```bash
   rustup target add x86_64-apple-darwin aarch64-apple-darwin
   rustup target add x86_64-pc-windows-msvc aarch64-pc-windows-msvc
   ```
3. **The Swift binding** at [`swift/Generated/`](swift/Generated/) is a
   standalone artifact. macOS host code that wants to call into Rust
   directly (outside of Flutter, e.g. from a share extension) can compile
   those `.swift` and `.h` files alongside the same `liblinguaray_runtime.dylib`
   that native_assets bundles.
4. **uniffi-dart 0.2.x limitations.** `HashMap`, `BigInt`, trait methods
   and proc-macro-only crates are not yet supported. Core model types cross
   the boundary as uniffi *remote* types (see `rust/src/remote.rs`).
