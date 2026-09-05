import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_runtime/linguaray_runtime.dart'
    show listCatalogSnapshotModels, listProviderCatalog;

import '../../../app/runtime.dart';
import '../../../app/settings/settings_store.dart';
import '../../../platform/credentials/secret_fields.dart';
import '../../../platform/credentials/secret_store.dart';
import 'provider_catalog.dart';
import 'provider_util.dart';

final class RuntimeProviderSettingsAdapter
    implements ProviderSettingsRepository {
  const RuntimeProviderSettingsAdapter(this._store, this._credentials);

  final SettingsStore _store;
  final ProviderCredentialsController _credentials;

  @override
  Future<List<ProviderTypeOption>> listProviderTypes() async =>
      providerTypeOptionsFromCatalog(listProviderCatalog());

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
          storedSecretKeys: _storedSecretKeys(provider),
        ),
    ];
  }

  @override
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

  @override
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

  @override
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

  @override
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

  @override
  Future<List<String>> listProviderModels(String providerId) =>
      runtime.settings().listModels(providerId: providerId);

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
