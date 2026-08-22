/// Stable-channel window coordination for LinguaRay.
///
/// LinguaRay intentionally stays on Flutter stable rather than relying on the
/// experimental multi-window API from the main channel, so the
/// workbench and quick translator are mutually-exclusive Flutter surfaces in
/// one native host window. `nativeapi` owns all native sizing, positioning,
/// focus, tray and display behavior.
library;

import 'package:flutter/widgets.dart';
import 'package:go_router/go_router.dart';
import 'package:nativeapi/nativeapi.dart';

import '../platform/onboarding_controller.dart';
import '../utils/platform_util.dart';
import 'dock_icon_controller.dart';

const kWorkbenchWindowTitle = 'LinguaRay';
const kMiniTranslatorWindowTitle = 'LinguaRay Quick Translate';
const _kWorkbenchWindowSize = Size(840, 560);
const _kWorkbenchWindowMinimumSize = Size(840, 560);
const _kMiniTranslatorInitialSize = Size(396, 420);
const _kMiniTranslatorMinimumSize = Size(396, 180);
const _kMiniTranslatorTrayGap = 10.0;
const _kMiniTranslatorCursorGap = 12.0;

GoRouter? _workbenchRouter;
String _pendingWorkbenchLocation = '/translate';
bool _workbenchWindowConfigured = false;
int _surfaceSwitchGeneration = 0;

enum AppSurface { workbench, miniTranslator }

AppSurface _requestedSurface = AppSurface.workbench;

/// Exactly one surface is mounted at a time, which prevents duplicate title
/// frames and stacked workbench/quick-window layers.
final ValueNotifier<AppSurface> appSurface = ValueNotifier(
  AppSurface.workbench,
);

/// Whether the mounted quick translator still owns native window sizing.
///
/// A workbench transition deliberately keeps the compact Flutter surface
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

const workbenchWindowController = AppWindowController();
const miniTranslatorWindowController = AppWindowController();

/// Text handed from the quick translator to the workbench translation page.
final ValueNotifier<String?> workbenchTextHandoff = ValueNotifier(null);

enum WorkbenchDestination {
  welcome('/welcome'),
  translate('/translate'),
  history('/history'),
  glossary('/glossary'),
  vocabulary('/vocabulary'),
  settingsGeneral('/settings/general'),
  settingsServices('/settings/services'),
  settingsShortcuts('/settings/shortcuts'),
  settingsProviders('/settings/providers'),
  settingsAdvanced('/settings/advanced'),
  settingsUpdates('/settings/updates'),
  settingsAbout('/settings/about');

  const WorkbenchDestination(this.location);

  final String location;
}

void attachWorkbenchRouter(GoRouter router) {
  _workbenchRouter = router;
}

void detachWorkbenchRouter(GoRouter router) {
  if (_workbenchRouter == router) _workbenchRouter = null;
}

String get pendingWorkbenchLocation => _pendingWorkbenchLocation;

void showWorkbenchWindow({WorkbenchDestination? destination, String? text}) {
  final target =
      destination ??
      (onboardingController.isComplete
          ? WorkbenchDestination.translate
          : WorkbenchDestination.welcome);
  _pendingWorkbenchLocation = target.location;
  if (text != null) workbenchTextHandoff.value = text;
  _workbenchRouter?.go(_pendingWorkbenchLocation);
  focusWorkbenchWindow();
}

void focusWorkbenchWindow() {
  final window = workbenchWindowController.window;
  final switchedSurface = appSurface.value != AppSurface.workbench;
  final switchGeneration = ++_surfaceSwitchGeneration;
  _requestedSurface = AppSurface.workbench;

  // Resizing a top-right compact window expands its native frame before the
  // Flutter backing surface receives the new metrics. Keeping it visible in
  // that interval exposes the desktop as a white strip beside the old 396 px
  // surface. Hide only for the cross-surface transition and reveal it after
  // the first workbench frame has been laid out.
  if (switchedSurface) window.hide();

  window.title = kWorkbenchWindowTitle;
  window.titleBarStyle = TitleBarStyle.hidden;
  window.windowControlButtonsVisible = kIsMacOS;
  window.isResizable = true;
  window.setMinimumSize(
    _kWorkbenchWindowMinimumSize.width,
    _kWorkbenchWindowMinimumSize.height,
  );
  if (switchedSurface || window.size != _kWorkbenchWindowSize) {
    window.setSize(_kWorkbenchWindowSize.width, _kWorkbenchWindowSize.height);
  }
  if (!_workbenchWindowConfigured) {
    _workbenchWindowConfigured = true;
    window.center();
  }
  dockIconController.setWorkbenchWindowVisible(true);
  if (window.isMinimized) window.restore();

  if (switchedSurface) {
    // Keep the compact surface mounted while the native host expands. Mounting
    // the 840 px workbench into the 396 px quick-window constraints for even
    // one frame causes real overflows in the toolbar and settings rail.
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (switchGeneration != _surfaceSwitchGeneration) return;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (switchGeneration != _surfaceSwitchGeneration) return;
        appSurface.value = AppSurface.workbench;
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (switchGeneration != _surfaceSwitchGeneration) return;
          window.show();
          window.focus();
        });
        WidgetsBinding.instance.scheduleFrame();
      });
      WidgetsBinding.instance.scheduleFrame();
    });
  } else {
    window.show();
    window.focus();
  }
}

bool get isWorkbenchWindowOpen {
  final window = workbenchWindowController.window;
  return appSurface.value == AppSurface.workbench &&
      (window.isVisible || window.isMinimized);
}

Future<void> handleTrayIconClick({Rect? trayBounds}) async {
  if (isWorkbenchWindowOpen) {
    focusWorkbenchWindow();
  } else {
    await showMiniTranslatorWindow(trayBounds: trayBounds);
  }
}

void showSettingsWindow() {
  showWorkbenchWindow(destination: WorkbenchDestination.settingsGeneral);
}

void hideWorkbenchWindow() {
  if (appSurface.value != AppSurface.workbench) return;
  workbenchWindowController.window.hide();
  dockIconController.setWorkbenchWindowVisible(false);
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
    appSurface.value = AppSurface.miniTranslator;
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
  window.show();
  window.focus();
  dockIconController.setWorkbenchWindowVisible(false);
}

void hideMiniTranslatorWindow() {
  if (appSurface.value != AppSurface.miniTranslator) return;
  miniTranslatorWindowController.window.hide();
}

bool get isMiniTranslatorWindowVisible =>
    appSurface.value == AppSurface.miniTranslator &&
    miniTranslatorWindowController.window.isVisible;

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
