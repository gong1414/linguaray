import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_runtime/linguaray_runtime.dart' as rt;

import '../../../../app/runtime.dart' as runtime_service;

final class RuntimeVocabularyRepository implements VocabularyRepository {
  rt.RuntimeVocabulary get _vocabulary => runtime_service.runtime.vocabulary();

  @override
  Future<int> delete(List<String> entryIds) {
    return _vocabulary.deleteEntries(entryIds: entryIds);
  }

  @override
  Future<VocabularySnapshot> load({
    VocabularyFilter filter = VocabularyFilter.all,
    String query = '',
  }) async {
    try {
      final entries = await _vocabulary.listEntries(
        filter: filter == VocabularyFilter.favorites
            ? rt.VocabularyFilter.favorites
            : rt.VocabularyFilter.all,
        query: query.trim().isEmpty ? null : query.trim(),
      );
      return VocabularySnapshot(
        entries: [for (final entry in entries) _map(entry)],
        filter: filter,
        query: query,
      );
    } catch (_) {
      return VocabularySnapshot(
        entries: const [],
        filter: filter,
        query: query,
        errorCode: AppErrorCode.vocabularyUnavailable.wireName,
      );
    }
  }

  @override
  Future<VocabularyRecord?> setFavorite({
    required String entryId,
    required bool favorite,
  }) async {
    final entry = await _vocabulary.setFavorite(
      entryId: entryId,
      favorite: favorite,
    );
    return entry == null ? null : _map(entry);
  }

  @override
  Future<VocabularyRecord?> updateNote({
    required String entryId,
    String? note,
  }) async {
    final entry = await _vocabulary.setNote(entryId: entryId, note: note);
    return entry == null ? null : _map(entry);
  }

  @override
  Future<VocabularyRecord> upsert(VocabularyDraft draft) async {
    final entry = await _vocabulary.upsertEntry(
      input: rt.VocabularyEntryInput(
        id: draft.id,
        word: draft.word,
        translation: draft.translation,
        sourceLanguage: draft.sourceLanguage,
        targetLanguage: draft.targetLanguage,
        source: draft.source,
        note: draft.note,
      ),
    );
    return _map(entry);
  }

  VocabularyRecord _map(rt.VocabularyEntry entry) {
    return VocabularyRecord(
      id: entry.id,
      word: entry.word,
      translation: entry.translation,
      sourceLanguage: entry.sourceLanguage,
      targetLanguage: entry.targetLanguage,
      source: entry.source,
      favorite: entry.favorite,
      note: entry.note,
      createdAt: entry.createdAt,
      updatedAt: entry.updatedAt,
    );
  }
}
