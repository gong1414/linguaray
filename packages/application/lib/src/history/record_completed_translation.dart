import 'package:linguaray_application/src/history/models.dart';
import 'package:linguaray_application/src/history/ports.dart';
import 'package:linguaray_application/src/translation/models.dart';

/// Persists completed translation results once per service in a session.
///
/// Streaming chunks and retries reuse [sessionId], so the store updates the
/// same rows instead of duplicating them. Failed results are never written.
final class RecordCompletedTranslation {
  const RecordCompletedTranslation(this._repository);

  final HistoryRepository _repository;

  Future<List<HistoryRecord>> call({
    required String sessionId,
    required TranslationRun run,
  }) async {
    if (!run.complete) return const [];
    final source = run.sourceText.trim();
    if (source.isEmpty) return const [];

    final saved = <HistoryRecord>[];
    for (final result in run.results) {
      if (result.status != TranslationResultStatus.completed) continue;
      if (!result.hasText) continue;
      saved.add(
        await _repository.upsert(
          HistoryRecordDraft(
            id: '$sessionId:${result.service.id}',
            source: source,
            translation: result.text,
            sourceLanguage: run.detectedLanguage ?? run.sourceLanguage,
            targetLanguage: run.targetLanguage,
            serviceId: result.service.id,
            serviceName: result.service.name,
          ),
        ),
      );
    }
    return saved;
  }
}
