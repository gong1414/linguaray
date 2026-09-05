import 'dart:ui';

import 'selection_replacement_controller.dart';

/// Normalized permission state exposed to controllers and widgets.
enum PermissionState { granted, denied, notRequired, unknown }

/// User-facing actions that can be triggered by the tray or a global shortcut.
enum TriggerAction {
  toggleQuickWindow,
  translateSelection,
  openInputWindow,
  translateInput,
  captureAndTranslate,
  captureOcr,
  silentCaptureOcr,
  fileOcr,
  clipboardOcr,
  showOcrWindow,
}

enum ShortcutRegistrationState { unregistered, registered, conflict, invalid }

class ShortcutBinding {
  const ShortcutBinding({
    required this.action,
    required this.accelerator,
    required this.state,
    this.conflictReason,
  });

  final TriggerAction action;
  final String accelerator;
  final ShortcutRegistrationState state;
  final String? conflictReason;
}

enum SelectionReadMethod { simulatedCopy, clipboard, primarySelection }

class SelectionResult {
  const SelectionResult({
    required this.text,
    required this.triggerPosition,
    required this.readMethod,
    this.recoverableError,
    this.replacementTarget,
  });

  final String text;
  final Offset triggerPosition;
  final SelectionReadMethod readMethod;
  final String? recoverableError;
  final SelectionTarget? replacementTarget;
}

class CaptureResult {
  const CaptureResult({
    this.imagePath,
    this.pixelWidth,
    this.pixelHeight,
    this.displayId,
    this.cancelled = false,
    this.failureReason,
  });

  final String? imagePath;
  final int? pixelWidth;
  final int? pixelHeight;
  final String? displayId;
  final bool cancelled;
  final String? failureReason;

  bool get succeeded =>
      !cancelled && failureReason == null && imagePath?.isNotEmpty == true;
}

enum OcrInputSource { screenRegion, file, clipboard }

class OcrTextBlock {
  const OcrTextBlock({required this.text, this.bounds});

  final String text;
  final Rect? bounds;
}

class OcrRecognitionResult {
  const OcrRecognitionResult({
    required this.text,
    required this.source,
    this.blocks = const [],
    this.imagePath,
  });

  final String text;
  final OcrInputSource source;
  final List<OcrTextBlock> blocks;
  final String? imagePath;
}

abstract interface class SecretStore {
  String? read({required String providerId, required String field});

  void write({
    required String providerId,
    required String field,
    required String value,
  });

  void delete({required String providerId, required String field});

  void deleteProvider(String providerId);
}

class PlatformOperationException implements Exception {
  const PlatformOperationException({
    required this.action,
    required this.code,
    required this.message,
  });

  final TriggerAction action;
  final String code;
  final String message;

  @override
  String toString() => message;
}
