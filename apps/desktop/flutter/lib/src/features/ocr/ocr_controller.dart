// ignore_for_file: prefer_initializing_formals

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

import '../../app/settings/settings_store.dart';
import '../../platform/capture/capture_controller.dart';
import '../../platform/platform_types.dart';

typedef OcrClipboardWriter = Future<void> Function(String text);

final class OcrViewState {
  const OcrViewState({
    this.results = const [],
    this.text = '',
    this.busy = false,
    this.continuous = false,
    this.errorCode,
  });

  final List<OcrRecognitionResult> results;
  final String text;
  final bool busy;
  final bool continuous;
  final String? errorCode;

  OcrViewState copyWith({
    List<OcrRecognitionResult>? results,
    String? text,
    bool? busy,
    bool? continuous,
    String? errorCode,
    bool clearError = false,
  }) {
    return OcrViewState(
      results: results ?? this.results,
      text: text ?? this.text,
      busy: busy ?? this.busy,
      continuous: continuous ?? this.continuous,
      errorCode: clearError ? null : errorCode ?? this.errorCode,
    );
  }
}

/// UI-independent OCR workflow shared by tray actions, shortcuts, and the OCR
/// surface. Platform capture and runtime OCR stay behind typed controllers.
class OcrController extends ChangeNotifier {
  OcrController({
    required CaptureController capture,
    required bool Function() autoCopy,
    OcrClipboardWriter? writeClipboard,
  }) : _capture = capture,
       _autoCopy = autoCopy,
       _writeClipboard =
           writeClipboard ??
           ((text) => Clipboard.setData(ClipboardData(text: text)));

  final CaptureController _capture;
  final bool Function() _autoCopy;
  final OcrClipboardWriter _writeClipboard;
  OcrViewState _state = const OcrViewState();

  OcrViewState get state => _state;

  void setContinuous(bool value) {
    _state = _state.copyWith(continuous: value);
    notifyListeners();
  }

  void clear() {
    _state = OcrViewState(continuous: _state.continuous);
    notifyListeners();
  }

  void setText(String value) {
    _state = _state.copyWith(text: value);
    notifyListeners();
  }

  Future<CaptureResult> captureRegion() => _capture.captureRegion();

  Future<OcrRecognitionResult> recognizeCapture(
    CaptureResult capture, {
    required TriggerAction action,
    bool forceCopy = false,
  }) {
    return _run(
      () => _capture.recognize(capture, action: action),
      forceCopy: forceCopy,
    );
  }

  Future<OcrRecognitionResult> recognizeFile({
    TriggerAction action = TriggerAction.fileOcr,
  }) {
    return _run(() => _capture.recognizeImageFile(action: action));
  }

  Future<OcrRecognitionResult> recognizeClipboard({
    TriggerAction action = TriggerAction.clipboardOcr,
  }) {
    return _run(() => _capture.recognizeClipboardImage(action: action));
  }

  Future<void> copy() async {
    final text = _state.text.trim();
    if (text.isEmpty) return;
    await _writeClipboard(text);
  }

  Future<OcrRecognitionResult> _run(
    Future<OcrRecognitionResult> Function() operation, {
    bool forceCopy = false,
  }) async {
    _state = _state.copyWith(busy: true, clearError: true);
    notifyListeners();
    try {
      final result = await operation();
      final results = _state.continuous
          ? [..._state.results, result]
          : [result];
      final text = _state.continuous && _state.text.trim().isNotEmpty
          ? '${_state.text.trimRight()}\n\n${result.text}'
          : result.text;
      _state = _state.copyWith(
        results: results,
        text: text,
        busy: false,
        clearError: true,
      );
      notifyListeners();
      if (forceCopy || _autoCopy()) await copy();
      return result;
    } on PlatformOperationException catch (error) {
      _state = _state.copyWith(busy: false, errorCode: error.code);
      notifyListeners();
      rethrow;
    } catch (_) {
      _state = _state.copyWith(busy: false, errorCode: 'operationFailed');
      notifyListeners();
      rethrow;
    }
  }
}

final ocrController = OcrController(
  capture: captureController,
  autoCopy: () => settingsStore.autoCopyDetectedText,
);
