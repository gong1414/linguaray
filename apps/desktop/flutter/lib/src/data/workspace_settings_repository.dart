import 'package:linguaray_application/linguaray_application.dart';

import '../platform/secret_store.dart';
import '../services/settings_store.dart';
import 'workspace_settings/general_settings_adapter.dart';
import 'workspace_settings/provider_settings_adapter.dart';
import 'workspace_settings/system_settings_adapter.dart';

/// Stable application-facing facade over focused runtime settings adapters.
final class RuntimeWorkspaceSettingsRepository
    implements WorkspaceSettingsRepository {
  factory RuntimeWorkspaceSettingsRepository({
    SettingsStore? store,
    ProviderCredentialsController? credentials,
  }) {
    final resolvedStore = store ?? settingsStore;
    final system = RuntimeSystemSettingsAdapter(resolvedStore);
    return RuntimeWorkspaceSettingsRepository._(
      RuntimeGeneralSettingsAdapter(resolvedStore),
      RuntimeProviderSettingsAdapter(
        resolvedStore,
        credentials ?? providerCredentialsController,
        system.loadCapabilities,
      ),
      system,
    );
  }

  const RuntimeWorkspaceSettingsRepository._(
    this._general,
    this._providers,
    this._system,
  );

  final RuntimeGeneralSettingsAdapter _general;
  final RuntimeProviderSettingsAdapter _providers;
  final RuntimeSystemSettingsAdapter _system;

  @override
  Future<GeneralPreferences> loadGeneral() => _general.loadGeneral();

  @override
  Future<void> setLaunchAtLogin(bool value) => _general.setLaunchAtLogin(value);

  @override
  Future<void> setShowInMenuBar(bool value) => _general.setShowInMenuBar(value);

  @override
  Future<void> setLanguage(String language) => _general.setLanguage(language);

  @override
  Future<void> setThemeMode(ThemePreference mode) =>
      _general.setThemeMode(mode);

  @override
  Future<List<LanguageChoice>> listAppLanguages() =>
      _general.listAppLanguages();

  @override
  Future<List<LanguageChoice>> listTranslationLanguages() =>
      _general.listTranslationLanguages();

  @override
  Future<List<String>> loadCommonLanguages() => _general.loadCommonLanguages();

  @override
  Future<void> setCommonLanguages(List<String> codes) =>
      _general.setCommonLanguages(codes);

  @override
  Future<List<TranslationTargetRule>> loadTranslationTargets() =>
      _general.loadTranslationTargets();

  @override
  Future<void> setTranslationTargets(List<TranslationTargetRule> targets) =>
      _general.setTranslationTargets(targets);

  @override
  Future<InputSubmitMode> loadInputSubmitMode() =>
      _general.loadInputSubmitMode();

  @override
  Future<void> setInputSubmitMode(InputSubmitMode mode) =>
      _general.setInputSubmitMode(mode);

  @override
  Future<bool> loadAutoCopyDetectedText() =>
      _general.loadAutoCopyDetectedText();

  @override
  Future<void> setAutoCopyDetectedText(bool value) =>
      _general.setAutoCopyDetectedText(value);

  @override
  Future<bool> loadDoubleClickCopyResult() =>
      _general.loadDoubleClickCopyResult();

  @override
  Future<void> setDoubleClickCopyResult(bool value) =>
      _general.setDoubleClickCopyResult(value);

  @override
  Future<String?> loadDefaultDictionaryService() =>
      _general.loadDefaultDictionaryService();

  @override
  Future<void> setDefaultDictionaryService(String? serviceId) =>
      _general.setDefaultDictionaryService(serviceId);

  @override
  Future<String?> loadDefaultTranslationService() =>
      _general.loadDefaultTranslationService();

  @override
  Future<void> setDefaultTranslationService(String? serviceId) =>
      _general.setDefaultTranslationService(serviceId);

  @override
  Future<String?> loadDefaultOcrService() => _general.loadDefaultOcrService();

  @override
  Future<void> setDefaultOcrService(String? serviceId) =>
      _general.setDefaultOcrService(serviceId);

  @override
  Future<List<ServiceRecord>> listServices() => _providers.listServices();

  @override
  Future<void> setServiceEnabled({
    required String serviceId,
    required bool enabled,
  }) => _providers.setServiceEnabled(serviceId: serviceId, enabled: enabled);

  @override
  Future<List<ProviderTypeOption>> listProviderTypes() =>
      _providers.listProviderTypes();

  @override
  Future<List<ProviderRecord>> listProviders() => _providers.listProviders();

  @override
  Future<void> saveProvider(ProviderDraft draft) =>
      _providers.saveProvider(draft);

  @override
  Future<void> deleteProvider(String providerId) =>
      _providers.deleteProvider(providerId);

  @override
  Future<ProviderTestResult> testProvider(ProviderDraft draft) =>
      _providers.testProvider(draft);

  @override
  Future<List<String>> discoverProviderModels(ProviderDraft draft) =>
      _providers.discoverProviderModels(draft);

  @override
  Future<List<String>> listProviderModels(String providerId) =>
      _providers.listProviderModels(providerId);

  @override
  Future<void> saveService(ServiceDraft draft) => _providers.saveService(draft);

  @override
  Future<void> deleteService(String serviceId) =>
      _providers.deleteService(serviceId);

  @override
  Future<void> reorderTranslationServices(List<String> serviceIds) =>
      _providers.reorderTranslationServices(serviceIds);

  @override
  Future<ApiServerStatus> loadApiServer() => _system.loadApiServer();

  @override
  Future<ApiServerStatus> setApiServerEnabled(bool enabled) =>
      _system.setApiServerEnabled(enabled);

  @override
  Future<ApiServerStatus> setApiServerPort(int port) =>
      _system.setApiServerPort(port);

  @override
  Future<NetworkSettings> loadNetworkSettings() =>
      _system.loadNetworkSettings();

  @override
  Future<NetworkSettings> saveNetworkSettings(NetworkSettings settings) =>
      _system.saveNetworkSettings(settings);

  @override
  Future<PlatformCapabilities> loadCapabilities() => _system.loadCapabilities();

  @override
  Future<AboutInfo> loadAbout() => _system.loadAbout();
}
