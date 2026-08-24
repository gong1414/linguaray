/// Stable-channel window coordination for LinguaRay.
///
/// LinguaRay intentionally stays on Flutter stable rather than relying on the
/// experimental multi-window API from the main channel, so the
/// settings and quick translator are mutually-exclusive Flutter surfaces in
/// one native host window. `nativeapi` owns all native sizing, positioning,
/// focus, tray and display behavior.
library;

import 'package:flutter/widgets.dart';
import 'package:go_router/go_router.dart';
import 'package:nativeapi/nativeapi.dart';

import '../models/settings_navigation.dart';
import '../utils/platform_util.dart';
import 'dock_icon_controller.dart';
import 'window_positioning.dart';

export '../models/settings_navigation.dart' show SettingsDestination;
export 'window_positioning.dart'
    show
        miniTranslatorPositionAtCursorScreenTopRight,
        miniTranslatorPositionNearCursor,
        miniTranslatorPositionNearPoint,
        ocrWindowPositionNearCursor,
        ocrWindowPositionNearPoint,
        positionPopoverNearPoint;

const kSettingsWindowTitle = 'LinguaRay';
const kMiniTranslatorWindowTitle = 'LinguaRay Quick Translate';
const kOcrWindowTitle = 'LinguaRay OCR';
const _kSettingsWindowSize = Size(1000, 700);
const _kSettingsWindowMinimumSize = Size(780, 520);
const _kMiniTranslatorMinimumSize = Size(396, 180);
const _kOcrWindowMinimumSize = Size(440, 300);

GoRouter? _settingsRouter;
String _pendingSettingsLocation =
    SettingsDestination.settingsTranslation.location;
bool _settingsWindowConfigured = false;
int _surfaceSwitchGeneration = 0;

enum AppSurface { settings, miniTranslator, ocr }

AppSurface _requestedSurface = AppSurface.settings;

/// Exactly one surface is mounted at a time, which prevents duplicate title
/// frames and stacked settings/quick-window layers.
final ValueNotifier<AppSurface> appSurface = ValueNotifier(AppSurface.settings);

/// Whether the mounted quick translator still owns native window sizing.
///
/// A settings transition deliberately keeps the compact Flutter surface
/// mounted until the native host has expanded. Its delayed content-measurement
/// callbacks must not be allowed to shrink that host back to 396 px while the
/// transition is in flight.
bool get canResizeMiniTranslatorWindow =>
    appSurface.value == AppSurface.miniTranslator &&
    _requestedSurface == AppSurface.miniTranslator;

class AppWindowController {
  const AppWindowController();

  Window get window {
    final manager = WindowManager.instance;
    final current = manager.getCurrent();
    if (current != null) return current;
    final windows = manager.getAll();
    if (windows.isEmpty) {
      throw StateError('The Flutter host window is not ready.');
    }
    return windows.first;
  }
}

const settingsWindowController = AppWindowController();
const miniTranslatorWindowController = AppWindowController();
const ocrWindowController = AppWindowController();

void attachSettingsRouter(GoRouter router) {
  _settingsRouter = router;
}

void detachSettingsRouter(GoRouter router) {
  if (_settingsRouter == router) _settingsRouter = null;
}

String get pendingSettingsLocation => _pendingSettingsLocation;

void showSettingsWindow({SettingsDestination? destination}) {
  final target = destination ?? SettingsDestination.settingsTranslation;
  _pendingSettingsLocation = target.location;
  _settingsRouter?.go(_pendingSettingsLocation);
  focusSettingsWindow();
}

void focusSettingsWindow() {
  final window = settingsWindowController.window;
  final switchedSurface = appSurface.value != AppSurface.settings;
  final switchGeneration = ++_surfaceSwitchGeneration;
  _requestedSurface = AppSurface.settings;

  // Keep the native host alive while it expands so Flutter continues to
  // produce frames. A hidden macOS window can stop scheduling frames, leaving
  // the compact surface mounted forever. Opacity prevents the intermediate
  // 396 px backing surface from flashing as a white strip.
  if (switchedSurface) {
    window.opacity = 0;
    if (!window.isVisible) window.showInactive();
  }

  window.title = kSettingsWindowTitle;
  window.titleBarStyle = kIsMacOS ? TitleBarStyle.normal : TitleBarStyle.hidden;
  window.windowControlButtonsVisible = kIsMacOS;
  window.isResizable = true;
  window.setMinimumSize(
    _kSettingsWindowMinimumSize.width,
    _kSettingsWindowMinimumSize.height,
  );
  if (switchedSurface || window.size != _kSettingsWindowSize) {
    window.setSize(_kSettingsWindowSize.width, _kSettingsWindowSize.height);
  }
  if (!_settingsWindowConfigured) {
    _settingsWindowConfigured = true;
    window.center();
  }
  dockIconController.setSettingsWindowVisible(true);
  if (window.isMinimized) window.restore();

  if (switchedSurface) {
    // Resize first, then mount settings while the host is transparent.
    // The post-frame callback reveals only the fully laid-out surface.
    appSurface.value = AppSurface.settings;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (switchGeneration != _surfaceSwitchGeneration) return;
      window.opacity = 1;
      window.show();
      window.focus();
    });
    WidgetsBinding.instance.scheduleFrame();
  } else {
    window.opacity = 1;
    window.show();
    window.focus();
  }
}

bool get isSettingsWindowOpen {
  final window = settingsWindowController.window;
  return appSurface.value == AppSurface.settings &&
      (window.isVisible || window.isMinimized);
}

/// Configures LinguaRay as a tray-resident app without presenting a window.
void initializeResidentApp() {
  final window = settingsWindowController.window;
  _requestedSurface = AppSurface.settings;
  appSurface.value = AppSurface.settings;
  window.title = kSettingsWindowTitle;
  window.titleBarStyle = kIsMacOS ? TitleBarStyle.normal : TitleBarStyle.hidden;
  window.windowControlButtonsVisible = kIsMacOS;
  window.isResizable = true;
  window.setMinimumSize(
    _kSettingsWindowMinimumSize.width,
    _kSettingsWindowMinimumSize.height,
  );
  window.hide();
  dockIconController.setSettingsWindowVisible(false);
}

void hideSettingsWindow() {
  if (appSurface.value != AppSurface.settings) return;
  settingsWindowController.window.hide();
  dockIconController.setSettingsWindowVisible(false);
}

Future<bool> _mountTransientSurface(Window window, AppSurface surface) async {
  _surfaceSwitchGeneration++;
  _requestedSurface = surface;
  final switchedSurface = appSurface.value != surface;
  if (!switchedSurface) return false;

  // A fully hidden macOS window may stop producing Flutter frames. Mount the
  // requested surface while the native host is transparent, then reveal it
  // only after the new frame has completed.
  window.opacity = 0;
  if (!window.isVisible) window.showInactive();
  appSurface.value = surface;
  WidgetsBinding.instance.scheduleFrame();
  await WidgetsBinding.instance.endOfFrame;
  return true;
}

void _configureTransientWindow(
  Window window, {
  required String title,
  required Size minimumSize,
  required bool isResizable,
}) {
  window.title = title;
  window.titleBarStyle = TitleBarStyle.hidden;
  window.windowControlButtonsVisible = false;
  window.isResizable = isResizable;
  window.setMinimumSize(minimumSize.width, minimumSize.height);
}

void _presentTransientWindow(Window window, Offset? position) {
  if (position != null) window.setPosition(position.dx, position.dy);
  window.opacity = 1;
  window.show();
  window.focus();
  dockIconController.setSettingsWindowVisible(false);
}

Future<void> showMiniTranslatorWindow({
  Offset? position,
  Rect? trayBounds,
}) async {
  final window = miniTranslatorWindowController.window;
  final switchedSurface = await _mountTransientSurface(
    window,
    AppSurface.miniTranslator,
  );
  _configureTransientWindow(
    window,
    title: kMiniTranslatorWindowTitle,
    minimumSize: _kMiniTranslatorMinimumSize,
    isResizable: false,
  );
  if (switchedSurface) {
    window.setSize(
      miniTranslatorInitialSize.width,
      miniTranslatorInitialSize.height,
    );
  }

  final newPosition =
      position ??
      (trayBounds != null ? windowPositionBelowTray(trayBounds) : null);
  _presentTransientWindow(window, newPosition);
}

void hideMiniTranslatorWindow() {
  if (appSurface.value != AppSurface.miniTranslator) return;
  miniTranslatorWindowController.window.hide();
}

bool get isMiniTranslatorWindowVisible =>
    appSurface.value == AppSurface.miniTranslator &&
    miniTranslatorWindowController.window.isVisible;

Future<void> showOcrWindow({Offset? position, Rect? trayBounds}) async {
  final window = ocrWindowController.window;
  final switchedSurface = await _mountTransientSurface(window, AppSurface.ocr);
  _configureTransientWindow(
    window,
    title: kOcrWindowTitle,
    minimumSize: _kOcrWindowMinimumSize,
    isResizable: true,
  );
  if (switchedSurface) {
    window.setSize(ocrWindowSize.width, ocrWindowSize.height);
  }
  final newPosition =
      position ??
      (trayBounds != null
          ? windowPositionBelowTray(trayBounds, windowSize: ocrWindowSize)
          : null);
  _presentTransientWindow(window, newPosition);
}

void hideOcrWindow() {
  if (appSurface.value != AppSurface.ocr) return;
  ocrWindowController.window.hide();
}

bool get isOcrWindowVisible =>
    appSurface.value == AppSurface.ocr && ocrWindowController.window.isVisible;
