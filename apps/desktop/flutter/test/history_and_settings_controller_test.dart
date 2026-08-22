import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';

void main() {
  test(
    'completed translation recording skips failures and retries in place',
    () async {
      final repository = _MemoryHistory();
      final useCase = RecordCompletedTranslation(repository);
      const service = TranslationServiceOption(
        id: 'deepl',
        name: 'DeepL',
        isStreaming: false,
      );
      const run = TranslationRun(
        sourceText: 'hello',
        sourceLanguage: 'en',
        targetLanguage: 'zh-Hans',
        complete: true,
        results: [
          ServiceTranslationResult(
            service: service,
            text: '你好',
            status: TranslationResultStatus.completed,
          ),
          ServiceTranslationResult(
            service: TranslationServiceOption(
              id: 'bad',
              name: 'Bad',
              isStreaming: false,
            ),
            status: TranslationResultStatus.failed,
            errorCode: 'network_failure',
          ),
        ],
      );

      await useCase(sessionId: 's1', run: run);
      expect(repository.entries, hasLength(1));
      await useCase(
        sessionId: 's1',
        run: const TranslationRun(
          sourceText: 'hello',
          sourceLanguage: 'en',
          targetLanguage: 'zh-Hans',
          complete: true,
          results: [
            ServiceTranslationResult(
              service: service,
              text: '你好！',
              status: TranslationResultStatus.completed,
            ),
          ],
        ),
      );
      expect(repository.entries, hasLength(1));
      expect(repository.entries.single.translation, '你好！');
    },
  );

  test('settings errors stay in view-model shaped state', () {
    const state = GeneralPreferences(
      launchAtLogin: false,
      showInMenuBar: true,
      language: 'en',
      themeMode: ThemePreference.light,
      inputSubmitMode: InputSubmitMode.enter,
      autoCopyDetectedText: true,
      doubleClickCopyResult: true,
    );
    expect(state.inputSubmitMode, InputSubmitMode.enter);
    expect(state.autoCopyDetectedText, isTrue);
  });
}

final class _MemoryHistory implements HistoryRepository {
  final List<HistoryRecordDraft> entries = [];

  @override
  Future<int> clear() async {
    final count = entries.length;
    entries.clear();
    return count;
  }

  @override
  Future<int> delete(List<String> entryIds) async {
    entries.removeWhere((item) => entryIds.contains(item.id));
    return entryIds.length;
  }

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
    entries.removeWhere((item) => item.id == draft.id);
    entries.add(draft);
    return HistoryRecord(
      id: draft.id ?? 'x',
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
