import 'package:flutter/foundation.dart';
import 'package:nativeapi/nativeapi.dart';

import '../../features/ocr/ocr_controller.dart';
import '../../platform/capture/capture_controller.dart';
import '../../platform/permissions/permission_controller.dart';
import '../../platform/platform_types.dart';
import '../../platform/selection/selection_controller.dart';
import '../../platform/selection/selection_replacement_controller.dart';
import '../windows/app_windows.dart';

class QuickWindowRequest {
  const QuickWindowRequest({
    this.text,
    this.replacementTarget,
    this.submit = false,
    this.clearExisting = false,
  });

  final String? text;
  final SelectionTarget? replacementTarget;
  final bool submit;
  final bool clearExisting;
}

class TriggerController {
  TriggerController({
    SelectionController? selection,
    CaptureController? capture,
    OcrController? ocr,
    PermissionController? permissions,
  }) : _selection = selection ?? selectionController,
       _capture = capture ?? captureController,
       _ocr = ocr ?? OcrController(capture: capture ?? captureController),
       _permissions = permissions ?? permissionController;

  final SelectionController _selection;
  final CaptureController _capture;
  final OcrController _ocr;
  final PermissionController _permissions;

  final ValueNotifier<QuickWindowRequest?> quickWindowRequest = ValueNotifier(
    null,
  );
  final ValueNotifier<PlatformOperationException?> lastError = ValueNotifier(
    null,
  );

  Future<void> translateText(String text) {
    quickWindowRequest.value = QuickWindowRequest(
      text: text,
      submit: true,
      clearExisting: true,
    );
    return showMiniTranslatorWindow();
  }

  Future<void> openInputWindow({Rect? trayBounds}) async {
    quickWindowRequest.value = const QuickWindowRequest(clearExisting: true);
    await showMiniTranslatorWindow(trayBounds: trayBounds);
  }

  Future<void> showTranslationWindow({Rect? trayBounds}) {
    return showMiniTranslatorWindow(trayBounds: trayBounds);
  }

  Future<void> trigger(TriggerAction action) async {
    lastError.value = null;
    try {
      switch (action) {
        case TriggerAction.toggleQuickWindow:
          if (isMiniTranslatorWindowVisible) {
            hideMiniTranslatorWindow();
          } else {
            await showMiniTranslatorWindow(
              position: miniTranslatorPositionNearCursor(),
            );
          }
          break;
        case TriggerAction.translateSelection:
          final result = await _selection.readSelection();
          if (result.text.trim().isEmpty) {
            throw const PlatformOperationException(
              action: TriggerAction.translateSelection,
              code: 'empty_selection',
              message: 'No text is selected.',
            );
          }
          if (result.recoverableError != null &&
              result.recoverableError!.isNotEmpty) {
            lastError.value = const PlatformOperationException(
              action: TriggerAction.translateSelection,
              code: 'clipboard_restore_failed',
              message: 'Clipboard restoration failed.',
            );
          }
          quickWindowRequest.value = QuickWindowRequest(
            text: result.text,
            replacementTarget: result.replacementTarget,
            submit: true,
            clearExisting: true,
          );
          await showMiniTranslatorWindow(
            position: miniTranslatorPositionNearPoint(result.triggerPosition),
          );
          break;
        case TriggerAction.openInputWindow:
          await openInputWindow();
          break;
        case TriggerAction.translateInput:
          final position = DisplayManager.instance.getCursorPosition();
          final text = await _selection.readClipboard();
          if (text == null || text.trim().isEmpty) {
            throw const PlatformOperationException(
              action: TriggerAction.translateInput,
              code: 'clipboard_unavailable',
              message: 'The clipboard could not be read.',
            );
          }
          quickWindowRequest.value = QuickWindowRequest(
            text: text,
            submit: true,
            clearExisting: true,
          );
          await showMiniTranslatorWindow(
            position: miniTranslatorPositionNearPoint(position),
          );
          break;
        case TriggerAction.captureAndTranslate:
          await _captureAndTranslate();
          break;
        case TriggerAction.captureOcr:
          await _captureOcr(silent: false);
          break;
        case TriggerAction.silentCaptureOcr:
          await _captureOcr(silent: true);
          break;
        case TriggerAction.fileOcr:
          await _fileOcr();
          break;
        case TriggerAction.clipboardOcr:
          await _ocr.recognizeClipboard();
          await showOcrWindow(position: ocrWindowPositionNearCursor());
          break;
        case TriggerAction.showOcrWindow:
          await showOcrWindow(position: ocrWindowPositionNearCursor());
          break;
      }
    } on PlatformOperationException catch (error) {
      lastError.value = error;
      final dismissibleAction =
          action == TriggerAction.captureAndTranslate ||
          action == TriggerAction.captureOcr ||
          action == TriggerAction.silentCaptureOcr ||
          action == TriggerAction.fileOcr;
      final cancelled =
          error.code == 'capture_cancelled' ||
          error.code == 'cancelled' ||
          error.code == 'file_cancelled';
      if (!dismissibleAction || !cancelled) {
        if (_isOcrAction(action)) {
          await showOcrWindow(position: ocrWindowPositionNearCursor());
        } else {
          await showMiniTranslatorWindow(
            position: miniTranslatorPositionNearCursor(),
          );
        }
      }
    } catch (error) {
      lastError.value = PlatformOperationException(
        action: action,
        code: 'operationFailed',
        message: error.toString(),
      );
      if (_isOcrAction(action)) {
        await showOcrWindow(position: ocrWindowPositionNearCursor());
      } else {
        await showMiniTranslatorWindow(
          position: miniTranslatorPositionNearCursor(),
        );
      }
    } finally {
      await _permissions.refresh();
    }
  }

  Future<void> _captureAndTranslate() async {
    final position = DisplayManager.instance.getCursorPosition();
    final wasSettingsVisible = isSettingsWindowOpen;
    final wasMiniVisible = isMiniTranslatorWindowVisible;
    final wasOcrVisible = isOcrWindowVisible;
    hideMiniTranslatorWindow();
    hideSettingsWindow();
    hideOcrWindow();
    final capture = await _capture.captureRegion();
    if (capture.cancelled) {
      await _restoreSurface(
        position: position,
        settings: wasSettingsVisible,
        mini: wasMiniVisible,
        ocr: wasOcrVisible,
      );
      return;
    }
    final result = await _capture.recognize(
      capture,
      action: TriggerAction.captureAndTranslate,
    );
    quickWindowRequest.value = QuickWindowRequest(
      text: result.text,
      submit: true,
      clearExisting: true,
    );
    await showMiniTranslatorWindow(
      position: miniTranslatorPositionNearPoint(position),
    );
  }

  Future<void> _captureOcr({required bool silent}) async {
    final position = DisplayManager.instance.getCursorPosition();
    final wasSettingsVisible = isSettingsWindowOpen;
    final wasMiniVisible = isMiniTranslatorWindowVisible;
    final wasOcrVisible = isOcrWindowVisible;
    hideMiniTranslatorWindow();
    hideSettingsWindow();
    hideOcrWindow();
    final capture = await _ocr.captureRegion();
    if (capture.cancelled) {
      await _restoreSurface(
        position: position,
        settings: wasSettingsVisible,
        mini: wasMiniVisible,
        ocr: wasOcrVisible,
      );
      return;
    }
    await _ocr.recognizeCapture(
      capture,
      action: silent
          ? TriggerAction.silentCaptureOcr
          : TriggerAction.captureOcr,
      forceCopy: silent,
    );
    if (silent) {
      await _restoreSurface(
        position: position,
        settings: wasSettingsVisible,
        mini: wasMiniVisible,
        ocr: wasOcrVisible,
      );
      return;
    }
    await showOcrWindow(position: ocrWindowPositionNearPoint(position));
  }

  Future<void> _fileOcr() async {
    final wasOcrVisible = isOcrWindowVisible;
    try {
      await _ocr.recognizeFile();
      await showOcrWindow(position: ocrWindowPositionNearCursor());
    } on PlatformOperationException catch (error) {
      if (error.code == 'file_cancelled' && wasOcrVisible) {
        await showOcrWindow(position: ocrWindowPositionNearCursor());
      }
      rethrow;
    }
  }

  Future<void> _restoreSurface({
    required Offset position,
    required bool settings,
    required bool mini,
    required bool ocr,
  }) async {
    if (settings) {
      focusSettingsWindow();
    } else if (mini) {
      await showMiniTranslatorWindow(
        position: miniTranslatorPositionNearPoint(position),
      );
    } else if (ocr) {
      await showOcrWindow(position: ocrWindowPositionNearPoint(position));
    }
  }

  bool _isOcrAction(TriggerAction action) => switch (action) {
    TriggerAction.captureOcr ||
    TriggerAction.silentCaptureOcr ||
    TriggerAction.fileOcr ||
    TriggerAction.clipboardOcr ||
    TriggerAction.showOcrWindow => true,
    _ => false,
  };
}

final triggerController = TriggerController(
  ocr: ocrController,
  capture: captureController,
);
