import 'dart:io';

import 'package:nativeapi/nativeapi.dart';
import 'package:screen_capturer/screen_capturer.dart';

import '../services/runtime.dart';
import '../services/settings_store.dart';
import 'permission_controller.dart';
import 'platform_types.dart';

class CaptureController {
  CaptureController({PermissionController? permissions})
    : _permissions = permissions ?? permissionController;

  final PermissionController _permissions;

  Future<CaptureResult> captureRegion() async {
    final permission = await _permissions.refresh();
    if (permission.screenRecording == PermissionState.denied) {
      return const CaptureResult(
        failureReason: 'Screen recording permission is required for capture.',
      );
    }

    final cursor = DisplayManager.instance.getCursorPosition();
    final display = _displayAt(cursor);
    final directory = await Directory.systemTemp.createTemp(
      'linguaray-capture-',
    );
    final imagePath = '${directory.path}/region.png';
    try {
      final data = await screenCapturer.capture(
        mode: CaptureMode.region,
        imagePath: imagePath,
        copyToClipboard: false,
        silent: true,
      );
      if (data == null || data.imagePath == null) {
        await directory.delete(recursive: true);
        return CaptureResult(displayId: display?.id, cancelled: true);
      }
      return CaptureResult(
        imagePath: data.imagePath,
        pixelWidth: data.imageWidth,
        pixelHeight: data.imageHeight,
        displayId: display?.id,
      );
    } catch (error) {
      if (await directory.exists()) await directory.delete(recursive: true);
      return CaptureResult(
        displayId: display?.id,
        failureReason: error.toString(),
      );
    }
  }

  Future<String> recognize(
    CaptureResult capture, {
    TriggerAction action = TriggerAction.captureAndTranslate,
  }) async {
    if (!capture.succeeded) {
      throw PlatformOperationException(
        action: action,
        code: capture.cancelled ? 'capture_cancelled' : 'capture_failed',
        message: capture.failureReason ?? 'Screen capture was cancelled.',
      );
    }

    try {
      final configured = settingsStore.defaultOcrService.trim();
      String? fallback;
      for (final service in settingsStore.services) {
        if (service.type == ServiceType.ocr) {
          fallback = service.id;
          break;
        }
      }
      final serviceId = configured.isNotEmpty ? configured : fallback;
      if (serviceId == null || serviceId.isEmpty) {
        throw PlatformOperationException(
          action: action,
          code: 'ocr_not_configured',
          message: 'No OCR service is configured.',
        );
      }

      final response = await runtime
          .ocr(providerId: serviceId)
          .recognizeText(
            request: RecognizeTextRequest(imagePath: capture.imagePath),
          );
      if (response.text.trim().isEmpty) {
        throw PlatformOperationException(
          action: action,
          code: 'ocr_empty',
          message: 'No text was found in the selected region.',
        );
      }
      return response.text;
    } finally {
      final imagePath = capture.imagePath;
      if (imagePath != null) {
        final directory = File(imagePath).parent;
        if (directory.path.contains('linguaray-capture-') &&
            await directory.exists()) {
          await directory.delete(recursive: true);
        }
      }
    }
  }

  Display? _displayAt(Offset position) {
    final displays = DisplayManager.instance.getAll();
    for (final display in displays) {
      final bounds = Rect.fromLTWH(
        display.position.dx,
        display.position.dy,
        display.size.width,
        display.size.height,
      );
      if (bounds.contains(position)) return display;
    }
    return displays.isEmpty ? null : displays.first;
  }
}

final captureController = CaptureController();
