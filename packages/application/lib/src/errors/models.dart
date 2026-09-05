/// Stable, localizable operation errors.
///
/// Wire names are snake_case so UI copy, logs, and recovery routing share one
/// identifier. Callers must never invent a new string at an adapter boundary.
enum AppErrorCode {
  accessibilityDenied('accessibility_denied'),
  screenRecordingDenied('screen_recording_denied'),
  captureFailed('capture_failed'),
  captureCancelled('capture_cancelled'),
  ocrNotConfigured('ocr_not_configured'),
  ocrEmpty('ocr_empty'),
  emptySelection('empty_selection'),
  clipboardUnavailable('clipboard_unavailable'),
  clipboardRestoreFailed('clipboard_restore_failed'),
  sourceLanguageDetectionFailed('source_language_detection_failed'),
  unsupportedPair('unsupported_language_pair'),
  languagePackMissing('language_pack_missing'),
  networkFailure('network_failure'),
  proxyConfigurationInvalid('proxy_configuration_invalid'),
  providerAuthFailed('provider_auth_failed'),
  translationFailed('translation_failed'),
  translationIncomplete('translation_incomplete'),
  emptyResult('empty_result'),
  catalogUnavailable('catalog_unavailable'),
  noTranslationService('no_translation_service'),
  glossaryCorrupt('glossary_corrupt'),
  historyUnavailable('history_unavailable'),
  dictionaryUnavailable('dictionary_unavailable'),
  vocabularyUnavailable('vocabulary_unavailable'),
  speechUnavailable('speech_unavailable'),
  speechInterrupted('speech_interrupted'),
  speechFailed('speech_failed'),
  updateCheckFailed('update_check_failed'),
  updateDownloadFailed('update_download_failed'),
  updateChecksumMissing('update_checksum_missing'),
  updateChecksumMismatch('update_checksum_mismatch'),
  updateSignatureInvalid('update_signature_invalid'),
  updateInstallFailed('update_install_failed'),
  protocolInvalid('protocol_invalid'),
  protocolTooLarge('protocol_too_large'),
  apiServerBindFailed('api_server_bind_failed'),
  invalidPort('invalid_port'),
  unknown('unknown');

  const AppErrorCode(this.wireName);

  /// Stable identifier used by localization and recovery routing.
  final String wireName;

  static AppErrorCode parse(String? raw) {
    if (raw == null || raw.trim().isEmpty) return AppErrorCode.unknown;
    final normalized = raw.trim();
    for (final code in AppErrorCode.values) {
      if (code.wireName == normalized) return code;
    }
    return AppErrorCode.unknown;
  }
}

/// What the UI should offer after an [AppFailure].
enum RecoveryAction {
  none,
  retry,
  recheckPermission,
  openPermissionSettings,
  configureOcr,
  configureTranslationProvider,
  editInput,
  chooseLanguage,
  switchToGoogleWeb,
}

/// One recoverable failure. Secret values must never appear in [details].
final class AppFailure implements Exception {
  const AppFailure(
    this.code, {
    this.recovery = RecoveryAction.retry,
    this.details,
  });

  final AppErrorCode code;
  final RecoveryAction recovery;
  final String? details;

  String get wireName => code.wireName;

  @override
  String toString() => 'AppFailure($wireName)';
}

/// Default recovery for a known code.
RecoveryAction recoveryFor(AppErrorCode code) {
  return switch (code) {
    AppErrorCode.accessibilityDenied ||
    AppErrorCode.screenRecordingDenied => RecoveryAction.openPermissionSettings,
    AppErrorCode.captureFailed => RecoveryAction.retry,
    AppErrorCode.captureCancelled => RecoveryAction.none,
    AppErrorCode.ocrNotConfigured => RecoveryAction.configureOcr,
    AppErrorCode.ocrEmpty => RecoveryAction.retry,
    AppErrorCode.emptySelection => RecoveryAction.editInput,
    AppErrorCode.clipboardUnavailable => RecoveryAction.retry,
    AppErrorCode.clipboardRestoreFailed => RecoveryAction.none,
    AppErrorCode.sourceLanguageDetectionFailed => RecoveryAction.chooseLanguage,
    AppErrorCode.unsupportedPair => RecoveryAction.chooseLanguage,
    AppErrorCode.languagePackMissing => RecoveryAction.switchToGoogleWeb,
    AppErrorCode.networkFailure => RecoveryAction.retry,
    AppErrorCode.proxyConfigurationInvalid => RecoveryAction.editInput,
    AppErrorCode.providerAuthFailed =>
      RecoveryAction.configureTranslationProvider,
    AppErrorCode.translationFailed ||
    AppErrorCode.translationIncomplete => RecoveryAction.retry,
    AppErrorCode.emptyResult => RecoveryAction.retry,
    AppErrorCode.catalogUnavailable => RecoveryAction.retry,
    AppErrorCode.noTranslationService =>
      RecoveryAction.configureTranslationProvider,
    AppErrorCode.glossaryCorrupt => RecoveryAction.none,
    AppErrorCode.historyUnavailable => RecoveryAction.retry,
    AppErrorCode.dictionaryUnavailable => RecoveryAction.none,
    AppErrorCode.vocabularyUnavailable => RecoveryAction.retry,
    AppErrorCode.speechUnavailable => RecoveryAction.none,
    AppErrorCode.speechInterrupted => RecoveryAction.retry,
    AppErrorCode.speechFailed => RecoveryAction.retry,
    AppErrorCode.updateCheckFailed => RecoveryAction.retry,
    AppErrorCode.updateDownloadFailed => RecoveryAction.retry,
    AppErrorCode.updateChecksumMissing => RecoveryAction.none,
    AppErrorCode.updateChecksumMismatch => RecoveryAction.none,
    AppErrorCode.updateSignatureInvalid => RecoveryAction.none,
    AppErrorCode.updateInstallFailed => RecoveryAction.retry,
    AppErrorCode.protocolInvalid => RecoveryAction.none,
    AppErrorCode.protocolTooLarge => RecoveryAction.editInput,
    AppErrorCode.apiServerBindFailed => RecoveryAction.retry,
    AppErrorCode.invalidPort => RecoveryAction.editInput,
    AppErrorCode.unknown => RecoveryAction.retry,
  };
}
