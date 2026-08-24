import Cocoa
import FlutterMacOS

class MainFlutterWindow: NSWindow {
  override func awakeFromNib() {
    let flutterViewController = FlutterViewController()
    let windowFrame = frame
    contentViewController = flutterViewController
    setFrame(windowFrame, display: true)

    RegisterGeneratedPlugins(registry: flutterViewController)
    MacAppPresentationPlugin.register(
      with: flutterViewController.registrar(forPlugin: "MacAppPresentationPlugin")
    )

    super.awakeFromNib()

    // The product is menu-bar resident. Dart presents this stable host only
    // for Settings or a transient translation panel.
    isReleasedWhenClosed = false
    orderOut(nil)
  }

  override func performClose(_ sender: Any?) {
    orderOut(sender)
  }
}
