import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:nativeapi/nativeapi.dart';

import '../services/app_windows.dart';
import '../services/settings_store.dart';
import 'capture_controller.dart';
import 'permission_controller.dart';
import 'platform_types.dart';
import 'selection_controller.dart';

class QuickWindowRequest {
  const QuickWindowRequest({
    this.text,
    this.submit = false,
    this.clearExisting = false,
  });

  final String? text;
  final bool submit;
  final bool clearExisting;
}

class TriggerController {
  TriggerController({
    SelectionController? selection,
    CaptureController? capture,
    PermissionController? permissions,
  }) : _selection = selection ?? selectionController,
       _capture = capture ?? captureController,
       _permissions = permissions ?? permissionController;

  final SelectionController _selection;
  final CaptureController _capture;
  final PermissionController _permissions;

  final ValueNotifier<QuickWindowRequest?> quickWindowRequest = ValueNotifier(
    null,
  );
  final ValueNotifier<PlatformOperationException?> lastError = ValueNotifier(
    null,
  );

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
          await _captureAndOpen(autoSubmit: true);
          break;
        case TriggerAction.captureOcr:
          await _captureAndOpen(autoSubmit: false);
          break;
      }
    } on PlatformOperationException catch (error) {
      lastError.value = error;
      final captureAction =
          action == TriggerAction.captureAndTranslate ||
          action == TriggerAction.captureOcr;
      if (!captureAction ||
          error.code != 'capture_cancelled' && error.code != 'cancelled') {
        await showMiniTranslatorWindow(
          position: miniTranslatorPositionNearCursor(),
        );
      }
    } catch (error) {
      lastError.value = PlatformOperationException(
        action: action,
        code: 'operationFailed',
        message: error.toString(),
      );
      await showMiniTranslatorWindow(
        position: miniTranslatorPositionNearCursor(),
      );
    } finally {
      await _permissions.refresh();
    }
  }

  Future<void> _captureAndOpen({required bool autoSubmit}) async {
    final position = DisplayManager.instance.getCursorPosition();
    final wasSettingsVisible = isSettingsWindowOpen;
    final wasMiniVisible = isMiniTranslatorWindowVisible;
    hideMiniTranslatorWindow();
    hideSettingsWindow();
    final capture = await _capture.captureRegion();
    if (capture.cancelled) {
      if (wasSettingsVisible) {
        focusSettingsWindow();
      } else if (wasMiniVisible) {
        await showMiniTranslatorWindow(position: position);
      }
      return;
    }
    final action = autoSubmit
        ? TriggerAction.captureAndTranslate
        : TriggerAction.captureOcr;
    final text = await _capture.recognize(capture, action: action);
    quickWindowRequest.value = QuickWindowRequest(
      text: text,
      submit: autoSubmit,
      clearExisting: true,
    );
    if (!autoSubmit && settingsStore.autoCopyDetectedText) {
      try {
        await Clipboard.setData(ClipboardData(text: text));
      } catch (_) {
        throw PlatformOperationException(
          action: action,
          code: 'clipboard_unavailable',
          message: 'Recognized text could not be copied to the clipboard.',
        );
      }
    }
    await showMiniTranslatorWindow(
      position: miniTranslatorPositionNearPoint(position),
    );
  }
}

final triggerController = TriggerController();
