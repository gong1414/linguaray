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

import '../utils/platform_util.dart';
import 'dock_icon_controller.dart';

const kSettingsWindowTitle = 'LinguaRay';
const kMiniTranslatorWindowTitle = 'LinguaRay Quick Translate';
const kOcrWindowTitle = 'LinguaRay OCR';
const _kSettingsWindowSize = Size(1000, 700);
const _kSettingsWindowMinimumSize = Size(780, 520);
const _kMiniTranslatorInitialSize = Size(396, 420);
const _kMiniTranslatorMinimumSize = Size(396, 180);
const _kOcrWindowSize = Size(600, 520);
const _kOcrWindowMinimumSize = Size(440, 300);
const _kMiniTranslatorTrayGap = 10.0;
const _kMiniTranslatorCursorGap = 12.0;

GoRouter? _settingsRouter;
String _pendingSettingsLocation = '/settings/translation';
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

enum SettingsDestination {
  settingsTranslation('/settings/translation'),
  settingsTranslationServices('/settings/services/translation'),
  settingsFavorites('/settings/favorites'),
  settingsHistory('/settings/history'),
  settingsGlossary('/settings/glossary'),
  settingsVocabulary('/settings/vocabulary'),
  settingsOcr('/settings/ocr'),
  settingsOcrServices('/settings/services/ocr'),
  settingsGeneral('/settings/general'),
  settingsPermissions('/settings/permissions'),
  settingsDataTransfer('/settings/data-transfer'),
  settingsIntegration('/settings/integration'),
  settingsUpdates('/settings/updates'),
  settingsAbout('/settings/about');

  const SettingsDestination(this.location);

  final String location;
}

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

Future<void> showMiniTranslatorWindow({
  Offset? position,
  Rect? trayBounds,
}) async {
  final window = miniTranslatorWindowController.window;
  _surfaceSwitchGeneration++;
  _requestedSurface = AppSurface.miniTranslator;
  final switchedSurface = appSurface.value != AppSurface.miniTranslator;
  if (switchedSurface) {
    // A fully hidden macOS window may stop producing Flutter frames. Mount the
    // compact surface while the native host is transparent, then reveal it
    // only after the new frame has completed.
    window.opacity = 0;
    if (!window.isVisible) window.showInactive();
    appSurface.value = AppSurface.miniTranslator;
    WidgetsBinding.instance.scheduleFrame();
    await WidgetsBinding.instance.endOfFrame;
  }

  window.title = kMiniTranslatorWindowTitle;
  window.titleBarStyle = TitleBarStyle.hidden;
  window.windowControlButtonsVisible = false;
  window.isResizable = false;
  window.setMinimumSize(
    _kMiniTranslatorMinimumSize.width,
    _kMiniTranslatorMinimumSize.height,
  );
  if (switchedSurface) {
    window.setSize(
      _kMiniTranslatorInitialSize.width,
      _kMiniTranslatorInitialSize.height,
    );
  }

  final newPosition =
      position ??
      (trayBounds != null
          ? _miniTranslatorPositionBelowTray(trayBounds)
          : null);
  if (newPosition != null) {
    window.setPosition(newPosition.dx, newPosition.dy);
  }
  window.opacity = 1;
  window.show();
  window.focus();
  dockIconController.setSettingsWindowVisible(false);
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
  _surfaceSwitchGeneration++;
  _requestedSurface = AppSurface.ocr;
  final switchedSurface = appSurface.value != AppSurface.ocr;
  if (switchedSurface) {
    window.opacity = 0;
    if (!window.isVisible) window.showInactive();
    appSurface.value = AppSurface.ocr;
    WidgetsBinding.instance.scheduleFrame();
    await WidgetsBinding.instance.endOfFrame;
  }

  window.title = kOcrWindowTitle;
  window.titleBarStyle = TitleBarStyle.hidden;
  window.windowControlButtonsVisible = false;
  window.isResizable = true;
  window.setMinimumSize(
    _kOcrWindowMinimumSize.width,
    _kOcrWindowMinimumSize.height,
  );
  if (switchedSurface) {
    window.setSize(_kOcrWindowSize.width, _kOcrWindowSize.height);
  }
  final newPosition =
      position ??
      (trayBounds != null
          ? _miniTranslatorPositionBelowTray(
              trayBounds,
              windowSize: _kOcrWindowSize,
            )
          : null);
  if (newPosition != null) {
    window.setPosition(newPosition.dx, newPosition.dy);
  }
  window.opacity = 1;
  window.show();
  window.focus();
  dockIconController.setSettingsWindowVisible(false);
}

void hideOcrWindow() {
  if (appSurface.value != AppSurface.ocr) return;
  ocrWindowController.window.hide();
}

bool get isOcrWindowVisible =>
    appSurface.value == AppSurface.ocr && ocrWindowController.window.isVisible;

Offset? _miniTranslatorPositionBelowTray(Rect trayBounds, {Size? windowSize}) {
  final size = windowSize ?? _kMiniTranslatorInitialSize;
  final anchor = _resolveTrayAnchor(trayBounds);
  if (anchor == null) return null;

  if (!kIsMacOS) {
    final position = Offset(
      anchor.bounds.left - (size.width - anchor.bounds.width) / 2,
      anchor.bounds.bottom + _kMiniTranslatorTrayGap,
    );
    return _clampPositionToDisplay(position, size, anchor.display);
  }

  final displayBounds = _displayBounds(anchor.display);
  final menuBarBottom = anchor.display.workArea.top > displayBounds.top
      ? anchor.display.workArea.top
      : displayBounds.top + anchor.bounds.height;
  final position = Offset(
    anchor.bounds.center.dx - size.width / 2,
    menuBarBottom + _kMiniTranslatorTrayGap,
  );
  return _clampPositionToDisplay(position, size, anchor.display);
}

/// Places the quick window next to [point] and keeps it in the display's work
/// area. It flips around the pointer before falling back to clamping.
Offset? miniTranslatorPositionNearPoint(Offset point, {Size? windowSize}) {
  final displays = DisplayManager.instance.getAll();
  if (displays.isEmpty) return null;

  Display? pointDisplay;
  for (final display in displays) {
    if (_displayBounds(display).contains(point)) {
      pointDisplay = display;
      break;
    }
  }

  pointDisplay ??= displays.first;
  return positionPopoverNearPoint(
    point: point,
    windowSize: windowSize ?? _kMiniTranslatorInitialSize,
    workArea: pointDisplay.workArea,
  );
}

Offset? miniTranslatorPositionNearCursor({Size? windowSize}) =>
    miniTranslatorPositionNearPoint(
      DisplayManager.instance.getCursorPosition(),
      windowSize: windowSize,
    );

Offset? ocrWindowPositionNearPoint(Offset point) =>
    miniTranslatorPositionNearPoint(point, windowSize: _kOcrWindowSize);

Offset? ocrWindowPositionNearCursor() =>
    ocrWindowPositionNearPoint(DisplayManager.instance.getCursorPosition());

/// Compatibility alias retained for upstream callers.
Offset? miniTranslatorPositionAtCursorScreenTopRight({Size? windowSize}) =>
    miniTranslatorPositionNearCursor(windowSize: windowSize);

@visibleForTesting
Offset positionPopoverNearPoint({
  required Offset point,
  required Size windowSize,
  required Rect workArea,
}) {
  final candidates = [
    Offset(
      point.dx + _kMiniTranslatorCursorGap,
      point.dy + _kMiniTranslatorCursorGap,
    ),
    Offset(
      point.dx + _kMiniTranslatorCursorGap,
      point.dy - windowSize.height - _kMiniTranslatorCursorGap,
    ),
    Offset(
      point.dx - windowSize.width - _kMiniTranslatorCursorGap,
      point.dy + _kMiniTranslatorCursorGap,
    ),
    Offset(
      point.dx - windowSize.width - _kMiniTranslatorCursorGap,
      point.dy - windowSize.height - _kMiniTranslatorCursorGap,
    ),
  ];
  for (final candidate in candidates) {
    final bounds = candidate & windowSize;
    if (bounds.left >= workArea.left &&
        bounds.top >= workArea.top &&
        bounds.right <= workArea.right &&
        bounds.bottom <= workArea.bottom) {
      return candidate;
    }
  }
  final first = candidates.first;
  return Offset(
    _clampDouble(first.dx, workArea.left, workArea.right - windowSize.width),
    _clampDouble(first.dy, workArea.top, workArea.bottom - windowSize.height),
  );
}

Offset _clampPositionToDisplay(
  Offset position,
  Size windowSize,
  Display display,
) {
  final workArea = display.workArea;
  return Offset(
    _clampDouble(position.dx, workArea.left, workArea.right - windowSize.width),
    _clampDouble(
      position.dy,
      workArea.top,
      workArea.bottom - windowSize.height,
    ),
  );
}

_TrayAnchor? _resolveTrayAnchor(Rect rawBounds) {
  final displays = DisplayManager.instance.getAll();
  if (displays.isEmpty) return null;

  final rawCenter = rawBounds.center;
  for (final display in displays) {
    if (_displayBounds(display).contains(rawCenter)) {
      return _TrayAnchor(
        display: display,
        bounds: _trayBoundsOnDisplay(rawBounds, display),
      );
    }
  }
  for (final display in displays) {
    if (_containsHorizontally(_displayBounds(display), rawCenter.dx)) {
      return _TrayAnchor(
        display: display,
        bounds: _trayBoundsOnDisplay(rawBounds, display),
      );
    }
  }
  for (final display in displays) {
    final normalizedBounds = _normalizeScaledTrayBounds(rawBounds, display);
    if (_containsHorizontally(
      _displayBounds(display),
      normalizedBounds.center.dx,
    )) {
      return _TrayAnchor(
        display: display,
        bounds: _trayBoundsOnDisplay(normalizedBounds, display),
      );
    }
  }

  displays.sort((a, b) {
    final aDistance = _distanceSquared(_displayBounds(a).center, rawCenter);
    final bDistance = _distanceSquared(_displayBounds(b).center, rawCenter);
    return aDistance.compareTo(bDistance);
  });
  final display = displays.first;
  return _TrayAnchor(
    display: display,
    bounds: _trayBoundsOnDisplay(rawBounds, display),
  );
}

Rect _displayBounds(Display display) {
  return Rect.fromLTWH(
    display.position.dx,
    display.position.dy,
    display.size.width,
    display.size.height,
  );
}

Rect _trayBoundsOnDisplay(Rect bounds, Display display) {
  return Rect.fromLTWH(
    bounds.left,
    _displayBounds(display).top,
    bounds.width,
    bounds.height,
  );
}

Rect _normalizeScaledTrayBounds(Rect bounds, Display display) {
  final scaleFactor = display.scaleFactor;
  if (scaleFactor == 0 || scaleFactor == 1) return bounds;

  final displayBounds = _displayBounds(display);
  return Rect.fromLTWH(
    displayBounds.left +
        (bounds.left - displayBounds.left * scaleFactor) / scaleFactor,
    displayBounds.top +
        (bounds.top - displayBounds.top * scaleFactor) / scaleFactor,
    bounds.width / scaleFactor,
    bounds.height / scaleFactor,
  );
}

bool _containsHorizontally(Rect rect, double x) =>
    x >= rect.left && x <= rect.right;

double _distanceSquared(Offset a, Offset b) {
  final dx = a.dx - b.dx;
  final dy = a.dy - b.dy;
  return dx * dx + dy * dy;
}

double _clampDouble(double value, double min, double max) {
  if (max < min) return min;
  return value.clamp(min, max).toDouble();
}

class _TrayAnchor {
  const _TrayAnchor({required this.display, required this.bounds});

  final Display display;
  final Rect bounds;
}
