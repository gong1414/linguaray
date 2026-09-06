import 'package:linguaray_application/src/settings/models.dart';

abstract interface class TranslationPreferencesRepository {
  Future<List<LanguageChoice>> listTranslationLanguages();

  Future<List<String>> loadCommonLanguages();

  Future<void> setCommonLanguages(List<String> codes);

  Future<List<TranslationTargetRule>> loadTranslationTargets();

  Future<void> setTranslationTargets(List<TranslationTargetRule> targets);

  Future<InputSubmitMode> loadInputSubmitMode();

  Future<void> setInputSubmitMode(InputSubmitMode mode);

  Future<bool> loadAutoCopyDetectedText();

  Future<void> setAutoCopyDetectedText(bool value);

  Future<bool> loadDoubleClickCopyResult();

  Future<void> setDoubleClickCopyResult(bool value);
}
