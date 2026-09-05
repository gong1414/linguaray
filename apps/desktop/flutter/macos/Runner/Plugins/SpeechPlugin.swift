import AVFoundation
import FlutterMacOS

// Flutter platform-channel callbacks and AVSpeechSynthesizer callbacks both
// arrive on the runner's main thread. Swift cannot infer that executor contract
// from the Objective-C protocols, so declare the shared plugin explicitly.
final class SpeechPlugin: NSObject, FlutterPlugin, AVSpeechSynthesizerDelegate,
  @unchecked Sendable
{
  static let channelName = "linguaray/speech"
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
      name: channelName,
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
