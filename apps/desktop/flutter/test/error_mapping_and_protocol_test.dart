import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/platform/protocol_controller.dart';

void main() {
  test('legacy and canonical error identifiers share recovery', () {
    expect(
      mapErrorCode('accessibilityDenied'),
      AppErrorCode.accessibilityDenied,
    );
    expect(mapErrorCode('ocrNotConfigured'), AppErrorCode.ocrNotConfigured);
    expect(mapErrorCode('ocrEmpty'), AppErrorCode.ocrEmpty);
    expect(mapErrorCode('captureFailed'), AppErrorCode.captureFailed);
    expect(
      recoveryFor(AppErrorCode.ocrNotConfigured),
      RecoveryAction.configureOcr,
    );
  });

  test('windows capabilities hide system translation', () {
    const windows = PlatformCapabilities.windows();
    expect(windows.systemOcr, isTrue);
    expect(windows.systemTranslation, isFalse);
    expect(windows.systemDictionary, isFalse);
    expect(windows.systemLanguageDetection, isFalse);
  });

  test('protocol parser routes translate and ignores unknown actions', () {
    const parse = ParseProtocolLink();
    expect(parse('linguaray://translate?text=hello').text, 'hello');
    expect(parse('linguaray://settings').action, ProtocolAction.settings);
    expect(parse('linguaray://replace?text=no').action, ProtocolAction.ignored);
  });

  test('settings protocol invokes its configured route exactly once', () async {
    final controller = ProtocolController();
    var openCount = 0;
    controller.onOpenSettings = () => openCount++;

    await controller.handleRaw('linguaray://settings');

    expect(openCount, 1);
  });
}
