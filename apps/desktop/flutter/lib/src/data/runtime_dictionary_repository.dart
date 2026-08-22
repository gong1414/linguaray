import 'package:linguaray_application/linguaray_application.dart';

import '../services/runtime.dart';
import '../services/settings_store.dart';
import '../utils/platform_util.dart';

final class RuntimeDictionaryRepository implements DictionaryRepository {
  @override
  Future<List<String>> listCompatibleServiceIds() async {
    await settingsStore.reloadServices();
    return [
      for (final service in settingsStore.services)
        if (service.type == ServiceType.dictionary &&
            !(kIsWindows && service.providerId == 'system'))
          service.id,
    ];
  }

  @override
  Future<DictionaryEntry> lookup(DictionaryLookupQuery query) async {
    await settingsStore.reloadServices();
    var providerName = query.serviceId ?? '';
    for (final service in settingsStore.services) {
      if (service.id == query.serviceId && service.name.trim().isNotEmpty) {
        providerName = service.name;
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
