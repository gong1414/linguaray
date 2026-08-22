import 'dart:io';

import 'package:linguaray_application/linguaray_application.dart';

import '../platform/secret_store.dart';
import '../routes/settings/provider_meta.dart';
import '../services/runtime.dart';
import '../services/settings_store.dart';
import '../utils/env.dart';
import '../utils/language_util.dart';

final class RuntimeWorkspaceSettingsRepository
    implements WorkspaceSettingsRepository {
  RuntimeWorkspaceSettingsRepository({
    SettingsStore? store,
    ProviderCredentialsController? credentials,
  }) : _store = store ?? settingsStore,
       _credentials = credentials ?? providerCredentialsController;

  final SettingsStore _store;
  final ProviderCredentialsController _credentials;

  @override
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
    );
  }

  @override
  Future<void> setLaunchAtLogin(bool value) {
    return _store.updateGeneral(GeneralSettingsPatch(launchAtLogin: value));
  }

  @override
  Future<void> setShowInMenuBar(bool value) {
    return _store.updateGeneral(GeneralSettingsPatch(showInMenuBar: value));
  }

  @override
  Future<void> setLanguage(String language) {
    return _store.updateAppearance(AppearanceSettingsPatch(language: language));
  }

  @override
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

  @override
  Future<List<LanguageChoice>> listAppLanguages() async {
    return [
      for (final code in appLanguages)
        LanguageChoice(code: code, name: getLanguageName(code)),
    ];
  }

  @override
  Future<List<LanguageChoice>> listTranslationLanguages() async {
    return [
      for (final language in runtime.listLanguages())
        LanguageChoice(code: language.code, name: language.localName),
    ];
  }

  @override
  Future<List<String>> loadCommonLanguages() async {
    return List<String>.from(_store.general.commonLanguages);
  }

  @override
  Future<void> setCommonLanguages(List<String> codes) {
    return _store.updateGeneral(GeneralSettingsPatch(commonLanguages: codes));
  }

  @override
  Future<String?> loadDefaultTranslationService() async {
    final value = _store.defaultTranslationService;
    return value.isEmpty ? null : value;
  }

  @override
  Future<void> setDefaultTranslationService(String? serviceId) {
    return _store.updateGeneral(
      GeneralSettingsPatch(defaultTranslationService: serviceId ?? ''),
    );
  }

  @override
  Future<String?> loadDefaultOcrService() async {
    final value = _store.defaultOcrService;
    return value.isEmpty ? null : value;
  }

  @override
  Future<void> setDefaultOcrService(String? serviceId) {
    return _store.updateGeneral(
      GeneralSettingsPatch(defaultOcrService: serviceId ?? ''),
    );
  }

  @override
  Future<List<ServiceRecord>> listServices() async {
    await Future.wait([_store.reloadServices(), _store.reloadProviders()]);
    final providers = {for (final item in _store.providers) item.id: item};
    final defaultTranslation = _store.defaultTranslationService;
    final defaultOcr = _store.defaultOcrService;
    return [
      for (final service in _store.services)
        if (service.type == ServiceType.translation ||
            service.type == ServiceType.ocr)
          ServiceRecord(
            id: service.id,
            name: service.name.trim().isEmpty ? service.id : service.name,
            providerId: service.providerId,
            providerName: providers[service.providerId] == null
                ? service.providerId
                : providerTypeDisplayName(providers[service.providerId]!.type),
            kind: service.type == ServiceType.ocr ? 'ocr' : 'translation',
            enabled: isServiceEnabled(service),
            isDefault:
                service.id == defaultTranslation || service.id == defaultOcr,
          ),
    ];
  }

  @override
  Future<void> setServiceEnabled({
    required String serviceId,
    required bool enabled,
  }) async {
    final service = _store.services
        .where((item) => item.id == serviceId)
        .firstOrNull;
    if (service == null) return;
    final fields = Map<String, String>.from(service.fields)
      ..[kServiceEnabledField] = enabled ? 'true' : 'false';
    await runtime.settings().updateService(
      serviceId: service.id,
      providerId: service.providerId,
      serviceType: service.type,
      name: service.name,
      fields: fields,
    );
    await _store.reloadServices();
  }

  @override
  Future<List<ProviderTypeOption>> listProviderTypes() async {
    return [
      for (final type in kKnownProviderTypes)
        ProviderTypeOption(
          id: providerTypeValue(type),
          label: providerTypeDisplayName(type),
          isLlm: isLlmProviderType(type),
          fields: [
            for (final key in kProviderFields[type] ?? const <String>[])
              ProviderFieldSpec(
                key: key,
                label: key,
                secret: isSecretField(key),
                requiredField:
                    (kRequiredProviderFields[type] ?? const <String>[])
                        .contains(key),
                placeholder: key == 'baseUrl' ? defaultBaseUrl(type) : null,
              ),
          ],
        ),
    ];
  }

  @override
  Future<List<ProviderRecord>> listProviders() async {
    await _store.reloadProviders();
    return [
      for (final provider in _store.providers)
        ProviderRecord(
          id: provider.id,
          typeId: providerTypeValue(provider.type),
          displayName: providerTypeDisplayName(provider.type),
          publicFields: {
            for (final entry in provider.fields.entries)
              if (!isSecretField(entry.key)) entry.key: entry.value,
          },
          storedSecretKeys: {
            for (final entry in provider.fields.entries)
              if (isSecretField(entry.key) &&
                  _credentials.isReference(entry.value))
                entry.key,
          },
        ),
    ];
  }

  @override
  Future<void> saveProvider(ProviderDraft draft) async {
    final type = _typeFromId(draft.typeId);
    final existing = _store.providers
        .where((item) => item.id == draft.id)
        .firstOrNull;
    _validateProviderDraft(draft, type: type, existing: existing);
    final protected = _credentials.protectFields(
      providerId: draft.id,
      fields: draft.fields,
      existingFields: existing?.fields ?? const {},
    );
    final provider = await runtime.settings().updateProvider(
      providerId: draft.id,
      providerType: providerTypeValue(type),
      fields: protected,
    );
    await _credentials.hydrateProvider(provider);
    await Future.wait([_store.reloadProviders(), _store.reloadServices()]);
  }

  @override
  Future<void> deleteProvider(String providerId) async {
    await runtime.settings().deleteProvider(providerId: providerId);
    _credentials.deleteProvider(providerId);
    await Future.wait([_store.reloadProviders(), _store.reloadServices()]);
  }

  @override
  Future<ProviderTestResult> testProvider(ProviderDraft draft) async {
    try {
      final type = _typeFromId(draft.typeId);
      final existing = _store.providers
          .where((item) => item.id == draft.id)
          .firstOrNull;
      _validateProviderDraft(draft, type: type, existing: existing);
      final fields = _credentials.materializeFields(
        providerId: draft.id,
        fields: draft.fields,
        existingFields: existing?.fields ?? const {},
      );
      final modelCount = await runtime.settings().testProvider(
        providerId: draft.id,
        providerType: providerTypeValue(type),
        fields: fields,
      );
      if (isLlmProviderType(type)) {
        return ProviderTestResult(
          status: ProviderTestStatus.passed,
          message: '$modelCount',
        );
      }
      return const ProviderTestResult(status: ProviderTestStatus.passed);
    } on ArgumentError {
      return const ProviderTestResult(
        status: ProviderTestStatus.failed,
        errorCode: 'validation_missing',
      );
    } catch (_) {
      return const ProviderTestResult(
        status: ProviderTestStatus.failed,
        errorCode: 'network_error',
      );
    }
  }

  @override
  Future<AboutInfo> loadAbout() async {
    return AboutInfo(
      appName: 'LinguaRay',
      version: Env.instance.appVersion,
      buildNumber: '${Env.instance.appBuildNumber}',
      platformLabel: Platform.operatingSystem,
      license: 'MIT',
    );
  }

  ProviderType _typeFromId(String id) {
    for (final type in kKnownProviderTypes) {
      if (providerTypeValue(type) == id) return type;
    }
    throw ArgumentError.value(id, 'typeId', 'Unknown provider type');
  }

  void _validateProviderDraft(
    ProviderDraft draft, {
    required ProviderType type,
    required ProviderConfigEntry? existing,
  }) {
    if (draft.id.trim().isEmpty) {
      throw ArgumentError.value(draft.id, 'id', 'Provider ID is required');
    }
    for (final key in kRequiredProviderFields[type] ?? const <String>[]) {
      final value = draft.fields[key]?.trim() ?? '';
      final existingValue = existing?.fields[key];
      final keepsStoredSecret =
          isSecretField(key) &&
          existingValue != null &&
          _credentials.isReference(existingValue);
      if (value.isEmpty && !keepsStoredSecret) {
        throw ArgumentError.value(key, 'fields', 'Required field is missing');
      }
    }
  }
}
