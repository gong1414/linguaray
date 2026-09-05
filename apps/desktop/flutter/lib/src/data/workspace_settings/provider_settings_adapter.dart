import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_runtime/linguaray_runtime.dart'
    show listCatalogSnapshotModels, listProviderCatalog;

import '../../platform/secret_fields.dart';
import '../../platform/secret_store.dart';
import '../../routes/settings/provider_catalog.dart';
import '../../services/runtime.dart';
import '../../services/settings_store.dart';
import '../../utils/provider_util.dart';
import '../provider_draft_validation.dart';

final class RuntimeProviderSettingsAdapter {
  const RuntimeProviderSettingsAdapter(
    this._store,
    this._credentials,
    this._loadCapabilities,
  );

  final SettingsStore _store;
  final ProviderCredentialsController _credentials;
  final Future<PlatformCapabilities> Function() _loadCapabilities;

  Future<List<ServiceRecord>> listServices() async {
    await Future.wait([
      _store.reloadGeneral(),
      _store.reloadServices(),
      _store.reloadProviders(),
    ]);
    final providers = {for (final item in _store.providers) item.id: item};
    final defaultTranslation = _store.defaultTranslationService;
    final defaultOcr = _store.defaultOcrService;
    final capabilities = await _loadCapabilities();
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

  Future<List<ProviderTypeOption>> listProviderTypes() async =>
      providerTypeOptionsFromCatalog(listProviderCatalog());

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
          storedSecretKeys: _storedSecretKeys(provider),
        ),
    ];
  }

  Future<void> saveProvider(ProviderDraft draft) async {
    final type = parseProviderType(draft.typeId);
    final existing = _existingProvider(draft.id);
    _ensureValidDraft(draft, type: type, existing: existing);
    final protected = await _credentials.protectFields(
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

  Future<void> deleteProvider(String providerId) async {
    final existing = await runtime.settings().getProvider(
      providerId: providerId,
    );
    await _credentials.deleteProvider(
      providerId,
      fields: existing?.fields.keys ?? const [],
    );
    await runtime.settings().deleteProvider(providerId: providerId);
    await Future.wait([_store.reloadProviders(), _store.reloadServices()]);
  }

  Future<ProviderTestResult> testProvider(ProviderDraft draft) async {
    try {
      final type = parseProviderType(draft.typeId);
      final existing = _existingProvider(draft.id);
      _ensureValidDraft(draft, type: type, existing: existing);
      final fields = await _credentials.materializeFields(
        providerId: draft.id,
        fields: draft.fields,
        existingFields: existing?.fields ?? const {},
      );
      await runtime.settings().testProvider(
        providerId: draft.id,
        providerType: providerTypeValue(type),
        fields: fields,
      );
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

  Future<ProviderModelDiscovery> discoverProviderModels(
    ProviderDraft draft,
  ) async {
    final type = parseProviderType(draft.typeId);
    final existing = _existingProvider(draft.id);
    final option = findProviderCatalogOption(
      providerTypeOptionsFromCatalog(listProviderCatalog()),
      presetId: draft.presetId,
      engineTypeId: draft.typeId,
    );
    final reference = [
      for (final model in listCatalogSnapshotModels(
        presetId: draft.presetId ?? '',
      ))
        model.id,
    ];
    final validation = validateProviderDraft(
      draft: draft,
      type: option,
      storedSecretKeys: _storedSecretKeys(existing),
      ignoredRequiredFields: const {'defaultModel'},
    );
    if (!validation.isValid) {
      return ProviderModelDiscovery(
        referenceModels: reference,
        errorCode: 'validation_missing',
      );
    }
    final fields = await _credentials.materializeFields(
      providerId: draft.id,
      fields: draft.fields,
      existingFields: existing?.fields ?? const {},
    );
    if (fields['defaultModel']?.trim().isEmpty ?? true) {
      fields['defaultModel'] = '__model_discovery__';
    }
    try {
      final live = await runtime.settings().discoverProviderModels(
        providerId: draft.id,
        providerType: providerTypeValue(type),
        fields: fields,
      );
      return ProviderModelDiscovery(
        liveModels: live.toSet().toList()..sort(),
        referenceModels: reference,
        queriedAt: DateTime.now(),
      );
    } catch (error) {
      // Do not put response bodies, URLs or materialized credentials in UI state.
      final message = error.toString().toLowerCase();
      final code =
          message.contains('401') ||
              message.contains('403') ||
              message.contains('auth')
          ? 'auth_error'
          : message.contains('429') || message.contains('rate limit')
          ? 'rate_limited'
          : message.contains('404') || message.contains('405')
          ? 'unsupported'
          : message.contains('timed out') || message.contains('timeout')
          ? 'timeout'
          : 'network_error';
      return ProviderModelDiscovery(
        referenceModels: reference,
        queriedAt: DateTime.now(),
        errorCode: code,
      );
    }
  }

  Future<List<String>> listProviderModels(String providerId) =>
      runtime.settings().listModels(providerId: providerId);

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

  Future<void> deleteService(String serviceId) async {
    await runtime.settings().deleteService(serviceId: serviceId);
    await _store.reloadServices();
  }

  Future<void> reorderTranslationServices(List<String> serviceIds) async {
    await runtime.settings().setTranslationServiceOrder(order: serviceIds);
    await Future.wait([_store.reloadGeneral(), _store.reloadServices()]);
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

  String _kindName(ServiceType type) => switch (type) {
    ServiceType.ocr => 'ocr',
    ServiceType.dictionary => 'dictionary',
    ServiceType.llm || ServiceType.translation => 'translation',
  };

  ProviderConfigEntry? _existingProvider(String id) =>
      _store.providers.where((item) => item.id == id).firstOrNull;

  void _ensureValidDraft(
    ProviderDraft draft, {
    required ProviderType type,
    required ProviderConfigEntry? existing,
  }) {
    final option = findProviderCatalogOption(
      providerTypeOptionsFromCatalog(listProviderCatalog()),
      presetId: draft.presetId,
      engineTypeId: draft.typeId,
    );
    final validation = validateProviderDraft(
      draft: draft,
      type: option,
      storedSecretKeys: _storedSecretKeys(existing),
    );
    if (!validation.isValid) {
      _throwDraftValidation(draft, validation, type: type);
    }
  }

  Set<String> _storedSecretKeys(ProviderConfigEntry? existing) {
    if (existing == null) return const {};
    return {
      for (final entry in existing.fields.entries)
        if (isSecretField(entry.key) && _credentials.isReference(entry.value))
          entry.key,
    };
  }

  Never _throwDraftValidation(
    ProviderDraft draft,
    ProviderDraftValidation validation, {
    required ProviderType type,
  }) {
    final fieldKey = validation.fieldKey;
    if (fieldKey != null) {
      throw ArgumentError.value(
        fieldKey,
        'fields',
        'Required field is missing',
      );
    }
    throw ArgumentError.value(
      draft.id,
      'id',
      'Provider ID is required for ${providerTypeValue(type)}',
    );
  }
}
