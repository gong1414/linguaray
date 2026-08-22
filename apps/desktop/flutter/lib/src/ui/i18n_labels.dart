import '../i18n/i18n.dart';
import 'chrome/workbench_shell_view.dart';
import 'first_run/first_run_view.dart';
import 'quick_translate/widgets/quick_translate_view.dart';
import 'settings/settings_labels.dart';
import 'translation/widgets/translation_workspace_view.dart';

WorkbenchShellLabels workbenchShellLabels() => WorkbenchShellLabels(
  appName: t.ui.shell.app_name,
  translate: t.ui.shell.translate,
  settings: t.ui.shell.settings,
  minimize: t.ui.shell.minimize,
  maximize: t.ui.shell.maximize,
  close: t.ui.shell.close,
);

FirstRunLabels firstRunLabels() {
  final ui = t.ui.first_run;
  return FirstRunLabels(
    title: ui.title,
    subtitle: ui.subtitle,
    permissionsTitle: ui.permissions_title,
    permissionsBody: ui.permissions_body,
    accessibility: ui.accessibility,
    screenRecording: ui.screen_recording,
    shortcutsTitle: ui.shortcuts_title,
    shortcutsBody: ui.shortcuts_body,
    servicesTitle: ui.services_title,
    servicesBody: ui.services_body,
    granted: ui.granted,
    denied: ui.denied,
    notRequired: ui.not_required,
    unknown: ui.unknown,
    checking: ui.checking,
    conflict: ui.conflict,
    noProvider: ui.no_provider,
    ready: ui.ready,
    grant: ui.grant,
    recheck: ui.recheck,
    configureServices: ui.configure_services,
    start: ui.start,
    skip: ui.skip,
  );
}

TranslationWorkspaceLabels translationWorkspaceLabels() {
  final translation = t.workbench.translation;
  return TranslationWorkspaceLabels(
    title: t.workbench.translate,
    subtitle: t.workbench.subtitle.translate,
    source: translation.source,
    target: translation.target,
    autoDetect: t.mini_translator.language.auto_detect,
    autoMatch: t.mini_translator.language.auto_match,
    inputHint: translation.input_hint,
    translate: translation.button,
    clear: translation.clear,
    swapLanguages: translation.swap_languages,
    loadingServices: translation.loading_services,
    noServices: translation.no_services,
    translating: translation.translating,
    failed: translation.failed,
    empty: translation.empty,
    services: translation.services,
    copy: translation.copy_result,
    copied: translation.copied,
    configureServices: translation.configure_services,
    retry: translation.retry,
    characterCount: (count) => translation.character_count(count: count),
    failureMessage: translationFailureMessage,
    partialFailure: (count) => translation.partial_failure(count: count),
    streaming: translation.streaming,
  );
}

String translationFailureMessage(String? code) {
  final translation = t.workbench.translation;
  return switch (code) {
    'language_pair_not_installed' => translation.language_pair_not_installed,
    'unsupported_language_pair' => translation.unsupported_language_pair,
    'source_language_detection_failed' =>
      translation.source_language_detection_failed,
    'network_error' => translation.network_error,
    _ => translation.failed,
  };
}

QuickTranslateLabels quickTranslateLabels() {
  return QuickTranslateLabels(
    title: t.ui.quick.title,
    inputHint: t.ui.quick.input_hint,
    translate: t.mini_translator.button.translate,
    clear: t.mini_translator.button.clear,
    copy: t.mini_translator.button.copy,
    copied: t.mini_translator.button.copied,
    pin: t.ui.quick.pin,
    unpin: t.ui.quick.unpin,
    capture: t.mini_translator.toolbar.tooltip.extract_text_from_screen_capture,
    clipboard: t.mini_translator.toolbar.tooltip.extract_text_from_clipboard,
    openWorkbench: t.mini_translator.toolbar.menu.open_main_window,
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
  );
}

SettingsShellLabels settingsShellLabels() => SettingsShellLabels(
  title: t.settings.layout.title,
  general: t.settings.general.title,
  services: t.settings.services.title,
  providers: t.settings.providers.title,
  shortcuts: t.settings.shortcuts.title,
  permissions: t.settings.permissions.title,
  about: t.settings.about.title,
);

GeneralSettingsLabels generalSettingsLabels() => GeneralSettingsLabels(
  startup: t.settings.general.section.startup,
  launchAtLogin: t.settings.general.row.launch_at_login,
  showInMenuBar: t.settings.general.row.show_in_menu_bar,
  appearance: t.settings.appearance.title,
  language: t.settings.appearance.section.app_language,
  theme: t.settings.appearance.section.theme_mode,
  light: t.common.theme_mode.light,
  dark: t.common.theme_mode.dark,
  system: t.common.theme_mode.system,
);

ServicesSettingsLabels servicesSettingsLabels() => ServicesSettingsLabels(
  title: t.settings.services.title,
  empty: t.settings.providers.item.empty,
  loading: t.settings.providers.item.loading,
  translation: t.settings.providers.capability.translation,
  ocr: t.settings.providers.capability.ocr,
  enabled: t.settings.advanced.enable,
  makeDefault: t.settings.services.make_default,
  isDefault: t.settings.providers.detail.models.default_badge,
  configureProviders: t.settings.providers.button.add,
  commonLanguages: t.settings.general.row.common_languages,
  defaultService: t.settings.general.row.default_translation_service,
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
  'captureAndTranslate' =>
    t.settings.shortcuts.row.extract_text_from_screen_capture,
  'translateInput' => t.settings.shortcuts.row.extract_text_from_clipboard,
  _ => actionId,
};
