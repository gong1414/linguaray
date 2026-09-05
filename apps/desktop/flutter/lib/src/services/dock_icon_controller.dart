import 'dart:async';

import '../utils/platform_util.dart';
import 'mac_app_presentation.dart';

/// Every LinguaRay surface is a menu-bar accessory, including Settings.
/// Reopening the app and global shortcuts remain available when its tray is hidden.
class DockIconController {
  DockIconController._();

  static final DockIconController instance = DockIconController._();
  bool _applied = false;

  void setSettingsWindowVisible(bool value) => _apply();
  void setTrayIconVisible(bool value) => _apply();

  void _apply() {
    if (!kIsMacOS || _applied) return;
    _applied = true;
    unawaited(MacAppPresentation.setDockIconVisible(false));
  }
}

final dockIconController = DockIconController.instance;
