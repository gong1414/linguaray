import Darwin
import FlutterMacOS
import Foundation

/// Flutter plugin entry point for the macOS side of `linguaray_runtime`.
///
/// Flutter's plugin auto-registration calls `LinguarayRuntimePlugin.register(with:)`
/// from `GeneratedPluginRegistrant` very early in app startup. We use that as
/// a hook to `dlopen` the bundled runtime framework so that subsequent
/// Swift calls into the uniffi-rs generated binding can resolve their FFI
/// symbols at runtime.
///
/// The plugin does not register any method channels - all Dart <-> Rust calls
/// happen through Dart's native_assets system. This class exists purely so
/// that Flutter's macOS tooling will pick up the SPM `Package.swift` shipped
/// alongside this file and link `linguaray_runtime` into the host binary.
public final class LinguarayRuntimePlugin: NSObject, FlutterPlugin {
  public static func register(with registrar: FlutterPluginRegistrar) {
    loadRustDylibIfNeeded()
    SystemTranslationServiceBridge.start()
  }
}

private var didLoadDylib = false

private func loadRustDylibIfNeeded() {
  if didLoadDylib { return }

  guard let frameworksURL = Bundle.main.privateFrameworksURL else {
    NSLog("[linguaray_runtime] Bundle.main.privateFrameworksURL is nil; skipping dlopen")
    return
  }

  let dylib =
    frameworksURL
    .appendingPathComponent("linguaray_runtime.framework")
    .appendingPathComponent("linguaray_runtime")

  if dlopen(dylib.path, RTLD_NOW | RTLD_GLOBAL) == nil {
    let raw = dlerror()
    let msg = raw.flatMap { String(cString: $0) } ?? "(no dlerror)"
    NSLog("[linguaray_runtime] dlopen failed for %@: %@", dylib.path, msg)
    return
  }

  didLoadDylib = true
}
