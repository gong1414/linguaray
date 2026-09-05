import 'package:flutter/widgets.dart';
import 'package:nativeapi/nativeapi.dart';

import '../utils/platform_util.dart';

const miniTranslatorInitialSize = Size(720, 420);
const ocrWindowSize = Size(600, 520);
const _kMiniTranslatorTrayGap = 10.0;
const _kMiniTranslatorCursorGap = 12.0;

/// Used after content measurement so a wider reading window and long results
/// remain inside the current display, including scaled and secondary displays.
Rect fitPopoverToWorkArea({
  required Offset position,
  required Size desiredSize,
  required Rect workArea,
}) {
  final size = Size(
    desiredSize.width.clamp(0, workArea.width),
    desiredSize.height.clamp(0, workArea.height),
  );
  return Offset(
        _clampDouble(position.dx, workArea.left, workArea.right - size.width),
        _clampDouble(position.dy, workArea.top, workArea.bottom - size.height),
      ) &
      size;
}

Offset? windowPositionBelowTray(Rect trayBounds, {Size? windowSize}) {
  final size = windowSize ?? miniTranslatorInitialSize;
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
    windowSize: windowSize ?? miniTranslatorInitialSize,
    workArea: pointDisplay.workArea,
  );
}

Offset? miniTranslatorPositionNearCursor({Size? windowSize}) =>
    miniTranslatorPositionNearPoint(
      DisplayManager.instance.getCursorPosition(),
      windowSize: windowSize,
    );

Offset? ocrWindowPositionNearPoint(Offset point) =>
    miniTranslatorPositionNearPoint(point, windowSize: ocrWindowSize);

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
