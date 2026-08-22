// swift-tools-version: 5.9
//
// Swift Package for the macOS side of the linguaray_runtime Flutter plugin.
//
// Flutter's macOS plugin tooling auto-discovers this manifest because the
// host pubspec declares `pluginClass: LinguarayRuntimePlugin` (combined with
// `ffiPlugin: true`). When the Flutter build runs, it adds this package as
// a SPM dependency of `FlutterGeneratedPluginSwiftPackage`, so any host app
// can simply `import linguaray_runtime` from native Swift code without touching
// its Xcode project.
//
// Two targets:
//
//   * `linguaray_runtimeFFI` exposes the C ABI declared in the
//     generated FFI header to Swift via a Clang module.
//   * `linguaray_runtime` re-exports the uniffi-rs generated Swift binding
//     (`Generated/linguaray_runtime.swift`) plus a tiny `FlutterPlugin` stub
//     (`LinguarayRuntimePlugin.swift`) whose `register(with:)` performs a one-shot
//     `dlopen` of the bundled runtime framework. The dlopen is needed
//     because Dart's native_assets system bundles the cdylib but only loads
//     it lazily on the first `@Native(...)` call - native Swift call sites
//     would otherwise see NULL function pointers.
//
// `dynamic_lookup` defers symbol resolution to runtime; the symbols come
// from the framework `dlopen`'d in `register(with:)`.
import PackageDescription

let package = Package(
  name: "linguaray_runtime",
  platforms: [.macOS(.v10_15)],
  products: [
    .library(name: "linguaray-runtime", targets: ["linguaray_runtime"])
  ],
  dependencies: [
    // Provided by Flutter's macOS SPM tooling at build time. Resolves to
    // `FlutterMacOS.framework` so we can `import FlutterMacOS` from
    // `LinguarayRuntimePlugin`.
    .package(name: "FlutterFramework", path: "../FlutterFramework")
  ],
  targets: [
    .target(
      name: "linguaray_runtime",
      dependencies: [
        "linguaray_runtimeFFI",
        .product(name: "FlutterFramework", package: "FlutterFramework"),
      ],
      path: "Sources/linguaray_runtime",
      linkerSettings: [
        .unsafeFlags(["-Xlinker", "-undefined", "-Xlinker", "dynamic_lookup"])
      ]
    ),
    .target(
      name: "linguaray_runtimeFFI",
      path: "Sources/linguaray_runtimeFFI",
      publicHeadersPath: "include"
    ),
  ]
)
