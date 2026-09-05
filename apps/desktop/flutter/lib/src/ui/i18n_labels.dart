import 'package:linguaray_application/linguaray_application.dart';

import '../i18n/i18n.dart';
import 'quick_translate/widgets/quick_translate_view.dart';
import 'settings/settings_labels.dart';

String translationFailureMessage(String? code) {
  final translation = t.workbench.translation;
  final errors = t.ui.errors;
  return switch (mapErrorCode(code)) {
    AppErrorCode.languagePackMissing => translation.language_pair_not_installed,
    AppErrorCode.unsupportedPair => translation.unsupported_language_pair,
    AppErrorCode.sourceLanguageDetectionFailed =>
      translation.source_language_detection_failed,
    AppErrorCode.networkFailure => translation.network_error,
    AppErrorCode.providerAuthFailed => errors.provider_auth_failed,
    AppErrorCode.translationIncomplete => errors.translation_incomplete,
    AppErrorCode.noTranslationService => errors.no_translation_service,
    AppErrorCode.emptyResult => errors.empty_result,
    _ => translation.failed,
  };
}

/// Converts every stable operation error code into user-facing localized copy.
/// Raw wire identifiers must never escape into a screen or dialog.
String appErrorMessage(String? code) {
  final errors = t.ui.errors;
  return switch (mapErrorCode(code)) {
    AppErrorCode.accessibilityDenied => errors.accessibility_denied,
    AppErrorCode.screenRecordingDenied => errors.screen_recording_denied,
    AppErrorCode.captureFailed => errors.capture_failed,
    AppErrorCode.captureCancelled => errors.capture_cancelled,
    AppErrorCode.ocrNotConfigured => errors.ocr_not_configured,
    AppErrorCode.ocrEmpty => errors.ocr_empty,
    AppErrorCode.emptySelection => errors.empty_selection,
    AppErrorCode.clipboardUnavailable => errors.clipboard_unavailable,
    AppErrorCode.clipboardRestoreFailed => errors.clipboard_restore_failed,
    AppErrorCode.sourceLanguageDetectionFailed =>
      errors.source_language_detection_failed,
    AppErrorCode.unsupportedPair => errors.unsupported_language_pair,
    AppErrorCode.languagePackMissing => errors.language_pack_missing,
    AppErrorCode.networkFailure => errors.network_failure,
    AppErrorCode.providerAuthFailed => errors.provider_auth_failed,
    AppErrorCode.translationIncomplete => errors.translation_incomplete,
    AppErrorCode.translationFailed => errors.translation_failed,
    AppErrorCode.emptyResult => errors.empty_result,
    AppErrorCode.catalogUnavailable => errors.catalog_unavailable,
    AppErrorCode.noTranslationService => errors.no_translation_service,
    AppErrorCode.glossaryCorrupt => errors.glossary_corrupt,
    AppErrorCode.historyUnavailable => errors.history_unavailable,
    AppErrorCode.dictionaryUnavailable => errors.dictionary_unavailable,
    AppErrorCode.vocabularyUnavailable => errors.vocabulary_unavailable,
    AppErrorCode.speechUnavailable => errors.speech_unavailable,
    AppErrorCode.speechInterrupted => errors.speech_interrupted,
    AppErrorCode.speechFailed => errors.speech_failed,
    AppErrorCode.updateCheckFailed => errors.update_check_failed,
    AppErrorCode.updateDownloadFailed => errors.update_download_failed,
    AppErrorCode.updateChecksumMissing => errors.update_checksum_missing,
    AppErrorCode.updateChecksumMismatch => errors.update_checksum_mismatch,
    AppErrorCode.updateSignatureInvalid => errors.update_signature_invalid,
    AppErrorCode.updateInstallFailed => errors.update_install_failed,
    AppErrorCode.protocolInvalid => errors.protocol_invalid,
    AppErrorCode.protocolTooLarge => errors.protocol_too_large,
    AppErrorCode.apiServerBindFailed => errors.api_server_bind_failed,
    AppErrorCode.invalidPort => errors.invalid_port,
    AppErrorCode.proxyConfigurationInvalid =>
      errors.proxy_configuration_invalid,
    AppErrorCode.unknown => errors.unknown,
  };
}

QuickTranslateLabels quickTranslateLabels() {
  return QuickTranslateLabels(
    title: t.ui.quick.title,
    close: t.ui.quick.close,
    collapseSource: t.ui.quick.collapse_source,
    showSource: t.ui.quick.show_source,
    expandReading: t.ui.quick.expand_reading,
    compactReading: t.ui.quick.compact_reading,
    fontLarger: t.ui.quick.font_larger,
    fontSmaller: t.ui.quick.font_smaller,
    fontReset: t.ui.quick.font_reset,
    stop: t.ui.quick.stop,
    stopped: t.ui.quick.stopped,
    replace: t.ui.quick.replace,
    replaceChanged: t.ui.quick.replace_changed,
    replaceUnsupported: t.ui.quick.replace_unsupported,
    serviceHint: t.ui.quick.service_hint,

    sourceLabel: t.ui.quick.source_label,
    resultLabel: t.ui.quick.result_label,
    resultPlaceholder: t.ui.quick.result_placeholder,
    inputHint: t.ui.quick.input_hint,
    translate: t.mini_translator.button.translate,
    clear: t.mini_translator.button.clear,
    copy: t.mini_translator.button.copy,
    copied: t.mini_translator.button.copied,
    pin: t.ui.quick.pin,
    unpin: t.ui.quick.unpin,
    capture: t.mini_translator.toolbar.tooltip.extract_text_from_screen_capture,
    clipboard: t.mini_translator.toolbar.tooltip.extract_text_from_clipboard,
    openSettings: t.mini_translator.toolbar.menu.open_settings,
    autoDetect: t.mini_translator.language.auto_detect,
    autoMatch: t.mini_translator.language.auto_match,
    swapLanguages: t.workbench.translation.swap_languages,
    translating: t.mini_translator.result.translating,
    empty: t.workbench.translation.empty,
    retry: t.workbench.translation.retry,
    configureServices: t.workbench.translation.configure_services,
    permissionDenied: t.ui.quick.permission_denied,
    permissionNext: t.ui.quick.permission_next,
    captureCancelled: t.ui.quick.capture_cancelled,
    serviceError: t.workbench.translation.failed,
    noServices: t.workbench.translation.no_services,
    failureMessage: translationFailureMessage,
    captureFailed: t.ui.quick.capture_failed,
    ocrNotConfigured: t.ui.quick.ocr_not_configured,
    ocrEmpty: t.ui.quick.ocr_empty,
    emptySelection: t.ui.quick.empty_selection,
    clipboardUnavailable: t.ui.quick.clipboard_unavailable,
    clipboardRestoreFailed: t.ui.quick.clipboard_restore_failed,
    recheck: t.ui.quick.recheck,
    speakSource: t.ui.speech.speak_source,
    speakResult: t.ui.speech.speak_result,
    stopSpeaking: t.ui.speech.stop,
    lookup: t.ui.dictionary.lookup,
    saveVocabulary: t.ui.vocabulary.add,
    vocabularySaved: t.ui.vocabulary.saved,
    favorite: t.workbench.history_page.favorite,
    unfavorite: t.workbench.history_page.unfavorite,
    glossaryMatches: t.workbench.translation.terms,
    glossaryWarnings: t.workbench.translation.quality,
  );
}

SettingsShellLabels settingsShellLabels() => SettingsShellLabels(
  libraryGroup: t.settings.navigation.library_group,
  translationGroup: t.settings.navigation.translation_group,
  translationSettings: t.settings.navigation.translation_settings,
  translationServices: t.settings.navigation.translation_services,
  favorites: t.settings.navigation.favorites,
  history: t.settings.navigation.history,
  glossary: t.ui.shell.glossary,
  vocabulary: t.ui.shell.vocabulary,
  ocrGroup: t.settings.navigation.ocr_group,
  ocrSettings: t.settings.navigation.ocr_settings,
  ocrServices: t.settings.navigation.ocr_services,
  generalGroup: t.settings.navigation.general_group,
  general: t.settings.navigation.general_settings,
  permissions: t.settings.permissions.title,
  integration: t.settings.advanced.title,
  dataTransfer: t.settings.data_transfer.title,
  about: t.settings.about.title,
  updates: t.ui.updates.title,
);

GeneralSettingsLabels generalSettingsLabels() => GeneralSettingsLabels(
  startup: t.settings.general.section.startup,
  launchAtLogin: t.settings.general.row.launch_at_login,
  showInMenuBar: t.settings.general.row.show_in_menu_bar,
  appearance: t.settings.appearance.title,
  language: t.settings.appearance.section.app_language,
  light: t.common.theme_mode.light,
  dark: t.common.theme_mode.dark,
  system: t.common.theme_mode.system,
  error: t.ui.providers.save_failed,
  errorMessage: appErrorMessage,
  retry: t.workbench.translation.retry,
);

ServicesSettingsLabels servicesSettingsLabels() => ServicesSettingsLabels(
  title: t.settings.services.title,
  empty: t.settings.providers.item.empty,
  loading: t.settings.providers.item.loading,
  translation: t.settings.providers.capability.translation,
  dictionary: t.settings.providers.capability.dictionary,
  ocr: t.settings.providers.capability.ocr,
  enabled: t.settings.advanced.enable,
  makeDefault: t.settings.services.make_default,
  isDefault: t.settings.providers.detail.models.default_badge,
  configureProviders: t.settings.services.button.manage_providers,
  commonLanguages: t.settings.general.row.common_languages,
  defaultService: t.settings.general.row.default_translation_service,
  delete: t.common.ui.button.delete,
  deleteConfirm: t.settings.providers.delete_dialog.message,
  errorMessage: appErrorMessage,
);

ProvidersSettingsLabels providersSettingsLabels() => ProvidersSettingsLabels(
  title: t.settings.providers.title,
  empty: t.settings.providers.item.empty,
  loading: t.settings.providers.item.loading,
  add: t.settings.providers.button.add,
  edit: t.common.ui.button.edit,
  delete: t.common.ui.button.delete,
  deleteConfirmTitle: t.settings.providers.delete_dialog.title,
  deleteConfirmBody: t.settings.providers.delete_dialog.message,
  secretStored: t.ui.providers.secret_stored,
  secretPlaceholder: t.ui.providers.secret_placeholder,
  save: t.common.ui.button.save,
  cancel: t.common.ui.button.cancel,
  test: t.settings.providers.editor.test.run,
  testing: t.settings.providers.editor.test.running,
  testPassed: t.settings.providers.editor.test.passed_footer,
  testFailed: t.settings.providers.editor.test.failed_suffix,
  idLabel: t.settings.providers.editor.row.id,
  typeLabel: t.settings.providers.editor.row.type,
  validationMissing: t.ui.providers.validation_missing,
  saveFailed: t.ui.providers.save_failed,
);

ShortcutsSettingsLabels shortcutsSettingsLabels() => ShortcutsSettingsLabels(
  title: t.settings.shortcuts.title,
  record: t.settings.shortcuts.record_placeholder,
  recording: t.settings.shortcuts.recording,
  clear: t.settings.shortcuts.clear,
  reset: t.settings.shortcuts.reset,
  resetConfirmTitle: t.settings.shortcuts.reset_dialog.title,
  resetConfirmBody: t.settings.shortcuts.reset_dialog.message,
  registered: t.settings.shortcuts.group.global.title,
  unregistered: t.settings.shortcuts.clear,
  invalid: t.settings.shortcuts.record_placeholder,
  conflict: (label) => t.settings.shortcuts.conflict(label: label),
  cancel: t.settings.shortcuts.reset_dialog.cancel,
  confirm: t.settings.shortcuts.reset_dialog.confirm,
);

PermissionsSettingsLabels permissionsSettingsLabels() =>
    PermissionsSettingsLabels(
      title: t.settings.permissions.title,
      accessibility: t.settings.general.row.screen_selection_access,
      accessibilityHint: t.settings.general.row.screen_selection_access_hint,
      screenRecording: t.settings.general.row.screen_capture_access,
      screenRecordingHint: t.settings.general.row.screen_capture_access_hint,
      granted: t.settings.general.option.granted,
      denied: t.ui.first_run.denied,
      notRequired: t.ui.first_run.not_required,
      unknown: t.ui.first_run.unknown,
      grant: t.settings.general.button.grant,
      recheck: t.settings.permissions.recheck,
      windowsNote: t.settings.permissions.windows_note,
    );

AboutSettingsLabels aboutSettingsLabels() => AboutSettingsLabels(
  title: t.settings.about.title,
  copyVersion: t.settings.about.copy_version_info,
  copied: t.common.ui.feedback.copied,
  license: t.settings.about.license,
  website: t.settings.about.website,
  changelog: t.settings.about.open_changelog,
  issues: t.settings.about.report_issue,
  copyright: '© LinguaRay contributors · MIT',
);

String shortcutActionLabel(String actionId) => switch (actionId) {
  'toggleQuickWindow' => t.settings.shortcuts.row.toggle_mini_translator,
  'translateSelection' =>
    t.settings.shortcuts.row.extract_text_from_screen_selection,
  'openInputWindow' => t.settings.shortcuts.row.translate_input,
  'captureAndTranslate' =>
    t.settings.shortcuts.row.extract_text_from_screen_capture,
  'captureOcr' => t.settings.shortcuts.row.capture_ocr,
  'silentCaptureOcr' => t.settings.shortcuts.row.silent_capture_ocr,
  'fileOcr' => t.settings.shortcuts.row.file_ocr,
  'clipboardOcr' => t.settings.shortcuts.row.clipboard_ocr,
  'showOcrWindow' => t.settings.shortcuts.row.show_ocr_window,
  'translateInput' => t.settings.shortcuts.row.extract_text_from_clipboard,
  _ => actionId,
};

String shortcutActionDescription(String actionId) => switch (actionId) {
  'toggleQuickWindow' =>
    t.settings.shortcuts.description.toggle_mini_translator,
  'translateSelection' => t.settings.shortcuts.description.selection,
  'openInputWindow' => t.settings.shortcuts.description.input,
  'captureAndTranslate' => t.settings.shortcuts.description.capture,
  'captureOcr' => t.settings.shortcuts.description.capture_ocr,
  'silentCaptureOcr' => t.settings.shortcuts.description.silent_capture_ocr,
  'fileOcr' => t.settings.shortcuts.description.file_ocr,
  'clipboardOcr' => t.settings.shortcuts.description.clipboard_ocr,
  'showOcrWindow' => t.settings.shortcuts.description.show_ocr_window,
  'translateInput' => t.settings.shortcuts.description.clipboard,
  _ => '',
};
