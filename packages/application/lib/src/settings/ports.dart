import 'package:linguaray_application/src/settings/models.dart';

abstract interface class WorkspaceSettingsRepository {
  Future<GeneralPreferences> loadGeneral();

  Future<void> setLaunchAtLogin(bool value);

  Future<void> setShowInMenuBar(bool value);

  Future<void> setLanguage(String language);

  Future<void> setThemeMode(ThemePreference mode);

  Future<List<LanguageChoice>> listAppLanguages();

  Future<List<LanguageChoice>> listTranslationLanguages();

  Future<List<String>> loadCommonLanguages();

  Future<void> setCommonLanguages(List<String> codes);

  Future<String?> loadDefaultTranslationService();

  Future<void> setDefaultTranslationService(String? serviceId);

  Future<String?> loadDefaultOcrService();

  Future<void> setDefaultOcrService(String? serviceId);

  Future<List<ServiceRecord>> listServices();

  Future<void> setServiceEnabled({
    required String serviceId,
    required bool enabled,
  });

  Future<List<ProviderTypeOption>> listProviderTypes();

  Future<List<ProviderRecord>> listProviders();

  Future<void> saveProvider(ProviderDraft draft);

  Future<void> deleteProvider(String providerId);

  Future<ProviderTestResult> testProvider(ProviderDraft draft);

  Future<AboutInfo> loadAbout();
}

abstract interface class PermissionRepository {
  Future<AccessSnapshot> refresh();

  Future<AccessSnapshot> requestAccessibility();

  Future<AccessSnapshot> requestScreenRecording();
}

abstract interface class ShortcutRepository {
  Future<List<ShortcutRecord>> load();

  Future<void> beginRecording();

  Future<void> endRecording();

  Future<void> setAccelerator({
    required String actionId,
    required String accelerator,
  });

  Future<void> clear(String actionId);

  Future<void> resetDefaults();
}
