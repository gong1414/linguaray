import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../app/dependencies.dart';

final shortcutsViewModelProvider =
    NotifierProvider<ShortcutsViewModel, ShortcutsViewState>(
      ShortcutsViewModel.new,
    );

final class ShortcutsViewState {
  const ShortcutsViewState({
    this.shortcuts = const [],
    this.recordingActionId,
    this.loading = true,
    this.errorCode,
  });

  final List<ShortcutRecord> shortcuts;
  final String? recordingActionId;
  final bool loading;
  final String? errorCode;

  ShortcutsViewState copyWith({
    List<ShortcutRecord>? shortcuts,
    Object? recordingActionId = _unset,
    bool? loading,
    Object? errorCode = _unset,
  }) {
    return ShortcutsViewState(
      shortcuts: shortcuts ?? this.shortcuts,
      recordingActionId: identical(recordingActionId, _unset)
          ? this.recordingActionId
          : recordingActionId as String?,
      loading: loading ?? this.loading,
      errorCode: identical(errorCode, _unset)
          ? this.errorCode
          : errorCode as String?,
    );
  }
}

const Object _unset = Object();

final class ShortcutsViewModel extends Notifier<ShortcutsViewState> {
  @override
  ShortcutsViewState build() {
    scheduleMicrotask(reload);
    return const ShortcutsViewState();
  }

  Future<void> reload() async {
    try {
      final shortcuts = await ref.read(shortcutRepositoryProvider).load();
      state = state.copyWith(
        shortcuts: shortcuts,
        loading: false,
        errorCode: null,
      );
    } catch (_) {
      state = state.copyWith(
        loading: false,
        errorCode: AppErrorCode.unknown.wireName,
      );
    }
  }

  Future<void> startRecording(String actionId) async {
    state = state.copyWith(recordingActionId: actionId);
    try {
      await ref.read(shortcutRepositoryProvider).beginRecording();
    } catch (_) {
      state = state.copyWith(recordingActionId: null);
    }
  }

  Future<void> cancelRecording() async {
    state = state.copyWith(recordingActionId: null);
    try {
      await ref.read(shortcutRepositoryProvider).endRecording();
    } catch (_) {
      // The recorder is already closed. A later settings reload or app
      // activation will retry normal shortcut registration.
    }
  }

  Future<void> submitRecording(String accelerator) async {
    final actionId = state.recordingActionId;
    if (actionId == null) return;
    try {
      await ref
          .read(shortcutRepositoryProvider)
          .setAccelerator(actionId: actionId, accelerator: accelerator);
    } catch (_) {
      // Registration status is surfaced by reload; never leave the recorder
      // stuck because persistence or the operating system rejected a key.
    } finally {
      state = state.copyWith(recordingActionId: null);
      await ref.read(shortcutRepositoryProvider).endRecording();
      await reload();
    }
  }

  Future<void> clear(String actionId) async {
    await cancelRecording();
    await ref.read(shortcutRepositoryProvider).clear(actionId);
    await reload();
  }

  Future<void> reset() async {
    await cancelRecording();
    await ref.read(shortcutRepositoryProvider).resetDefaults();
    await reload();
  }
}
