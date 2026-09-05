import AppKit
import ApplicationServices
import FlutterMacOS

/// Holds one original editable selection. Never pastes into the current focus.
final class SelectionReplacementPlugin {
  private(set) static var shared: SelectionReplacementPlugin?

  static func register(with registrar: FlutterPluginRegistrar) {
    shared = SelectionReplacementPlugin(messenger: registrar.messenger)
  }

  private struct Target {
    let id: String
    let element: AXUIElement
    let pid: pid_t
    let value: String
    let text: String
    let range: CFRange
    let capturedAt: Date
  }

  private var target: Target?
  private let channel: FlutterMethodChannel

  init(messenger: FlutterBinaryMessenger) {
    channel = FlutterMethodChannel(
      name: "linguaray/selection_replacement", binaryMessenger: messenger)
    channel.setMethodCallHandler { [self] call, result in
      switch call.method {
      case "capture": result(capture())
      case "replace":
        guard let args = call.arguments as? [String: Any],
          let id = args["id"] as? String, let text = args["text"] as? String,
          !text.isEmpty
        else {
          result(FlutterError(code: "bad_args", message: nil, details: nil))
          return
        }
        result(replace(id: id, text: text))
      case "clear":
        target = nil
        result(nil)
      default: result(FlutterMethodNotImplemented)
      }
    }
  }

  private func attribute(_ element: AXUIElement, _ name: String) -> CFTypeRef? {
    var value: CFTypeRef?
    guard AXUIElementCopyAttributeValue(element, name as CFString, &value) == .success else {
      return nil
    }
    return value
  }

  private func selectedRange(_ element: AXUIElement) -> CFRange? {
    guard let raw = attribute(element, kAXSelectedTextRangeAttribute),
      CFGetTypeID(raw) == AXValueGetTypeID()
    else { return nil }
    var range = CFRange()
    guard AXValueGetValue(raw as! AXValue, .cfRange, &range) else { return nil }
    return range
  }

  private func capture() -> [String: String]? {
    target = nil
    guard AXIsProcessTrusted(),
      let app = NSWorkspace.shared.frontmostApplication,
      app.processIdentifier != ProcessInfo.processInfo.processIdentifier,
      let raw = attribute(
        AXUIElementCreateApplication(app.processIdentifier), kAXFocusedUIElementAttribute),
      CFGetTypeID(raw) == AXUIElementGetTypeID()
    else { return nil }
    let element = raw as! AXUIElement
    guard attribute(element, kAXSubroleAttribute) as? String != kAXSecureTextFieldSubrole,
      let text = attribute(element, kAXSelectedTextAttribute) as? String, !text.isEmpty,
      let value = attribute(element, kAXValueAttribute) as? String,
      let range = selectedRange(element), range.location >= 0, range.length > 0,
      range.location <= (value as NSString).length,
      range.length <= (value as NSString).length - range.location,
      (value as NSString).substring(with: NSRange(location: range.location, length: range.length))
        == text
    else { return nil }
    var writable = DarwinBoolean(false)
    guard
      AXUIElementIsAttributeSettable(element, kAXSelectedTextAttribute as CFString, &writable)
        == .success,
      writable.boolValue
    else { return nil }
    let id = UUID().uuidString
    target = Target(
      id: id, element: element, pid: app.processIdentifier, value: value,
      text: text, range: range, capturedAt: Date())
    return ["id": id, "text": text]
  }

  private func replace(id: String, text: String) -> String {
    guard AXIsProcessTrusted() else { return "permission_denied" }
    guard let saved = target, saved.id == id,
      Date().timeIntervalSince(saved.capturedAt) < 300,
      let app = NSRunningApplication(processIdentifier: saved.pid), !app.isTerminated,
      attribute(saved.element, kAXValueAttribute) as? String == saved.value,
      attribute(saved.element, kAXSelectedTextAttribute) as? String == saved.text,
      let range = selectedRange(saved.element),
      range.location == saved.range.location, range.length == saved.range.length
    else {
      target = nil
      return "selection_changed"
    }
    // The AX write targets the saved element directly; no clipboard or synthetic keystrokes.
    guard
      AXUIElementSetAttributeValue(
        saved.element, kAXSelectedTextAttribute as CFString, text as CFString) == .success
    else { return "replace_unsupported" }
    target = nil
    let expected = (saved.value as NSString).replacingCharacters(
      in: NSRange(location: saved.range.location, length: saved.range.length), with: text)
    guard attribute(saved.element, kAXValueAttribute) as? String == expected else {
      return "replace_unsupported"
    }
    app.activate(options: [])
    return "replaced"
  }
}
