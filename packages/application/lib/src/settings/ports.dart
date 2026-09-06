import 'package:linguaray_application/src/settings/models.dart';

abstract interface class PermissionRepository {
  Future<AccessSnapshot> refresh();

  Future<AccessSnapshot> requestAccessibility();

  Future<AccessSnapshot> requestScreenRecording();
}

abstract interface class ShortcutRepository {
  Future<List<ShortcutRecord>> load();

  Future<void> beginRecording();

  Future<void> endRecording();

  Future<void> setAccelerator({
    required String actionId,
    required String accelerator,
  });

  Future<void> clear(String actionId);

  Future<void> resetDefaults();
}
