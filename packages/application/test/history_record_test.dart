import 'package:linguaray_application/linguaray_application.dart';
import 'package:test/test.dart';

void main() {
  test(
    'records completed results once per service and skips failures',
    () async {
      final repository = _FakeHistoryRepository();
      final useCase = RecordCompletedTranslation(repository);
      const deepl = TranslationServiceOption(
        id: 'deepl',
        name: 'DeepL',
        isStreaming: false,
      );
      const openai = TranslationServiceOption(
        id: 'openai',
        name: 'OpenAI',
        isStreaming: true,
      );

      await useCase(
        sessionId: 'session-1',
        run: const TranslationRun(
          sourceText: 'hello',
          sourceLanguage: 'auto',
          targetLanguage: 'zh-Hans',
          detectedLanguage: 'en',
          complete: true,
          results: [
            ServiceTranslationResult(
              service: deepl,
              text: '你好',
              status: TranslationResultStatus.completed,
            ),
            ServiceTranslationResult(
              service: openai,
              status: TranslationResultStatus.failed,
              errorCode: 'network_failure',
            ),
          ],
        ),
      );

      expect(repository.drafts, hasLength(1));
      expect(repository.drafts.single.id, 'session-1:deepl');
      expect(repository.drafts.single.translation, '你好');

      await useCase(
        sessionId: 'session-1',
        run: const TranslationRun(
          sourceText: 'hello',
          sourceLanguage: 'auto',
          targetLanguage: 'zh-Hans',
          detectedLanguage: 'en',
          complete: true,
          results: [
            ServiceTranslationResult(
              service: deepl,
              text: '你好。',
              status: TranslationResultStatus.completed,
            ),
          ],
        ),
      );

      expect(repository.drafts, hasLength(1));
      expect(repository.drafts.single.translation, '你好。');
    },
  );

  test('does not persist incomplete streaming snapshots', () async {
    final repository = _FakeHistoryRepository();
    await RecordCompletedTranslation(repository)(
      sessionId: 'session-2',
      run: const TranslationRun(
        sourceText: 'hello',
        sourceLanguage: 'en',
        targetLanguage: 'zh-Hans',
        complete: false,
        results: [
          ServiceTranslationResult(
            service: TranslationServiceOption(
              id: 'openai',
              name: 'OpenAI',
              isStreaming: true,
            ),
            text: '你',
            status: TranslationResultStatus.translating,
          ),
        ],
      ),
    );
    expect(repository.drafts, isEmpty);
  });
}

final class _FakeHistoryRepository implements HistoryRepository {
  final List<HistoryRecordDraft> drafts = [];

  @override
  Future<int> clear() async => 0;

  @override
  Future<int> delete(List<String> entryIds) async => entryIds.length;

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
    drafts.removeWhere((item) => item.id == draft.id);
    drafts.add(draft);
    return HistoryRecord(
      id: draft.id ?? draft.serviceId,
      source: draft.source,
      translation: draft.translation,
      sourceLanguage: draft.sourceLanguage,
      targetLanguage: draft.targetLanguage,
      serviceId: draft.serviceId,
      serviceName: draft.serviceName,
      favorite: false,
      edited: draft.edited,
      createdAt: 0,
      updatedAt: 0,
    );
  }
}
