// ignore_for_file: prefer_initializing_formals

import 'dart:io';

import 'package:file_selector/file_selector.dart';
import 'package:nativeapi/nativeapi.dart';
import 'package:screen_capturer/screen_capturer.dart';

import '../../app/runtime.dart';
import '../../app/settings/settings_store.dart';
import '../permissions/permission_controller.dart';
import '../platform_types.dart';

class CaptureController {
  CaptureController({
    required PermissionController permissions,
    required SettingsStore store,
  }) : _permissions = permissions,
       _store = store;

  final PermissionController _permissions;
  final SettingsStore _store;

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

  Future<OcrRecognitionResult> recognize(
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
      final response = await runtime
          .ocr(providerId: _ocrServiceId(action))
          .recognizeText(
            request: RecognizeTextRequest(imagePath: capture.imagePath),
          );
      return _result(
        response,
        action: action,
        source: OcrInputSource.screenRegion,
      );
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

  Future<OcrRecognitionResult> recognizeImageFile({
    required TriggerAction action,
  }) async {
    final file = await openFile(
      acceptedTypeGroups: const [
        XTypeGroup(
          label: 'Images',
          extensions: [
            'png',
            'jpg',
            'jpeg',
            'webp',
            'bmp',
            'gif',
            'tif',
            'tiff',
            'heic',
          ],
        ),
      ],
    );
    if (file == null) {
      throw PlatformOperationException(
        action: action,
        code: 'file_cancelled',
        message: 'Image selection was cancelled.',
      );
    }
    final response = await runtime
        .ocr(providerId: _ocrServiceId(action))
        .recognizeText(request: RecognizeTextRequest(imagePath: file.path));
    return _result(
      response,
      action: action,
      source: OcrInputSource.file,
      imagePath: file.path,
    );
  }

  Future<OcrRecognitionResult> recognizeClipboardImage({
    required TriggerAction action,
  }) async {
    try {
      final response = await runtime
          .ocr(providerId: _ocrServiceId(action))
          .recognizeClipboardImage();
      return _result(
        response,
        action: action,
        source: OcrInputSource.clipboard,
      );
    } on PlatformOperationException {
      rethrow;
    } catch (error) {
      throw PlatformOperationException(
        action: action,
        code: 'clipboard_unavailable',
        message: error.toString(),
      );
    }
  }

  String _ocrServiceId(TriggerAction action) {
    final configured = _store.defaultOcrService.trim();
    String? fallback;
    for (final service in _store.services) {
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
    return serviceId;
  }

  OcrRecognitionResult _result(
    RecognizeTextResponse response, {
    required TriggerAction action,
    required OcrInputSource source,
    String? imagePath,
  }) {
    final text = response.text.trim();
    if (text.isEmpty) {
      throw PlatformOperationException(
        action: action,
        code: 'ocr_empty',
        message: 'No text was found in the image.',
      );
    }
    return OcrRecognitionResult(
      text: text,
      source: source,
      imagePath: imagePath,
      blocks: [
        for (final recognition in response.recognitions ?? const [])
          OcrTextBlock(
            text: recognition.text,
            bounds: recognition.recognizedRect == null
                ? null
                : Rect.fromLTWH(
                    recognition.recognizedRect!.x,
                    recognition.recognizedRect!.y,
                    recognition.recognizedRect!.width,
                    recognition.recognizedRect!.height,
                  ),
          ),
      ],
    );
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

final captureController = CaptureController(
  permissions: permissionController,
  store: settingsStore,
);
