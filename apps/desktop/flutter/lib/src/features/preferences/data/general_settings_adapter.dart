import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_runtime/linguaray_runtime.dart'
    as rt
    show InputSubmitMode;

import '../../../app/runtime.dart' hide InputSubmitMode;
import '../../../app/settings/settings_store.dart';
import '../../../shared/language_util.dart';

final class RuntimeGeneralSettingsAdapter {
  const RuntimeGeneralSettingsAdapter(this._store);

  final SettingsStore _store;

  Future<GeneralPreferences> loadGeneral() async {
    await Future.wait([_store.reloadGeneral(), _store.reloadAppearance()]);
    return GeneralPreferences(
      launchAtLogin: _store.general.launchAtLogin,
      showInMenuBar: _store.general.showInMenuBar,
      language: _store.appearance.language,
      themeMode: switch (_store.appearance.themeMode) {
        'light' => ThemePreference.light,
        'dark' => ThemePreference.dark,
        _ => ThemePreference.system,
      },
      commonLanguages: List<String>.from(_store.general.commonLanguages),
      translationTargets: [
        for (final target in _store.general.translationTargets)
          TranslationTargetRule(
            source: target.source,
            target: target.target,
            enabled: target.enabled,
          ),
      ],
      inputSubmitMode: _store.general.inputSubmitMode.name == 'commandEnter'
          ? InputSubmitMode.commandEnter
          : InputSubmitMode.enter,
      autoCopyDetectedText: _store.general.autoCopyDetectedText,
      doubleClickCopyResult: _store.general.doubleClickCopyResult,
      defaultTranslationService: _nullableServiceId(
        _store.defaultTranslationService,
      ),
      defaultOcrService: _nullableServiceId(_store.defaultOcrService),
      defaultDictionaryService: _nullableServiceId(
        _store.defaultDirectoryService,
      ),
    );
  }

  Future<void> setLaunchAtLogin(bool value) =>
      _store.updateGeneral(GeneralSettingsPatch(launchAtLogin: value));

  Future<void> setShowInMenuBar(bool value) =>
      _store.updateGeneral(GeneralSettingsPatch(showInMenuBar: value));

  Future<void> setLanguage(String language) =>
      _store.updateAppearance(AppearanceSettingsPatch(language: language));

  Future<void> setThemeMode(ThemePreference mode) {
    return _store.updateAppearance(
      AppearanceSettingsPatch(
        themeMode: switch (mode) {
          ThemePreference.light => 'light',
          ThemePreference.dark => 'dark',
          ThemePreference.system => 'system',
        },
      ),
    );
  }

  Future<List<LanguageChoice>> listAppLanguages() async => [
    for (final code in appLanguages)
      LanguageChoice(code: code, name: getLanguageName(code)),
  ];

  Future<List<LanguageChoice>> listTranslationLanguages() async => [
    for (final language in runtime.listLanguages())
      LanguageChoice(code: language.code, name: language.localName),
  ];

  Future<List<String>> loadCommonLanguages() async =>
      List<String>.from(_store.general.commonLanguages);

  Future<void> setCommonLanguages(List<String> codes) =>
      _store.updateGeneral(GeneralSettingsPatch(commonLanguages: codes));

  Future<List<TranslationTargetRule>> loadTranslationTargets() async => [
    for (final target in _store.general.translationTargets)
      TranslationTargetRule(
        source: target.source,
        target: target.target,
        enabled: target.enabled,
      ),
  ];

  Future<void> setTranslationTargets(List<TranslationTargetRule> targets) {
    return _store.updateGeneral(
      GeneralSettingsPatch(
        translationTargets: [
          for (final target in targets)
            TranslationTarget(
              source: target.source,
              target: target.target,
              enabled: target.enabled,
            ),
        ],
      ),
    );
  }

  Future<InputSubmitMode> loadInputSubmitMode() async =>
      _store.general.inputSubmitMode.name == 'commandEnter'
      ? InputSubmitMode.commandEnter
      : InputSubmitMode.enter;

  Future<void> setInputSubmitMode(InputSubmitMode mode) {
    return _store.updateGeneral(
      GeneralSettingsPatch(
        inputSubmitMode: mode == InputSubmitMode.commandEnter
            ? rt.InputSubmitMode.commandEnter
            : rt.InputSubmitMode.enter,
      ),
    );
  }

  Future<bool> loadAutoCopyDetectedText() async =>
      _store.general.autoCopyDetectedText;

  Future<void> setAutoCopyDetectedText(bool value) =>
      _store.updateGeneral(GeneralSettingsPatch(autoCopyDetectedText: value));

  Future<bool> loadDoubleClickCopyResult() async =>
      _store.general.doubleClickCopyResult;

  Future<void> setDoubleClickCopyResult(bool value) =>
      _store.updateGeneral(GeneralSettingsPatch(doubleClickCopyResult: value));

  Future<String?> loadDefaultDictionaryService() async =>
      _nullableServiceId(_store.defaultDirectoryService);

  Future<void> setDefaultDictionaryService(String? serviceId) =>
      _store.updateGeneral(
        GeneralSettingsPatch(defaultDirectoryService: serviceId ?? ''),
      );

  Future<String?> loadDefaultTranslationService() async =>
      _nullableServiceId(_store.defaultTranslationService);

  Future<void> setDefaultTranslationService(String? serviceId) =>
      _store.updateGeneral(
        GeneralSettingsPatch(defaultTranslationService: serviceId ?? ''),
      );

  Future<String?> loadDefaultOcrService() async =>
      _nullableServiceId(_store.defaultOcrService);

  Future<void> setDefaultOcrService(String? serviceId) => _store.updateGeneral(
    GeneralSettingsPatch(defaultOcrService: serviceId ?? ''),
  );

  String? _nullableServiceId(String value) => value.isEmpty ? null : value;
}
