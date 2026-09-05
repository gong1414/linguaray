import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_runtime/linguaray_runtime.dart'
    as rt
    show InputSubmitMode;

import '../../../app/runtime.dart' hide InputSubmitMode;
import '../../../app/settings/settings_effects.dart';
import '../../../app/settings/settings_store.dart';
import '../../../shared/language_util.dart';

final class RuntimeGeneralSettingsAdapter
    implements PreferencesRepository, TranslationPreferencesRepository {
  const RuntimeGeneralSettingsAdapter(this._store);

  final SettingsStore _store;

  @override
  Future<GeneralPreferences> loadGeneral() async {
    await Future.wait([_store.reloadGeneral(), _store.reloadAppearance()]);
    return currentPreferences;
  }

  GeneralPreferences get currentPreferences => GeneralPreferences(
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

  @override
  Future<void> setLaunchAtLogin(bool value) async {
    await _store.updateGeneral(GeneralSettingsPatch(launchAtLogin: value));
    final sync = await settingsEffects.syncGeneral();
    if (sync.rejected) {
      throw StateError('The operating system rejected the login item change.');
    }
  }

  @override
  Future<void> setShowInMenuBar(bool value) =>
      _store.updateGeneral(GeneralSettingsPatch(showInMenuBar: value));

  @override
  Future<void> setLanguage(String language) =>
      _store.updateAppearance(AppearanceSettingsPatch(language: language));

  @override
  Future<void> setThemeMode(ThemePreference mode) async {
    await _store.updateAppearance(
      AppearanceSettingsPatch(
        themeMode: switch (mode) {
          ThemePreference.light => 'light',
          ThemePreference.dark => 'dark',
          ThemePreference.system => 'system',
        },
      ),
    );
    await settingsEffects.syncAppearance();
  }

  @override
  Future<List<LanguageChoice>> listAppLanguages() async => [
    for (final code in appLanguages)
      LanguageChoice(code: code, name: getLanguageName(code)),
  ];

  @override
  Future<List<LanguageChoice>> listTranslationLanguages() async => [
    for (final language in runtime.listLanguages())
      LanguageChoice(code: language.code, name: language.localName),
  ];

  @override
  Future<List<String>> loadCommonLanguages() async =>
      List<String>.from(_store.general.commonLanguages);

  @override
  Future<void> setCommonLanguages(List<String> codes) =>
      _store.updateGeneral(GeneralSettingsPatch(commonLanguages: codes));

  @override
  Future<List<TranslationTargetRule>> loadTranslationTargets() async => [
    for (final target in _store.general.translationTargets)
      TranslationTargetRule(
        source: target.source,
        target: target.target,
        enabled: target.enabled,
      ),
  ];

  @override
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

  @override
  Future<InputSubmitMode> loadInputSubmitMode() async =>
      _store.general.inputSubmitMode.name == 'commandEnter'
      ? InputSubmitMode.commandEnter
      : InputSubmitMode.enter;

  @override
  Future<void> setInputSubmitMode(InputSubmitMode mode) {
    return _store.updateGeneral(
      GeneralSettingsPatch(
        inputSubmitMode: mode == InputSubmitMode.commandEnter
            ? rt.InputSubmitMode.commandEnter
            : rt.InputSubmitMode.enter,
      ),
    );
  }

  @override
  Future<bool> loadAutoCopyDetectedText() async =>
      _store.general.autoCopyDetectedText;

  @override
  Future<void> setAutoCopyDetectedText(bool value) =>
      _store.updateGeneral(GeneralSettingsPatch(autoCopyDetectedText: value));

  @override
  Future<bool> loadDoubleClickCopyResult() async =>
      _store.general.doubleClickCopyResult;

  @override
  Future<void> setDoubleClickCopyResult(bool value) =>
      _store.updateGeneral(GeneralSettingsPatch(doubleClickCopyResult: value));

  String? _nullableServiceId(String value) => value.isEmpty ? null : value;
}
