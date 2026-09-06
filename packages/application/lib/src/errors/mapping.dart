import 'package:linguaray_application/src/errors/models.dart';

/// Maps adapter, platform, and runtime strings onto [AppErrorCode].
///
/// Accepts both the historical camelCase identifiers and the canonical
/// snake_case wire names so older call sites can be normalized in one place.
AppErrorCode mapErrorCode(String? raw) {
  if (raw == null) return AppErrorCode.unknown;
  final value = raw.trim();
  if (value.isEmpty) return AppErrorCode.unknown;

  final known = AppErrorCode.parse(value);
  if (known != AppErrorCode.unknown) return known;

  return switch (value) {
    'accessibilityDenied' ||
    'accessibility_denied' ||
    'permission_denied' => AppErrorCode.accessibilityDenied,
    'screenRecordingDenied' ||
    'screen_recording_denied' => AppErrorCode.screenRecordingDenied,
    'captureFailed' || 'capture_failed' => AppErrorCode.captureFailed,
    'cancelled' ||
    'canceled' ||
    'captureCancelled' ||
    'capture_cancelled' => AppErrorCode.captureCancelled,
    'ocrNotConfigured' || 'ocr_not_configured' => AppErrorCode.ocrNotConfigured,
    'ocrEmpty' || 'ocr_empty' => AppErrorCode.ocrEmpty,
    'emptySelection' ||
    'empty_selection' ||
    'no_selection' => AppErrorCode.emptySelection,
    'clipboardUnavailable' ||
    'clipboard_unavailable' => AppErrorCode.clipboardUnavailable,
    'clipboardRestoreFailed' ||
    'clipboard_restore_failed' => AppErrorCode.clipboardRestoreFailed,
    'source_language_detection_failed' || 'sourceLanguageDetectionFailed' =>
      AppErrorCode.sourceLanguageDetectionFailed,
    'unsupported_language_pair' ||
    'unsupportedPair' ||
    'unsupported_pair' => AppErrorCode.unsupportedPair,
    'language_pair_not_installed' ||
    'languagePackMissing' ||
    'language_pack_missing' => AppErrorCode.languagePackMissing,
    'network_error' ||
    'networkFailure' ||
    'network_failure' ||
    'offline' => AppErrorCode.networkFailure,
    'provider_auth_failed' ||
    'unauthorized' ||
    '401' ||
    'authentication_failed' => AppErrorCode.providerAuthFailed,
    'empty_result' => AppErrorCode.emptyResult,
    'catalog_unavailable' => AppErrorCode.catalogUnavailable,
    'no_translation_service' ||
    'no_provider' => AppErrorCode.noTranslationService,
    'service_unavailable' ||
    'translation_failed' => AppErrorCode.translationFailed,
    _ => _inferFromMessage(value.toLowerCase()),
  };
}

AppErrorCode _inferFromMessage(String message) {
  if (message.contains('accessibility')) {
    return AppErrorCode.accessibilityDenied;
  }
  if (message.contains('screen recording') ||
      message.contains('screen-recording')) {
    return AppErrorCode.screenRecordingDenied;
  }
  if (message.contains('not installed') || message.contains('language pack')) {
    return AppErrorCode.languagePackMissing;
  }
  if (message.contains('unsupported language') ||
      message.contains('unsupported pair')) {
    return AppErrorCode.unsupportedPair;
  }
  if (message.contains('unable to detect') ||
      message.contains('language detection failed')) {
    return AppErrorCode.sourceLanguageDetectionFailed;
  }
  if (message.contains('401') ||
      message.contains('unauthorized') ||
      message.contains('invalid api') ||
      message.contains('authentication')) {
    return AppErrorCode.providerAuthFailed;
  }
  if (message.contains('network') ||
      message.contains('connection') ||
      message.contains('timed out') ||
      message.contains('timeout')) {
    return AppErrorCode.networkFailure;
  }
  if (message.contains('bind')) return AppErrorCode.apiServerBindFailed;
  if (message.contains('ocr') && message.contains('config')) {
    return AppErrorCode.ocrNotConfigured;
  }
  return AppErrorCode.unknown;
}
