import 'package:linguaray_application/src/settings/models.dart';

abstract interface class ProviderSettingsRepository {
  Future<List<ProviderTypeOption>> listProviderTypes();

  Future<List<ProviderRecord>> listProviders();

  Future<void> saveProvider(ProviderDraft draft);

  Future<void> deleteProvider(String providerId);

  Future<ProviderTestResult> testProvider(ProviderDraft draft);

  Future<ProviderModelDiscovery> discoverProviderModels(ProviderDraft draft);

  Future<List<String>> listProviderModels(String providerId);
}
