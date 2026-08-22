import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/config/dependencies.dart';
import 'package:linguaray_desktop/src/ui/quick_translate/view_models/quick_translate_view_model.dart';
import 'package:linguaray_desktop/src/ui/translation/view_models/translation_view_model.dart';

void main() {
  test('loads catalog and exposes only application models', () async {
    final container = ProviderContainer(
      overrides: [
        translationRepositoryProvider.overrideWithValue(
          _FakeTranslationRepository(),
        ),
        historyRepositoryProvider.overrideWithValue(_FakeHistoryRepository()),
        glossaryRepositoryProvider.overrideWithValue(_FakeGlossaryRepository()),
      ],
    );
    addTearDown(container.dispose);
    final subscription = container.listen(
      translationViewModelProvider,
      (_, _) {},
      fireImmediately: true,
    );
    addTearDown(subscription.close);

    await _waitFor(
      () => !container.read(translationViewModelProvider).loadingCatalog,
    );

    final state = container.read(translationViewModelProvider);
    expect(state.catalog, isA<TranslationCatalog>());
    expect(state.languages.single.name, 'English');
    expect(state.services.single.name, 'Local stub');
    expect(state.selectedServiceId, 'stub');
    expect(state.targetLanguage, automaticTargetCode);
  });

  test('submits through the port and publishes completed result', () async {
    final container = ProviderContainer(
      overrides: [
        translationRepositoryProvider.overrideWithValue(
          _FakeTranslationRepository(),
        ),
        historyRepositoryProvider.overrideWithValue(_FakeHistoryRepository()),
        glossaryRepositoryProvider.overrideWithValue(_FakeGlossaryRepository()),
      ],
    );
    addTearDown(container.dispose);
    final subscription = container.listen(
      translationViewModelProvider,
      (_, _) {},
      fireImmediately: true,
    );
    addTearDown(subscription.close);
    await _waitFor(
      () => !container.read(translationViewModelProvider).loadingCatalog,
    );

    final viewModel = container.read(translationViewModelProvider.notifier);
    viewModel.setSourceText('Hello');
    await viewModel.submit();

    final state = container.read(translationViewModelProvider);
    expect(state.submitting, isFalse);
    expect(state.run?.detectedLanguage, 'en');
    expect(state.run?.targetLanguage, 'zh-Hans');
    expect(state.selectedResult?.text, '你好');
    expect(state.selectedResult?.status, TranslationResultStatus.completed);
  });

  test('separate workbench requests keep separate history rows', () async {
    final history = _FakeHistoryRepository();
    final container = ProviderContainer(
      overrides: [
        translationRepositoryProvider.overrideWithValue(
          _FakeTranslationRepository(),
        ),
        historyRepositoryProvider.overrideWithValue(history),
        glossaryRepositoryProvider.overrideWithValue(_FakeGlossaryRepository()),
      ],
    );
    addTearDown(container.dispose);
    final subscription = container.listen(
      translationViewModelProvider,
      (_, _) {},
      fireImmediately: true,
    );
    addTearDown(subscription.close);
    await _waitFor(
      () => !container.read(translationViewModelProvider).loadingCatalog,
    );

    final viewModel = container.read(translationViewModelProvider.notifier);
    viewModel.setSourceText('First');
    await viewModel.submit();
    viewModel.setSourceText('Second');
    await viewModel.submit();
    await _waitFor(() => history.entries.length == 2);

    expect(history.entries.map((entry) => entry.id).toSet(), hasLength(2));
  });

  test('quick translator records completed translations', () async {
    final history = _FakeHistoryRepository();
    final container = ProviderContainer(
      overrides: [
        translationRepositoryProvider.overrideWithValue(
          _FakeTranslationRepository(),
        ),
        historyRepositoryProvider.overrideWithValue(history),
      ],
    );
    addTearDown(container.dispose);
    final subscription = container.listen(
      quickTranslateViewModelProvider,
      (_, _) {},
      fireImmediately: true,
    );
    addTearDown(subscription.close);
    await _waitFor(
      () => !container.read(quickTranslateViewModelProvider).loadingCatalog,
    );

    final viewModel = container.read(quickTranslateViewModelProvider.notifier);
    viewModel.setSourceText('Hello');
    await viewModel.submit();
    await _waitFor(() => history.entries.length == 1);

    expect(history.entries.single.source, 'Hello');
    expect(history.entries.single.translation, '你好');
  });
}

Future<void> _waitFor(bool Function() condition) async {
  for (var attempt = 0; attempt < 50; attempt++) {
    if (condition()) return;
    await Future<void>.delayed(const Duration(milliseconds: 1));
  }
  fail('Timed out waiting for view-model state.');
}

final class _FakeTranslationRepository implements TranslationRepository {
  @override
  Future<TranslationCatalog> loadCatalog() async => const TranslationCatalog(
    languages: [LanguageOption(code: 'en', name: 'English')],
    services: [
      TranslationServiceOption(
        id: 'stub',
        name: 'Local stub',
        isStreaming: false,
      ),
    ],
    defaultSourceLanguage: autoLanguageCode,
    defaultTargetLanguage: 'zh-Hans',
  );

  @override
  Future<String?> detectLanguage({
    required String serviceId,
    required String text,
  }) async => 'en';

  @override
  Future<String> resolveTarget({
    required String? selectedTarget,
    required String fallbackTarget,
    required String? detectedLanguage,
  }) async => selectedTarget ?? fallbackTarget;

  @override
  Stream<String> translate({
    required TranslationServiceOption service,
    required String text,
    required String sourceLanguage,
    required String targetLanguage,
  }) async* {
    yield '你好';
  }
}

final class _FakeHistoryRepository implements HistoryRepository {
  final List<HistoryRecordDraft> entries = [];

  @override
  Future<int> clear() async => 0;

  @override
  Future<int> delete(List<String> entryIds) async => 0;

  @override
  Future<HistorySnapshot> load({
    HistoryFilter filter = HistoryFilter.all,
    String query = '',
  }) async => const HistorySnapshot.empty();

  @override
  Future<HistoryRecord?> setFavorite({
    required String entryId,
    required bool favorite,
  }) async => null;

  @override
  Future<HistoryRecord> upsert(HistoryRecordDraft draft) async {
    entries
      ..removeWhere((entry) => entry.id == draft.id)
      ..add(draft);
    return HistoryRecord(
      id: draft.id ?? '1',
      source: draft.source,
      translation: draft.translation,
      sourceLanguage: draft.sourceLanguage,
      targetLanguage: draft.targetLanguage,
      serviceId: draft.serviceId,
      serviceName: draft.serviceName,
      favorite: false,
      edited: false,
      createdAt: 0,
      updatedAt: 0,
    );
  }
}

final class _FakeGlossaryRepository implements GlossaryRepository {
  @override
  Future<List<GlossaryComplianceWarning>> checkCompliance({
    required String source,
    required String translated,
    String? sourceLanguage,
    String? targetLanguage,
  }) async => const [];

  @override
  Future<void> deleteBook(String bookId) async {}

  @override
  Future<void> deleteEntry({
    required String bookId,
    required String entryId,
  }) async {}

  @override
  Future<List<GlossaryBookRecord>> listBooks() async => const [];

  @override
  Future<List<GlossaryEntryRecord>> listEntries({
    required String bookId,
    String query = '',
  }) async => const [];

  @override
  Future<List<GlossaryMatchHit>> matchText({
    required String text,
    String? sourceLanguage,
    String? targetLanguage,
  }) async => const [];

  @override
  Future<GlossaryBookRecord> upsertBook(GlossaryBookDraft draft) async {
    return GlossaryBookRecord(
      id: draft.id ?? 'book',
      name: draft.name,
      enabled: draft.enabled,
      entryCount: 0,
    );
  }

  @override
  Future<GlossaryEntryRecord> upsertEntry({
    required String bookId,
    required GlossaryEntryDraft draft,
  }) async {
    return GlossaryEntryRecord(
      id: draft.id ?? 'entry',
      term: draft.term,
      translation: draft.translation,
      forbidden: draft.forbidden,
      caseSensitive: draft.caseSensitive,
      wholeWord: draft.wholeWord,
    );
  }
}
