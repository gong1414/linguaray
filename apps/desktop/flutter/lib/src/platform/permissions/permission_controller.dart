import 'dart:io';

import 'package:flutter/foundation.dart';

import '../../app/runtime.dart';
import '../platform_types.dart';

class PermissionSnapshot {
  const PermissionSnapshot({
    required this.accessibility,
    required this.screenRecording,
  });

  const PermissionSnapshot.unknown()
    : accessibility = PermissionState.unknown,
      screenRecording = PermissionState.unknown;

  final PermissionState accessibility;
  final PermissionState screenRecording;
}

/// Reads permissions from the OS every time [refresh] is called.
///
/// No result is cached across an operation boundary: callers refresh before
/// selection/capture, and lifecycle owners refresh after focus or app resume.
class PermissionController extends ChangeNotifier {
  PermissionSnapshot _snapshot = const PermissionSnapshot.unknown();
  Future<PermissionSnapshot>? _inFlightRefresh;

  PermissionSnapshot get snapshot => _snapshot;

  Future<PermissionSnapshot> refresh() {
    final inFlight = _inFlightRefresh;
    if (inFlight != null) return inFlight;

    late final Future<PermissionSnapshot> refresh;
    refresh = _readSnapshot().whenComplete(() {
      if (identical(_inFlightRefresh, refresh)) _inFlightRefresh = null;
    });
    _inFlightRefresh = refresh;
    return refresh;
  }

  Future<PermissionSnapshot> _readSnapshot() async {
    try {
      if (!Platform.isMacOS) {
        _snapshot = const PermissionSnapshot(
          accessibility: PermissionState.notRequired,
          screenRecording: PermissionState.notRequired,
        );
      } else {
        final permission = runtime.permission();
        final results = await Future.wait([
          permission.isAccessibilityPermissionGranted(),
          permission.isScreenRecordingPermissionGranted(),
        ]);
        _snapshot = PermissionSnapshot(
          accessibility: results[0]
              ? PermissionState.granted
              : PermissionState.denied,
          screenRecording: results[1]
              ? PermissionState.granted
              : PermissionState.denied,
        );
      }
      notifyListeners();
      return _snapshot;
    } catch (_) {
      _snapshot = const PermissionSnapshot.unknown();
      notifyListeners();
      return _snapshot;
    }
  }

  Future<PermissionSnapshot> requestAccessibility() async {
    if (Platform.isMacOS) {
      await runtime.permission().requestAccessibilityPermission(
        onlyOpenSystemSettings: false,
      );
    }
    return refresh();
  }

  Future<PermissionSnapshot> requestScreenRecording() async {
    if (Platform.isMacOS) {
      await runtime.permission().requestScreenRecordingPermission(
        onlyOpenSystemSettings: false,
      );
    }
    return refresh();
  }
}

final permissionController = PermissionController();
