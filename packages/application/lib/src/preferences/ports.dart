import 'package:linguaray_application/src/settings/models.dart';

abstract interface class PreferencesRepository {
  Future<GeneralPreferences> loadGeneral();

  Future<void> setLaunchAtLogin(bool value);

  Future<void> setShowInMenuBar(bool value);

  Future<void> setLanguage(String language);

  Future<void> setThemeMode(ThemePreference mode);

  Future<List<LanguageChoice>> listAppLanguages();
}
