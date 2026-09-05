import 'package:linguaray_application/linguaray_application.dart';

import '../../../app/runtime.dart';
import '../../../app/settings/settings_store.dart';
import '../../providers/data/provider_util.dart';

final class RuntimeServiceSettingsAdapter implements ServiceSettingsRepository {
  const RuntimeServiceSettingsAdapter(this._store, this._loadCapabilities);
  final SettingsStore _store;
  final Future<PlatformCapabilities> Function() _loadCapabilities;

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
  @override
  Future<String?> loadDefaultDictionaryService() async =>
      _nullableServiceId(_store.defaultDirectoryService);

  @override
  Future<void> setDefaultDictionaryService(String? serviceId) =>
      _store.updateGeneral(
        GeneralSettingsPatch(defaultDirectoryService: serviceId ?? ''),
      );

  @override
  Future<String?> loadDefaultTranslationService() async =>
      _nullableServiceId(_store.defaultTranslationService);

  @override
  Future<void> setDefaultTranslationService(String? serviceId) =>
      _store.updateGeneral(
        GeneralSettingsPatch(defaultTranslationService: serviceId ?? ''),
      );

  @override
  Future<String?> loadDefaultOcrService() async =>
      _nullableServiceId(_store.defaultOcrService);

  @override
  Future<void> setDefaultOcrService(String? serviceId) => _store.updateGeneral(
    GeneralSettingsPatch(defaultOcrService: serviceId ?? ''),
  );

  String? _nullableServiceId(String value) => value.isEmpty ? null : value;
}
