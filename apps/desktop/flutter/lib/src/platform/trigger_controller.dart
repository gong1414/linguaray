import 'package:flutter/foundation.dart';
import 'package:nativeapi/nativeapi.dart';

import '../services/app_windows.dart';
import 'capture_controller.dart';
import 'permission_controller.dart';
import 'platform_types.dart';
import 'selection_controller.dart';

class TriggerController {
  TriggerController({
    SelectionController? selection,
    CaptureController? capture,
    PermissionController? permissions,
  })  : _selection = selection ?? selectionController,
        _capture = capture ?? captureController,
        _permissions = permissions ?? permissionController;

  final SelectionController _selection;
  final CaptureController _capture;
  final PermissionController _permissions;

  final ValueNotifier<String?> quickWindowText = ValueNotifier(null);
  final ValueNotifier<PlatformOperationException?> lastError =
      ValueNotifier(null);

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
          quickWindowText.value = result.text;
          await showMiniTranslatorWindow(
            position: miniTranslatorPositionNearPoint(result.triggerPosition),
          );
          break;
        case TriggerAction.translateInput:
          final position = DisplayManager.instance.getCursorPosition();
          final text = await _selection.readClipboard();
          if (text?.trim().isNotEmpty == true) quickWindowText.value = text;
          await showMiniTranslatorWindow(
            position: miniTranslatorPositionNearPoint(position),
          );
          break;
        case TriggerAction.captureAndTranslate:
          final position = DisplayManager.instance.getCursorPosition();
          final wasWorkbenchVisible = isWorkbenchWindowOpen;
          final wasMiniVisible = isMiniTranslatorWindowVisible;
          hideMiniTranslatorWindow();
          hideWorkbenchWindow();
          final capture = await _capture.captureRegion();
          if (capture.cancelled) {
            if (wasWorkbenchVisible) {
              focusWorkbenchWindow();
            } else if (wasMiniVisible) {
              await showMiniTranslatorWindow(position: position);
            }
            return;
          }
          final text = await _capture.recognize(capture);
          quickWindowText.value = text;
          await showMiniTranslatorWindow(
            position: miniTranslatorPositionNearPoint(position),
          );
          break;
      }
    } on PlatformOperationException catch (error) {
      lastError.value = error;
      if (action != TriggerAction.captureAndTranslate ||
          error.code != 'cancelled') {
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
}

final triggerController = TriggerController();
