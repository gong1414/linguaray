import 'package:linguaray_application/src/glossary/models.dart';

abstract interface class GlossaryRepository {
  Future<List<GlossaryBookRecord>> listBooks();

  Future<GlossaryBookRecord> upsertBook(GlossaryBookDraft draft);

  Future<void> deleteBook(String bookId);

  Future<List<GlossaryEntryRecord>> listEntries({
    required String bookId,
    String query = '',
  });

  Future<GlossaryEntryRecord> upsertEntry({
    required String bookId,
    required GlossaryEntryDraft draft,
  });

  Future<void> deleteEntry({required String bookId, required String entryId});

  Future<String> exportEntries({
    required String bookId,
    required GlossaryExchangeFormat format,
  });

  Future<GlossaryImportSummary> importEntries({
    required String bookId,
    required String content,
    required GlossaryExchangeFormat format,
  });

  Future<List<GlossaryMatchHit>> matchText({
    required String text,
    String? sourceLanguage,
    String? targetLanguage,
  });

  Future<List<GlossaryComplianceWarning>> checkCompliance({
    required String source,
    required String translated,
    String? sourceLanguage,
    String? targetLanguage,
  });
}
