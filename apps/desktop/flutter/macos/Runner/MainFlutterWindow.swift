import Cocoa
import FlutterMacOS

class MainFlutterWindow: NSWindow {
  override func awakeFromNib() {
    let flutterViewController = FlutterViewController()
    let windowFrame = frame
    contentViewController = flutterViewController
    setFrame(windowFrame, display: true)

    RegisterGeneratedPlugins(registry: flutterViewController)
    MacWindowAppearancePlugin.register(
      with: flutterViewController.registrar(forPlugin: "MacWindowAppearancePlugin")
    )
    NativeTextFieldPlugin.register(
      with: flutterViewController.registrar(forPlugin: "NativeTextFieldPlugin")
    )
    NativeTextPlugin.register(
      with: flutterViewController.registrar(forPlugin: "NativeTextPlugin")
    )
    MacAppPresentationPlugin.register(
      with: flutterViewController.registrar(forPlugin: "MacAppPresentationPlugin")
    )

    super.awakeFromNib()
  }

  override func performClose(_ sender: Any?) {
    orderOut(sender)
  }
}
