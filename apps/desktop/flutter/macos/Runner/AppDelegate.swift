import Cocoa
import FlutterMacOS

@main
class AppDelegate: FlutterAppDelegate {
  override func applicationSupportsSecureRestorableState(_ app: NSApplication) -> Bool {
    return true
  }

  /// This is a menu bar app: closing the last window hides it rather than
  /// quitting, so the process must survive an empty window list.
  override func applicationShouldTerminateAfterLastWindowClosed(
    _ sender: NSApplication
  ) -> Bool {
    return false
  }

  /// Reopening the running accessory app shows Settings on demand.
  /// Returning false prevents AppKit from restoring another surface.
  override func applicationShouldHandleReopen(
    _ sender: NSApplication,
    hasVisibleWindows flag: Bool
  ) -> Bool {
    MacAppPresentationPlugin.shared?.notifyReopen()
    return false
  }

  override func applicationDidFinishLaunching(_ notification: Notification) {
    super.applicationDidFinishLaunching(notification)
    connectPreferencesMenuItem()
  }

  override func application(_ application: NSApplication, open urls: [URL]) {
    for url in urls where url.scheme?.lowercased() == "linguaray" {
      ProtocolPlugin.shared.open(url)
    }
  }

  /// The template's Preferences… item ships with no action, so it renders
  /// disabled once the menu bar becomes visible. Point it at the Dart side.
  private func connectPreferencesMenuItem() {
    guard let appMenu = NSApp.mainMenu?.items.first?.submenu else { return }

    for item in appMenu.items
    where item.keyEquivalent == ","
      && item.keyEquivalentModifierMask == .command
    {
      item.target = self
      item.action = #selector(openSettings(_:))
      return
    }
  }

  @objc private func openSettings(_ sender: Any?) {
    MacAppPresentationPlugin.shared?.notifyOpenSettings()
  }

}
