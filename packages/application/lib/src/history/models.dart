enum HistoryFilter { all, favorites, edited }

final class HistoryRecord {
  const HistoryRecord({
    required this.id,
    required this.source,
    required this.translation,
    required this.sourceLanguage,
    required this.targetLanguage,
    required this.serviceId,
    required this.serviceName,
    required this.favorite,
    required this.edited,
    required this.createdAt,
    required this.updatedAt,
  });

  final String id;
  final String source;
  final String translation;
  final String sourceLanguage;
  final String targetLanguage;
  final String serviceId;
  final String serviceName;
  final bool favorite;
  final bool edited;
  final int createdAt;
  final int updatedAt;
}

final class HistoryRecordDraft {
  const HistoryRecordDraft({
    this.id,
    required this.source,
    required this.translation,
    required this.sourceLanguage,
    required this.targetLanguage,
    required this.serviceId,
    required this.serviceName,
    this.edited = false,
  });

  final String? id;
  final String source;
  final String translation;
  final String sourceLanguage;
  final String targetLanguage;
  final String serviceId;
  final String serviceName;
  final bool edited;
}

final class HistoryCounts {
  const HistoryCounts({
    required this.all,
    required this.favorites,
    required this.edited,
  });

  const HistoryCounts.empty() : all = 0, favorites = 0, edited = 0;

  final int all;
  final int favorites;
  final int edited;
}

final class HistorySnapshot {
  const HistorySnapshot({
    required this.entries,
    required this.counts,
    required this.filter,
    required this.query,
    this.loading = false,
    this.errorCode,
  });

  const HistorySnapshot.empty()
    : entries = const [],
      counts = const HistoryCounts.empty(),
      filter = HistoryFilter.all,
      query = '',
      loading = false,
      errorCode = null;

  final List<HistoryRecord> entries;
  final HistoryCounts counts;
  final HistoryFilter filter;
  final String query;
  final bool loading;
  final String? errorCode;
}
