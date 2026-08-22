final class DictionaryLookupQuery {
  const DictionaryLookupQuery({
    required this.word,
    required this.sourceLanguage,
    required this.targetLanguage,
    this.serviceId,
  });

  final String word;
  final String sourceLanguage;
  final String targetLanguage;
  final String? serviceId;
}

final class DictionaryDefinition {
  const DictionaryDefinition({
    required this.partOfSpeech,
    required this.values,
  });

  final String? partOfSpeech;
  final List<String> values;
}

final class DictionaryPronunciation {
  const DictionaryPronunciation({required this.text, this.accent});

  final String text;
  final String? accent;
}

final class DictionaryEntry {
  const DictionaryEntry({
    required this.word,
    required this.providerName,
    required this.serviceId,
    this.translations = const [],
    this.pronunciations = const [],
    this.definitions = const [],
  });

  final String word;
  final String providerName;
  final String serviceId;
  final List<String> translations;
  final List<DictionaryPronunciation> pronunciations;
  final List<DictionaryDefinition> definitions;

  bool get isEmpty =>
      translations.isEmpty && pronunciations.isEmpty && definitions.isEmpty;
}
