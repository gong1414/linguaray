// ignore_for_file: prefer_initializing_formals

import 'package:linguaray_application/linguaray_application.dart';

import '../../../app/runtime.dart';
import '../../../app/settings/settings_section.dart';
import '../../../app/settings/settings_store.dart';
import '../../../platform/platform_util.dart';
import '../../providers/data/provider_util.dart';

final class RuntimeDictionaryRepository implements DictionaryRepository {
  RuntimeDictionaryRepository({required SettingsStore store}) : _store = store;

  final SettingsStore _store;

  @override
  Future<List<String>> listCompatibleServiceIds() async {
    await Future.wait([_store.reloadGeneral(), _store.reloadServices()]);
    _store.throwIfErrored(SettingsSection.general);
    _store.throwIfErrored(SettingsSection.services);
    final services = [
      for (final service in _store.services)
        if (service.type == ServiceType.dictionary &&
            isServiceEnabled(service) &&
            !(kIsWindows && service.providerId == 'system'))
          service.id,
    ];
    final configured = _store.defaultDirectoryService;
    final preferred = configured.isNotEmpty ? configured : 'ecdict+dictionary';
    if (preferred.isNotEmpty && services.remove(preferred)) {
      services.insert(0, preferred);
    }
    return services;
  }

  @override
  Future<DictionaryEntry> lookup(DictionaryLookupQuery query) async {
    await Future.wait([_store.reloadProviders(), _store.reloadServices()]);
    _store.throwIfErrored(SettingsSection.providers);
    _store.throwIfErrored(SettingsSection.services);
    var providerName = query.serviceId ?? '';
    for (final service in _store.services) {
      if (service.id == query.serviceId && service.name.trim().isNotEmpty) {
        final configuredName = service.name.trim();
        final generatedName =
            configuredName == service.id ||
            configuredName == service.providerId ||
            configuredName.contains('+');
        final provider = _store.providers
            .where((item) => item.id == service.providerId)
            .firstOrNull;
        providerName = generatedName && provider != null
            ? providerTypeDisplayName(provider.type)
            : configuredName;
        break;
      }
    }
    final response = await runtime
        .dictionary(providerId: query.serviceId ?? '')
        .lookup(
          request: LookUpRequest(
            sourceLanguage: query.sourceLanguage,
            targetLanguage: query.targetLanguage,
            word: query.word,
          ),
        );
    return DictionaryEntry(
      word: response.word ?? query.word,
      providerName: providerName,
      serviceId: query.serviceId ?? '',
      translations: [for (final item in response.translations) item.text],
      pronunciations: [
        for (final item in response.pronunciations ?? const [])
          DictionaryPronunciation(
            text: item.phoneticSymbol ?? '',
            accent: item.type,
          ),
      ],
      definitions: [
        for (final item in response.definitions ?? const [])
          DictionaryDefinition(
            partOfSpeech: item.name,
            values: item.values ?? const [],
          ),
      ],
    );
  }
}
