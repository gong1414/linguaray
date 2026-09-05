import FlutterMacOS
import SystemConfiguration

enum SystemProxyPlugin {
  static let channelName = "linguaray/system_proxy"

  static func register(messenger: FlutterBinaryMessenger) {
    let channel = FlutterMethodChannel(
      name: channelName,
      binaryMessenger: messenger
    )
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
