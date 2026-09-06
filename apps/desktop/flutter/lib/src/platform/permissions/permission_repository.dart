// ignore_for_file: prefer_initializing_formals

import 'package:linguaray_application/linguaray_application.dart';

import '../platform_types.dart';
import 'permission_controller.dart';

final class ControllerPermissionRepository implements PermissionRepository {
  ControllerPermissionRepository({required PermissionController controller})
    : _controller = controller;

  final PermissionController _controller;

  @override
  Future<AccessSnapshot> refresh() async {
    final snapshot = await _controller.refresh();
    return _map(snapshot);
  }

  @override
  Future<AccessSnapshot> requestAccessibility() async {
    return _map(await _controller.requestAccessibility());
  }

  @override
  Future<AccessSnapshot> requestScreenRecording() async {
    return _map(await _controller.requestScreenRecording());
  }

  AccessSnapshot _map(PermissionSnapshot snapshot) {
    return AccessSnapshot(
      accessibility: _state(snapshot.accessibility),
      screenRecording: _state(snapshot.screenRecording),
    );
  }

  AccessState _state(PermissionState state) => switch (state) {
    PermissionState.granted => AccessState.granted,
    PermissionState.denied => AccessState.denied,
    PermissionState.notRequired => AccessState.notRequired,
    PermissionState.unknown => AccessState.unknown,
  };
}
