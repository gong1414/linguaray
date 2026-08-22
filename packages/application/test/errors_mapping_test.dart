import 'package:linguaray_application/linguaray_application.dart';
import 'package:test/test.dart';

void main() {
  test('normalizes historical camelCase identifiers', () {
    expect(
      mapErrorCode('accessibilityDenied'),
      AppErrorCode.accessibilityDenied,
    );
    expect(mapErrorCode('captureFailed'), AppErrorCode.captureFailed);
    expect(mapErrorCode('ocrNotConfigured'), AppErrorCode.ocrNotConfigured);
    expect(mapErrorCode('ocrEmpty'), AppErrorCode.ocrEmpty);
    expect(mapErrorCode('cancelled'), AppErrorCode.captureCancelled);
  });

  test('keeps canonical snake_case identifiers', () {
    expect(
      mapErrorCode('accessibility_denied'),
      AppErrorCode.accessibilityDenied,
    );
    expect(mapErrorCode('ocr_not_configured'), AppErrorCode.ocrNotConfigured);
    expect(
      mapErrorCode('source_language_detection_failed'),
      AppErrorCode.sourceLanguageDetectionFailed,
    );
  });

  test('maps recovery actions for permission and OCR failures', () {
    expect(
      recoveryFor(AppErrorCode.accessibilityDenied),
      RecoveryAction.openPermissionSettings,
    );
    expect(
      recoveryFor(AppErrorCode.ocrNotConfigured),
      RecoveryAction.configureOcr,
    );
    expect(
      recoveryFor(AppErrorCode.noTranslationService),
      RecoveryAction.configureTranslationProvider,
    );
    expect(
      recoveryFor(AppErrorCode.sourceLanguageDetectionFailed),
      RecoveryAction.chooseLanguage,
    );
    expect(recoveryFor(AppErrorCode.captureCancelled), RecoveryAction.none);
    expect(
      recoveryFor(AppErrorCode.languagePackMissing),
      RecoveryAction.switchToGoogleWeb,
    );
  });

  test('infers network and auth failures from messages', () {
    expect(mapErrorCode('connection timed out'), AppErrorCode.networkFailure);
    expect(mapErrorCode('401 unauthorized'), AppErrorCode.providerAuthFailed);
    expect(
      mapErrorCode('language pack is not installed'),
      AppErrorCode.languagePackMissing,
    );
  });
}
