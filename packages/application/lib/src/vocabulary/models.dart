enum VocabularyFilter { all, favorites }

final class VocabularyRecord {
  const VocabularyRecord({
    required this.id,
    required this.word,
    required this.translation,
    required this.sourceLanguage,
    required this.targetLanguage,
    required this.source,
    required this.favorite,
    required this.createdAt,
    required this.updatedAt,
    this.note,
  });

  final String id;
  final String word;
  final String translation;
  final String sourceLanguage;
  final String targetLanguage;

  /// Origin of the entry, e.g. `dictionary` or `translation`.
  final String source;
  final bool favorite;
  final String? note;
  final int createdAt;
  final int updatedAt;
}

final class VocabularyDraft {
  const VocabularyDraft({
    this.id,
    required this.word,
    required this.translation,
    required this.sourceLanguage,
    required this.targetLanguage,
    required this.source,
    this.note,
  });

  final String? id;
  final String word;
  final String translation;
  final String sourceLanguage;
  final String targetLanguage;
  final String source;
  final String? note;
}

final class VocabularySnapshot {
  const VocabularySnapshot({
    required this.entries,
    required this.filter,
    required this.query,
    this.loading = false,
    this.errorCode,
  });

  const VocabularySnapshot.empty()
    : entries = const [],
      filter = VocabularyFilter.all,
      query = '',
      loading = false,
      errorCode = null;

  final List<VocabularyRecord> entries;
  final VocabularyFilter filter;
  final String query;
  final bool loading;
  final String? errorCode;
}
