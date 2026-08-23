import 'dart:io';

import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_runtime/linguaray_runtime.dart'
    as rt
    show InputSubmitMode;
import 'package:linguaray_runtime/linguaray_runtime.dart'
    show listCatalogSnapshotModels, listProviderCatalog;

import '../platform/secret_fields.dart';
import '../platform/secret_store.dart';
import '../routes/settings/provider_catalog.dart';
import '../services/runtime.dart' hide InputSubmitMode;
import '../services/settings_store.dart';
import '../utils/env.dart';
import '../utils/language_util.dart';
import '../utils/provider_util.dart';

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
      defaultTranslationService: _store.defaultTranslationService.isEmpty
          ? null
          : _store.defaultTranslationService,
      defaultOcrService: _store.defaultOcrService.isEmpty
          ? null
          : _store.defaultOcrService,
      defaultDictionaryService: _store.defaultDirectoryService.isEmpty
          ? null
          : _store.defaultDirectoryService,
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
  Future<List<TranslationTargetRule>> loadTranslationTargets() async {
    return [
      for (final target in _store.general.translationTargets)
        TranslationTargetRule(
          source: target.source,
          target: target.target,
          enabled: target.enabled,
        ),
    ];
  }

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
  Future<InputSubmitMode> loadInputSubmitMode() async {
    return _store.general.inputSubmitMode.name == 'commandEnter'
        ? InputSubmitMode.commandEnter
        : InputSubmitMode.enter;
  }

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
  Future<void> setAutoCopyDetectedText(bool value) {
    return _store.updateGeneral(
      GeneralSettingsPatch(autoCopyDetectedText: value),
    );
  }

  @override
  Future<bool> loadDoubleClickCopyResult() async =>
      _store.general.doubleClickCopyResult;

  @override
  Future<void> setDoubleClickCopyResult(bool value) {
    return _store.updateGeneral(
      GeneralSettingsPatch(doubleClickCopyResult: value),
    );
  }

  @override
  Future<String?> loadDefaultDictionaryService() async {
    final value = _store.defaultDirectoryService;
    return value.isEmpty ? null : value;
  }

  @override
  Future<void> setDefaultDictionaryService(String? serviceId) {
    return _store.updateGeneral(
      GeneralSettingsPatch(defaultDirectoryService: serviceId ?? ''),
    );
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
    await Future.wait([
      _store.reloadGeneral(),
      _store.reloadServices(),
      _store.reloadProviders(),
    ]);
    final providers = {for (final item in _store.providers) item.id: item};
    final defaultTranslation = _store.defaultTranslationService;
    final defaultOcr = _store.defaultOcrService;
    final capabilities = await loadCapabilities();
    return [
      for (final service in _store.services)
        if (_isVisibleService(service, capabilities))
          ServiceRecord(
            id: service.id,
            name: service.name.trim().isEmpty ? service.id : service.name,
            providerId: service.providerId,
            providerName: providers[service.providerId] == null
                ? service.providerId
                : providerTypeDisplayName(providers[service.providerId]!.type),
            kind: _kindName(service.type),
            enabled: isServiceEnabled(service),
            isDefault:
                service.id == defaultTranslation ||
                service.id == defaultOcr ||
                service.id == _store.defaultDirectoryService,
            synthesized: !service.id.contains('+custom-'),
            usable: true,
          ),
    ];
  }

  bool _isVisibleService(
    ServiceConfigEntry service,
    PlatformCapabilities capabilities,
  ) {
    if (service.providerId == 'system' &&
        service.type == ServiceType.translation &&
        !capabilities.systemTranslation) {
      return false;
    }
    if (service.providerId == 'system' &&
        service.type == ServiceType.dictionary &&
        !capabilities.systemDictionary) {
      return false;
    }
    return service.type == ServiceType.translation ||
        service.type == ServiceType.ocr ||
        service.type == ServiceType.dictionary;
  }

  String _kindName(ServiceType type) {
    return switch (type) {
      ServiceType.ocr => 'ocr',
      ServiceType.dictionary => 'dictionary',
      ServiceType.llm => 'translation',
      ServiceType.translation => 'translation',
    };
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
    return providerTypeOptionsFromCatalog(listProviderCatalog());
  }

  @override
  Future<List<ProviderRecord>> listProviders() async {
    await _store.reloadProviders();
    final catalog = providerTypeOptionsFromCatalog(listProviderCatalog());
    return [
      for (final provider in _store.providers)
        ProviderRecord(
          id: provider.id,
          typeId: providerTypeValue(provider.type),
          presetId: provider.presetId,
          displayName:
              findProviderCatalogOption(
                catalog,
                presetId: provider.presetId,
                engineTypeId: providerTypeValue(provider.type),
              )?.label ??
              providerTypeDisplayName(provider.type),
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
      presetId: draft.presetId,
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
  Future<List<String>> discoverProviderModels(ProviderDraft draft) async {
    final type = _typeFromId(draft.typeId);
    final existing = _store.providers
        .where((item) => item.id == draft.id)
        .firstOrNull;
    if (draft.id.trim().isEmpty) {
      throw ArgumentError.value(draft.id, 'id', 'Provider ID is required');
    }
    final catalog = providerTypeOptionsFromCatalog(listProviderCatalog());
    final option = findProviderCatalogOption(
      catalog,
      presetId: draft.presetId,
      engineTypeId: draft.typeId,
    );
    final snapshot = listCatalogSnapshotModels(presetId: draft.presetId ?? '');
    final snapshotIds = [for (final model in snapshot) model.id];
    for (final field in option?.fields ?? const <ProviderFieldSpec>[]) {
      if (!field.requiredField || field.key == 'defaultModel') continue;
      final value = draft.fields[field.key]?.trim() ?? '';
      final existingValue = existing?.fields[field.key];
      final keepsStoredSecret =
          isSecretField(field.key) &&
          existingValue != null &&
          _credentials.isReference(existingValue);
      if (value.isEmpty && !keepsStoredSecret) {
        if (snapshotIds.isNotEmpty) return snapshotIds;
        throw ArgumentError.value(
          field.key,
          'fields',
          'Required field missing',
        );
      }
    }
    final fields = _credentials.materializeFields(
      providerId: draft.id,
      fields: draft.fields,
      existingFields: existing?.fields ?? const {},
    );
    if (isLlmProviderType(type) &&
        (fields['defaultModel']?.trim().isEmpty ?? true)) {
      fields['defaultModel'] = '__model_discovery__';
    }
    List<String> live;
    try {
      live = await runtime.settings().discoverProviderModels(
        providerId: draft.id,
        providerType: providerTypeValue(type),
        fields: fields,
      );
    } catch (_) {
      if (snapshotIds.isNotEmpty) return snapshotIds;
      rethrow;
    }
    final out = <String>[];
    final saved = draft.fields['defaultModel']?.trim();
    if (saved != null && saved.isNotEmpty) out.add(saved);
    for (final id in live) {
      if (!out.contains(id)) out.add(id);
    }
    for (final model in snapshot) {
      if (!out.contains(model.id)) out.add(model.id);
    }
    return out;
  }

  @override
  Future<void> saveService(ServiceDraft draft) async {
    final type = switch (draft.kind) {
      'ocr' => ServiceType.ocr,
      'dictionary' => ServiceType.dictionary,
      _ => ServiceType.translation,
    };
    await runtime.settings().updateService(
      serviceId: draft.id ?? '${draft.providerId}+custom-${draft.kind}',
      providerId: draft.providerId,
      serviceType: type,
      name: draft.name,
      fields: draft.fields,
    );
    await _store.reloadServices();
  }

  @override
  Future<void> deleteService(String serviceId) async {
    await runtime.settings().deleteService(serviceId: serviceId);
    await _store.reloadServices();
  }

  @override
  Future<ApiServerStatus> loadApiServer() async {
    await _store.reloadAdvanced();
    final advanced = _store.advanced;
    try {
      final info = await applyApiServerSettings(advanced);
      return ApiServerStatus(
        enabled: advanced.apiServerEnabled,
        host: advanced.apiServerHost,
        port: info?.port ?? advanced.apiServerPort,
        baseUrl: info?.baseUrl,
      );
    } catch (_) {
      return ApiServerStatus(
        enabled: advanced.apiServerEnabled,
        host: advanced.apiServerHost,
        port: advanced.apiServerPort,
        bindErrorCode: AppErrorCode.apiServerBindFailed.wireName,
      );
    }
  }

  @override
  Future<ApiServerStatus> setApiServerEnabled(bool enabled) async {
    await _store.updateAdvanced(
      AdvancedSettingsPatch(apiServerEnabled: enabled),
    );
    return loadApiServer();
  }

  @override
  Future<ApiServerStatus> setApiServerPort(int port) async {
    if (port < 0 || port > 65535) {
      return ApiServerStatus(
        enabled: _store.advanced.apiServerEnabled,
        host: _store.advanced.apiServerHost,
        port: port,
        bindErrorCode: AppErrorCode.invalidPort.wireName,
      );
    }
    await _store.updateAdvanced(AdvancedSettingsPatch(apiServerPort: port));
    return loadApiServer();
  }

  @override
  Future<PlatformCapabilities> loadCapabilities() async {
    if (Platform.isWindows) return const PlatformCapabilities.windows();
    if (Platform.isMacOS) return const PlatformCapabilities.macos();
    return const PlatformCapabilities.windows();
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

  ProviderType _typeFromId(String id) => parseProviderType(id);

  void _validateProviderDraft(
    ProviderDraft draft, {
    required ProviderType type,
    required ProviderConfigEntry? existing,
  }) {
    if (draft.id.trim().isEmpty) {
      throw ArgumentError.value(
        draft.id,
        'id',
        'Provider ID is required for ${providerTypeValue(type)}',
      );
    }
    final catalog = providerTypeOptionsFromCatalog(listProviderCatalog());
    final option = findProviderCatalogOption(
      catalog,
      presetId: draft.presetId,
      engineTypeId: draft.typeId,
    );
    for (final field in option?.fields ?? const <ProviderFieldSpec>[]) {
      if (!field.requiredField) continue;
      final value = draft.fields[field.key]?.trim() ?? '';
      final existingValue = existing?.fields[field.key];
      final keepsStoredSecret =
          isSecretField(field.key) &&
          existingValue != null &&
          _credentials.isReference(existingValue);
      if (value.isEmpty && !keepsStoredSecret) {
        throw ArgumentError.value(
          field.key,
          'fields',
          'Required field is missing',
        );
      }
    }
  }

  @override
  Future<List<String>> listProviderModels(String providerId) async {
    try {
      final live = await runtime.settings().listModels(providerId: providerId);
      final provider = _store.providers
          .where((item) => item.id == providerId)
          .firstOrNull;
      final snapshot = listCatalogSnapshotModels(
        presetId: provider?.presetId ?? '',
      );
      final saved = provider?.fields['defaultModel']?.trim();
      final out = <String>[];
      if (saved != null && saved.isNotEmpty) out.add(saved);
      for (final id in live) {
        if (!out.contains(id)) out.add(id);
      }
      for (final model in snapshot) {
        if (!out.contains(model.id)) out.add(model.id);
      }
      return out;
    } catch (_) {
      final provider = _store.providers
          .where((item) => item.id == providerId)
          .firstOrNull;
      final snapshot = listCatalogSnapshotModels(
        presetId: provider?.presetId ?? '',
      );
      final saved = provider?.fields['defaultModel']?.trim();
      return [
        if (saved != null && saved.isNotEmpty) saved,
        for (final model in snapshot) model.id,
      ];
    }
  }

  @override
  Future<void> reorderTranslationServices(List<String> serviceIds) async {
    await runtime.settings().setTranslationServiceOrder(order: serviceIds);
    await _store.reloadGeneral();
    await _store.reloadServices();
  }
}
