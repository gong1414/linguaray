import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/src/features/ocr/ocr_controller.dart';
import 'package:linguaray_desktop/src/platform/capture/capture_controller.dart';
import 'package:linguaray_desktop/src/platform/platform_types.dart';

void main() {
  test(
    'continuous OCR appends results while normal OCR replaces them',
    () async {
      final capture = _FakeCaptureController();
      final controller = OcrController(
        capture: capture,
        autoCopy: () => false,
        writeClipboard: (_) async {},
      );

      await controller.recognizeClipboard();
      expect(controller.state.text, 'first');

      controller.setContinuous(true);
      await controller.recognizeClipboard();
      expect(controller.state.text, 'first\n\nsecond');
      expect(controller.state.results, hasLength(2));

      controller.setContinuous(false);
      await controller.recognizeClipboard();
      expect(controller.state.text, 'third');
      expect(controller.state.results, hasLength(1));
    },
  );

  test('silent OCR forces a clipboard write', () async {
    final copied = <String>[];
    final controller = OcrController(
      capture: _FakeCaptureController(),
      autoCopy: () => false,
      writeClipboard: (text) async => copied.add(text),
    );

    await controller.recognizeCapture(
      const CaptureResult(imagePath: '/tmp/region.png'),
      action: TriggerAction.silentCaptureOcr,
      forceCopy: true,
    );

    expect(copied, ['first']);
  });
}

final class _FakeCaptureController extends CaptureController {
  int _next = 0;

  OcrRecognitionResult _result(OcrInputSource source) {
    _next++;
    return OcrRecognitionResult(text: _word(_next), source: source);
  }

  @override
  Future<OcrRecognitionResult> recognize(
    CaptureResult capture, {
    TriggerAction action = TriggerAction.captureAndTranslate,
  }) async => _result(OcrInputSource.screenRegion);

  @override
  Future<OcrRecognitionResult> recognizeClipboardImage({
    required TriggerAction action,
  }) async => _result(OcrInputSource.clipboard);

  String _word(int value) => switch (value) {
    1 => 'first',
    2 => 'second',
    _ => 'third',
  };
}
