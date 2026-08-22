import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../config/dependencies.dart';

final permissionsViewModelProvider =
    NotifierProvider<PermissionsViewModel, AccessSnapshot>(
      PermissionsViewModel.new,
    );

final class PermissionsViewModel extends Notifier<AccessSnapshot> {
  @override
  AccessSnapshot build() {
    scheduleMicrotask(refresh);
    return const AccessSnapshot.unknown();
  }

  Future<void> refresh() async {
    state = await ref.read(permissionRepositoryProvider).refresh();
  }

  Future<void> requestAccessibility() async {
    state = await ref.read(permissionRepositoryProvider).requestAccessibility();
  }

  Future<void> requestScreenRecording() async {
    state = await ref
        .read(permissionRepositoryProvider)
        .requestScreenRecording();
  }
}
