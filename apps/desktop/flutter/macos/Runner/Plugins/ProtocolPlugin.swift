import FlutterMacOS

final class ProtocolPlugin: NSObject, FlutterPlugin {
  static let channelName = "linguaray/protocol"
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
      name: channelName,
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
