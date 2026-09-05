import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/app/dependencies.dart';
import 'package:linguaray_desktop/src/features/library/glossary/glossary_view_model.dart';
import 'package:linguaray_desktop/src/features/library/history/history_view_model.dart';
import 'package:linguaray_desktop/src/features/library/vocabulary/vocabulary_view_model.dart';

void main() {
  test(
    'glossary CRUD stays in the view model and ignores corrupt books',
    () async {
      final repository = _MemoryGlossary()
        ..books.addAll(const [
          GlossaryBookRecord(
            id: 'ui',
            name: 'UI',
            enabled: true,
            entryCount: 1,
          ),
          GlossaryBookRecord(
            id: 'bad',
            name: 'Broken',
            enabled: true,
            entryCount: 0,
            errorCode: 'glossary_corrupt',
          ),
        ])
        ..entries['ui'] = const [
          GlossaryEntryRecord(
            id: 'e1',
            term: 'window',
            translation: '窗口',
            forbidden: [],
            caseSensitive: false,
            wholeWord: true,
          ),
        ];
      final container = _container(glossary: repository);
      addTearDown(container.dispose);
      final subscription = container.listen(
        glossaryViewModelProvider,
        (_, _) {},
        fireImmediately: true,
      );
      addTearDown(subscription.close);
      await _waitFor(() => !container.read(glossaryViewModelProvider).loading);

      var state = container.read(glossaryViewModelProvider);
      expect(state.books.map((book) => book.id), ['ui']);
      expect(state.selectedBookId, 'ui');
      expect(state.entries.single.term, 'window');
      expect(state.errorCode, 'glossary_corrupt');

      await container
          .read(glossaryViewModelProvider.notifier)
          .upsertEntry(
            const GlossaryEntryDraft(term: 'menu', translation: '菜单'),
          );
      expect(repository.entries['ui'], hasLength(2));

      await container
          .read(glossaryViewModelProvider.notifier)
          .deleteEntry('e1');
      expect(
        container.read(glossaryViewModelProvider).entries.single.term,
        'menu',
      );

      await container
          .read(glossaryViewModelProvider.notifier)
          .toggleBook(state.books.single);
      expect(
        repository.books.singleWhere((book) => book.id == 'ui').enabled,
        isFalse,
      );

      await container.read(glossaryViewModelProvider.notifier).deleteBook('ui');
      state = container.read(glossaryViewModelProvider);
      expect(state.books, isEmpty);
      expect(state.selectedBookId, isNull);
    },
  );

  test(
    'vocabulary filter, notes, and favorites reload through the view model',
    () async {
      final repository = _MemoryVocabulary()
        ..records.addAll(const [
          VocabularyRecord(
            id: '1',
            word: 'window',
            translation: '窗口',
            sourceLanguage: 'en',
            targetLanguage: 'zh-Hans',
            source: 'dictionary',
            favorite: false,
            createdAt: 0,
            updatedAt: 0,
          ),
          VocabularyRecord(
            id: '2',
            word: 'menu',
            translation: '菜单',
            sourceLanguage: 'en',
            targetLanguage: 'zh-Hans',
            source: 'dictionary',
            favorite: true,
            createdAt: 0,
            updatedAt: 0,
          ),
        ]);
      final container = _container(vocabulary: repository);
      addTearDown(container.dispose);
      final subscription = container.listen(
        vocabularyViewModelProvider,
        (_, _) {},
        fireImmediately: true,
      );
      addTearDown(subscription.close);
      await _waitFor(
        () => !container.read(vocabularyViewModelProvider).loading,
      );
      expect(container.read(vocabularyViewModelProvider).entries, hasLength(2));

      await container
          .read(vocabularyViewModelProvider.notifier)
          .setFilter(VocabularyFilter.favorites);
      expect(
        container.read(vocabularyViewModelProvider).entries.single.id,
        '2',
      );

      await container
          .read(vocabularyViewModelProvider.notifier)
          .setFavorite(repository.records.first, true);
      await container
          .read(vocabularyViewModelProvider.notifier)
          .updateNote(repository.records.first, 'desktop');
      expect(repository.records.first.favorite, isTrue);
      expect(repository.records.first.note, 'desktop');

      await container.read(vocabularyViewModelProvider.notifier).delete('2');
      expect(repository.records.map((entry) => entry.id), ['1']);
    },
  );

  test(
    'history selection lives in view state and clears after delete',
    () async {
      final repository = _MemoryHistory()
        ..records.add(_history('a'))
        ..records.add(_history('b'));
      final container = _container(history: repository);
      addTearDown(container.dispose);
      final subscription = container.listen(
        historyViewModelProvider,
        (_, _) {},
        fireImmediately: true,
      );
      addTearDown(subscription.close);
      await _waitFor(
        () => !container.read(historyViewModelProvider).snapshot.loading,
      );

      final notifier = container.read(historyViewModelProvider.notifier);
      notifier.toggleSelected('a');
      notifier.toggleSelected('b');
      notifier.toggleSelected('a');
      expect(container.read(historyViewModelProvider).selectedIds, {'b'});

      await notifier.deleteSelected();
      expect(repository.records.map((entry) => entry.id), ['a']);
      expect(container.read(historyViewModelProvider).selectedIds, isEmpty);
    },
  );
}

ProviderContainer _container({
  GlossaryRepository? glossary,
  VocabularyRepository? vocabulary,
  HistoryRepository? history,
}) {
  return ProviderContainer(
    overrides: [
      if (glossary != null)
        glossaryRepositoryProvider.overrideWithValue(glossary),
      if (vocabulary != null)
        vocabularyRepositoryProvider.overrideWithValue(vocabulary),
      if (history != null) historyRepositoryProvider.overrideWithValue(history),
    ],
  );
}

Future<void> _waitFor(bool Function() condition) async {
  for (var attempt = 0; attempt < 50; attempt++) {
    if (condition()) return;
    await Future<void>.delayed(const Duration(milliseconds: 1));
  }
  fail('Timed out waiting for library view-model state.');
}

HistoryRecord _history(String id) => HistoryRecord(
  id: id,
  source: id,
  translation: id,
  sourceLanguage: 'en',
  targetLanguage: 'zh-Hans',
  serviceId: 'deepl',
  serviceName: 'DeepL',
  favorite: false,
  edited: false,
  createdAt: 0,
  updatedAt: 0,
);

final class _MemoryGlossary implements GlossaryRepository {
  final books = <GlossaryBookRecord>[];
  final entries = <String, List<GlossaryEntryRecord>>{};

  @override
  Future<List<GlossaryBookRecord>> listBooks() async => List.of(books);

  @override
  Future<GlossaryBookRecord> upsertBook(GlossaryBookDraft draft) async {
    final book = GlossaryBookRecord(
      id: draft.id ?? 'new',
      name: draft.name,
      enabled: draft.enabled,
      entryCount: entries[draft.id ?? 'new']?.length ?? 0,
      sourceLanguage: draft.sourceLanguage,
      targetLanguage: draft.targetLanguage,
    );
    books.removeWhere((item) => item.id == book.id);
    books.add(book);
    return book;
  }

  @override
  Future<void> deleteBook(String bookId) async {
    books.removeWhere((book) => book.id == bookId);
    entries.remove(bookId);
  }

  @override
  Future<List<GlossaryEntryRecord>> listEntries({
    required String bookId,
    String query = '',
  }) async {
    final all = entries[bookId] ?? const [];
    if (query.trim().isEmpty) return List.of(all);
    return [
      for (final entry in all)
        if (entry.term.contains(query) || entry.translation.contains(query))
          entry,
    ];
  }

  @override
  Future<GlossaryEntryRecord> upsertEntry({
    required String bookId,
    required GlossaryEntryDraft draft,
  }) async {
    final entry = GlossaryEntryRecord(
      id: draft.id ?? 'e${(entries[bookId]?.length ?? 0) + 1}',
      term: draft.term,
      translation: draft.translation,
      forbidden: draft.forbidden,
      caseSensitive: draft.caseSensitive,
      wholeWord: draft.wholeWord,
      note: draft.note,
    );
    final current = [...?entries[bookId]];
    current.removeWhere((item) => item.id == entry.id);
    current.add(entry);
    entries[bookId] = current;
    return entry;
  }

  @override
  Future<void> deleteEntry({
    required String bookId,
    required String entryId,
  }) async {
    entries[bookId]?.removeWhere((entry) => entry.id == entryId);
  }

  @override
  Future<String> exportEntries({
    required String bookId,
    required GlossaryExchangeFormat format,
  }) async => '';

  @override
  Future<GlossaryImportSummary> importEntries({
    required String bookId,
    required String content,
    required GlossaryExchangeFormat format,
  }) async => const GlossaryImportSummary(inserted: 0, updated: 0, skipped: 0);

  @override
  Future<List<GlossaryMatchHit>> matchText({
    required String text,
    String? sourceLanguage,
    String? targetLanguage,
  }) async => const [];

  @override
  Future<List<GlossaryComplianceWarning>> checkCompliance({
    required String source,
    required String translated,
    String? sourceLanguage,
    String? targetLanguage,
  }) async => const [];
}

final class _MemoryVocabulary implements VocabularyRepository {
  final records = <VocabularyRecord>[];

  @override
  Future<VocabularySnapshot> load({
    VocabularyFilter filter = VocabularyFilter.all,
    String query = '',
  }) async {
    final filtered = [
      for (final entry in records)
        if ((filter == VocabularyFilter.all || entry.favorite) &&
            (query.isEmpty || entry.word.contains(query)))
          entry,
    ];
    return VocabularySnapshot(entries: filtered, filter: filter, query: query);
  }

  @override
  Future<VocabularyRecord> upsert(VocabularyDraft draft) async {
    throw UnimplementedError();
  }

  @override
  Future<VocabularyRecord?> setFavorite({
    required String entryId,
    required bool favorite,
  }) async {
    final index = records.indexWhere((entry) => entry.id == entryId);
    if (index < 0) return null;
    records[index] = VocabularyRecord(
      id: records[index].id,
      word: records[index].word,
      translation: records[index].translation,
      sourceLanguage: records[index].sourceLanguage,
      targetLanguage: records[index].targetLanguage,
      source: records[index].source,
      favorite: favorite,
      createdAt: records[index].createdAt,
      updatedAt: records[index].updatedAt,
      note: records[index].note,
    );
    return records[index];
  }

  @override
  Future<VocabularyRecord?> updateNote({
    required String entryId,
    String? note,
  }) async {
    final index = records.indexWhere((entry) => entry.id == entryId);
    if (index < 0) return null;
    records[index] = VocabularyRecord(
      id: records[index].id,
      word: records[index].word,
      translation: records[index].translation,
      sourceLanguage: records[index].sourceLanguage,
      targetLanguage: records[index].targetLanguage,
      source: records[index].source,
      favorite: records[index].favorite,
      createdAt: records[index].createdAt,
      updatedAt: records[index].updatedAt,
      note: note,
    );
    return records[index];
  }

  @override
  Future<int> delete(List<String> entryIds) async {
    records.removeWhere((entry) => entryIds.contains(entry.id));
    return entryIds.length;
  }
}

final class _MemoryHistory implements HistoryRepository {
  final records = <HistoryRecord>[];

  @override
  Future<HistorySnapshot> load({
    HistoryFilter filter = HistoryFilter.all,
    String query = '',
  }) async => HistorySnapshot(
    entries: List.of(records),
    counts: HistoryCounts(
      all: records.length,
      favorites: records.where((entry) => entry.favorite).length,
      edited: records.where((entry) => entry.edited).length,
    ),
    filter: filter,
    query: query,
  );

  @override
  Future<HistoryRecord> upsert(HistoryRecordDraft draft) async {
    throw UnimplementedError();
  }

  @override
  Future<HistoryRecord?> setFavorite({
    required String entryId,
    required bool favorite,
  }) async => null;

  @override
  Future<int> delete(List<String> entryIds) async {
    records.removeWhere((entry) => entryIds.contains(entry.id));
    return entryIds.length;
  }

  @override
  Future<int> clear() async {
    final count = records.length;
    records.clear();
    return count;
  }
}
