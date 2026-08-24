import AVFoundation
import AppKit
import FlutterMacOS
import SystemConfiguration

/// Owns how the process presents itself to macOS: with a Dock icon and the main
/// menu bar (`.regular`), or as a status-bar-only app (`.accessory`).
///
/// The *decision* lives in Dart (`DockIconController`) because it depends on
/// app state — which windows are open, whether the tray icon is enabled. This
/// plugin only applies it, because the transition itself needs AppKit care that
/// has no Dart equivalent:
///
///   * promoting to `.regular` gives the process a Dock icon immediately, but
///     the menu bar is only drawn once the app becomes frontmost;
///   * demoting to `.accessory` while the app is frontmost leaves the stale
///     menu bar on screen until some other app takes over.
///
/// It also carries the two menu/Dock events that only AppKit sees back to Dart:
/// a click on the Dock icon, and the Preferences… item (⌘,).
final class MacAppPresentationPlugin: NSObject, FlutterPlugin {
  static let channelName = "linguaray/mac_app_presentation"

  /// Set by [register] so `AppDelegate` can forward AppKit callbacks without
  /// threading the instance through the engine registrar.
  private(set) static weak var shared: MacAppPresentationPlugin?

  private let channel: FlutterMethodChannel

  private init(channel: FlutterMethodChannel) {
    self.channel = channel
    super.init()
  }

  static func register(with registrar: FlutterPluginRegistrar) {
    let channel = FlutterMethodChannel(
      name: channelName,
      binaryMessenger: registrar.messenger
    )
    let instance = MacAppPresentationPlugin(channel: channel)
    shared = instance
    registrar.addMethodCallDelegate(instance, channel: channel)

    let speechChannel = FlutterMethodChannel(
      name: "linguaray/speech",
      binaryMessenger: registrar.messenger
    )
    SpeechPlugin.register(channel: speechChannel)

    let protocolChannel = FlutterMethodChannel(
      name: "linguaray/protocol",
      binaryMessenger: registrar.messenger
    )
    ProtocolPlugin.register(channel: protocolChannel)

    SystemProxyPlugin.register(messenger: registrar.messenger)
  }

  func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
    switch call.method {
    case "setDockIconVisible":
      guard
        let arguments = call.arguments as? [String: Any],
        let visible = arguments["visible"] as? Bool
      else {
        result(
          FlutterError(
            code: "bad_args",
            message: "Expected a boolean `visible`.",
            details: nil
          ))
        return
      }
      Task { @MainActor in
        Self.setDockIconVisible(visible)
        result(nil)
      }
    case "isDockIconVisible":
      Task { @MainActor in
        result(NSApp.activationPolicy() == .regular)
      }
    default:
      result(FlutterMethodNotImplemented)
    }
  }

  // ──────────────────────────────────────────────────────────────────────────
  // AppKit → Dart
  // ──────────────────────────────────────────────────────────────────────────

  /// The user clicked the Dock icon (or re-launched the app while it runs).
  func notifyReopen() {
    channel.invokeMethod("onReopen", arguments: nil)
  }

  /// The user chose the app menu's Preferences… item (⌘,).
  func notifyOpenSettings() {
    channel.invokeMethod("onOpenSettings", arguments: nil)
  }

  // ──────────────────────────────────────────────────────────────────────────
  // Activation policy
  // ──────────────────────────────────────────────────────────────────────────

  @MainActor
  static func setDockIconVisible(_ visible: Bool) {
    let policy: NSApplication.ActivationPolicy = visible ? .regular : .accessory
    guard NSApp.activationPolicy() != policy else { return }

    NSApp.setActivationPolicy(policy)

    // Both fix-ups have to run after AppKit has settled the policy change,
    // otherwise the activation is applied against the old policy and ignored.
    DispatchQueue.main.async {
      if visible {
        activateApp()
      } else if NSApp.isActive, !hasVisibleActivatableWindow() {
        // Hand the menu bar back to whatever the user was in before. Skipped
        // while one of our own windows is still up — deactivating would blur
        // the mini translator, which closes itself on blur.
        NSApp.deactivate()
      }
    }
  }

  /// `ignoringOtherApps` is deprecated since macOS 14 in favour of the
  /// cooperative `activate()`, but cooperative activation is refused for an app
  /// that has just promoted itself out of `.accessory` — it stays behind the
  /// previous app and never draws its menu bar. Promotion here is always the
  /// result of the user asking for a window, so taking the focus is the
  /// intended behaviour.
  @available(macOS, deprecated: 14.0)
  @MainActor
  private static func activateApp() {
    NSApp.activate(ignoringOtherApps: true)
  }

  @MainActor
  private static func hasVisibleActivatableWindow() -> Bool {
    NSApp.windows.contains { $0.isVisible && $0.canBecomeKey }
  }
}

private enum SystemProxyPlugin {
  private static var channel: FlutterMethodChannel?

  static func register(messenger: FlutterBinaryMessenger) {
    let channel = FlutterMethodChannel(
      name: "linguaray/system_proxy",
      binaryMessenger: messenger
    )
    self.channel = channel
    channel.setMethodCallHandler { call, result in
      guard call.method == "read" else {
        result(FlutterMethodNotImplemented)
        return
      }
      result(readSystemProxy())
    }
  }

  private static func readSystemProxy() -> [String: Any] {
    guard let proxies = SCDynamicStoreCopyProxies(nil) as? [String: Any] else {
      return [:]
    }

    var value: [String: Any] = [:]
    if let endpoint = endpoint(
      proxies,
      enabledKey: kSCPropNetProxiesHTTPEnable,
      hostKey: kSCPropNetProxiesHTTPProxy,
      portKey: kSCPropNetProxiesHTTPPort
    ) {
      value["http"] = endpoint
    }
    if let endpoint = endpoint(
      proxies,
      enabledKey: kSCPropNetProxiesHTTPSEnable,
      hostKey: kSCPropNetProxiesHTTPSProxy,
      portKey: kSCPropNetProxiesHTTPSPort
    ) {
      value["https"] = endpoint
    }
    var bypass = proxies[kSCPropNetProxiesExceptionsList as String] as? [String] ?? []
    if (proxies[kSCPropNetProxiesExcludeSimpleHostnames as String] as? NSNumber)?.boolValue == true
    {
      bypass.append("<local>")
    }
    value["bypass"] = bypass
    return value
  }

  private static func endpoint(
    _ proxies: [String: Any],
    enabledKey: CFString,
    hostKey: CFString,
    portKey: CFString
  ) -> String? {
    guard
      (proxies[enabledKey as String] as? NSNumber)?.boolValue == true,
      let host = proxies[hostKey as String] as? String,
      !host.isEmpty,
      let port = proxies[portKey as String] as? NSNumber,
      port.intValue > 0
    else {
      return nil
    }
    return "\(host):\(port.intValue)"
  }
}

// Flutter platform-channel callbacks and AVSpeechSynthesizer callbacks both
// arrive on the runner's main thread. Swift cannot infer that executor contract
// from the Objective-C protocols, so declare the shared plugin explicitly.
final class SpeechPlugin: NSObject, FlutterPlugin, AVSpeechSynthesizerDelegate,
  @unchecked Sendable
{
  static let shared = SpeechPlugin()
  private let synthesizer = AVSpeechSynthesizer()
  private var channel: FlutterMethodChannel?
  private var activeUtterance: AVSpeechUtterance?

  static func register(channel: FlutterMethodChannel) {
    shared.channel = channel
    shared.synthesizer.delegate = shared
    channel.setMethodCallHandler { call, result in
      shared.handle(call, result: result)
    }
  }

  static func register(with registrar: FlutterPluginRegistrar) {
    let channel = FlutterMethodChannel(
      name: "linguaray/speech",
      binaryMessenger: registrar.messenger
    )
    register(channel: channel)
  }

  func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
    switch call.method {
    case "isAvailable":
      result(true)
    case "stop":
      activeUtterance = nil
      synthesizer.stopSpeaking(at: .immediate)
      notifyState("idle")
      result(nil)
    case "speak":
      guard
        let arguments = call.arguments as? [String: Any],
        let text = arguments["text"] as? String,
        !text.isEmpty
      else {
        result(
          FlutterError(code: "bad_args", message: "Expected text.", details: nil)
        )
        return
      }
      // Invalidate the previous utterance before stopping it so its delayed
      // cancellation delegate callback cannot clear the state of the new one.
      activeUtterance = nil
      synthesizer.stopSpeaking(at: .immediate)
      let utterance = AVSpeechUtterance(string: text)
      if let language = arguments["language"] as? String, !language.isEmpty {
        utterance.voice = AVSpeechSynthesisVoice(language: language)
      }
      activeUtterance = utterance
      synthesizer.speak(utterance)
      result(nil)
    default:
      result(FlutterMethodNotImplemented)
    }
  }

  func speechSynthesizer(
    _ synthesizer: AVSpeechSynthesizer,
    didFinish utterance: AVSpeechUtterance
  ) {
    guard utterance === activeUtterance else { return }
    activeUtterance = nil
    notifyState("idle")
  }

  func speechSynthesizer(
    _ synthesizer: AVSpeechSynthesizer,
    didCancel utterance: AVSpeechUtterance
  ) {
    guard utterance === activeUtterance else { return }
    activeUtterance = nil
    notifyState("interrupted")
  }

  private func notifyState(_ state: String) {
    channel?.invokeMethod("stateChanged", arguments: state)
  }
}

final class ProtocolPlugin: NSObject, FlutterPlugin {
  static let shared = ProtocolPlugin()
  private var channel: FlutterMethodChannel?
  private var pendingURLs: [String] = []

  static func register(channel: FlutterMethodChannel) {
    shared.channel = channel
    for rawURL in shared.pendingURLs {
      channel.invokeMethod("open", arguments: rawURL)
    }
    shared.pendingURLs.removeAll(keepingCapacity: false)
  }

  static func register(with registrar: FlutterPluginRegistrar) {
    let channel = FlutterMethodChannel(
      name: "linguaray/protocol",
      binaryMessenger: registrar.messenger
    )
    register(channel: channel)
  }

  func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
    result(FlutterMethodNotImplemented)
  }

  func open(_ url: URL) {
    let rawURL = url.absoluteString
    guard let channel else {
      pendingURLs.append(rawURL)
      return
    }
    channel.invokeMethod("open", arguments: rawURL)
  }
}
