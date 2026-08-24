import 'dart:async';

import '../utils/platform_util.dart';
import 'mac_app_presentation.dart';

/// Single decision point for whether the app shows a Dock icon and the macOS
/// main menu bar.
///
/// LinguaRay ships as a menu bar app (`LSUIElement` in Info.plist), so it
/// launches without either. Two situations promote it to a regular app:
///
///   * **the settings window is on screen** — it needs a Dock icon to be
///     reachable through ⌘-Tab, Mission Control and the Dock, and it needs the
///     menu bar for ⌘, ⌘W and ⌘Q;
///   * **the tray icon is turned off** — otherwise the app would have no
///     visible entry point at all, and the only way back would be a global
///     shortcut the user may never have set.
///
/// The mini translator deliberately does *not* promote. It is a popover
/// anchored under the tray icon that closes as soon as it loses focus, and
/// flashing a Dock icon on every quick translation is worse than going without
/// a menu bar for the seconds it is open. Flutter handles its editing
/// shortcuts while the transient surface has focus.
///
/// Only macOS distinguishes the two policies, so everything here is a no-op
/// elsewhere.
class DockIconController {
  DockIconController._();

  static final DockIconController instance = DockIconController._();

  /// LinguaRay launches as a menu-bar accessory with no visible window.
  bool _isSettingsWindowVisible = false;
  bool _isTrayIconVisible = true;

  /// Last value pushed to AppKit, so repeated state changes that do not move
  /// the outcome never touch the activation policy. Seeded to match LSUIElement.
  bool? _appliedDockIconVisible = false;

  bool get shouldShowDockIcon =>
      _isSettingsWindowVisible || !_isTrayIconVisible;

  void setSettingsWindowVisible(bool value) {
    if (_isSettingsWindowVisible == value) return;
    _isSettingsWindowVisible = value;
    _apply();
  }

  void setTrayIconVisible(bool value) {
    if (_isTrayIconVisible == value) return;
    _isTrayIconVisible = value;
    _apply();
  }

  void _apply() {
    if (!kIsMacOS) return;

    final visible = shouldShowDockIcon;
    if (_appliedDockIconVisible == visible) return;
    _appliedDockIconVisible = visible;
    unawaited(MacAppPresentation.setDockIconVisible(visible));
  }
}

/// Singleton accessor, matching the other stores/controllers in `services/`.
final dockIconController = DockIconController.instance;
