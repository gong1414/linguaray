import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_runtime/linguaray_runtime.dart' as rt;

import '../services/runtime.dart' as runtime_service;

final class RuntimeHistoryRepository implements HistoryRepository {
  rt.RuntimeHistory get _history => runtime_service.runtime.history();

  @override
  Future<int> clear() => _history.clear();

  @override
  Future<int> delete(List<String> entryIds) =>
      _history.deleteEntries(entryIds: entryIds);

  @override
  Future<HistorySnapshot> load({
    HistoryFilter filter = HistoryFilter.all,
    String query = '',
  }) async {
    try {
      final entries = await _history.listEntries(
        filter: switch (filter) {
          HistoryFilter.all => rt.HistoryFilter.all,
          HistoryFilter.favorites => rt.HistoryFilter.favorites,
          HistoryFilter.edited => rt.HistoryFilter.edited,
        },
        query: query.trim().isEmpty ? null : query.trim(),
      );
      final counts = await _history.counts();
      return HistorySnapshot(
        entries: [for (final entry in entries) _map(entry)],
        counts: HistoryCounts(
          all: counts.all,
          favorites: counts.favorites,
          edited: counts.edited,
        ),
        filter: filter,
        query: query,
      );
    } catch (_) {
      return HistorySnapshot(
        entries: const [],
        counts: const HistoryCounts.empty(),
        filter: filter,
        query: query,
        errorCode: AppErrorCode.historyUnavailable.wireName,
      );
    }
  }

  @override
  Future<HistoryRecord?> setFavorite({
    required String entryId,
    required bool favorite,
  }) async {
    final entry = await _history.setFavorite(
      entryId: entryId,
      favorite: favorite,
    );
    return entry == null ? null : _map(entry);
  }

  @override
  Future<HistoryRecord> upsert(HistoryRecordDraft draft) async {
    final entry = await _history.upsertEntry(
      input: rt.HistoryEntryInput(
        id: draft.id,
        source: draft.source,
        translation: draft.translation,
        sourceLanguage: draft.sourceLanguage,
        targetLanguage: draft.targetLanguage,
        serviceId: draft.serviceId,
        serviceName: draft.serviceName,
        edited: draft.edited,
      ),
    );
    return _map(entry);
  }

  HistoryRecord _map(rt.HistoryEntry entry) {
    return HistoryRecord(
      id: entry.id,
      source: entry.source,
      translation: entry.translation,
      sourceLanguage: entry.sourceLanguage,
      targetLanguage: entry.targetLanguage,
      serviceId: entry.serviceId,
      serviceName: entry.serviceName,
      favorite: entry.favorite,
      edited: entry.edited,
      createdAt: entry.createdAt,
      updatedAt: entry.updatedAt,
    );
  }
}
