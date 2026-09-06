import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_runtime/linguaray_runtime.dart' as rt;

import '../../../../app/runtime.dart' as runtime_service;

final class RuntimeGlossaryRepository implements GlossaryRepository {
  RuntimeGlossaryRepository({rt.RuntimeGlossary? glossary})
    : _override = glossary;

  final rt.RuntimeGlossary? _override;

  rt.RuntimeGlossary get _glossary =>
      _override ?? runtime_service.runtime.glossary();

  @override
  Future<List<GlossaryComplianceWarning>> checkCompliance({
    required String source,
    required String translated,
    String? sourceLanguage,
    String? targetLanguage,
  }) async {
    try {
      final issues = await _glossary.check(
        source: source,
        translated: translated,
        sourceLanguage: sourceLanguage,
        targetLanguage: targetLanguage,
      );
      return [
        for (final issue in issues)
          GlossaryComplianceWarning(
            bookId: issue.bookId,
            entryId: issue.entryId,
            kind: issue.kind == rt.GlossaryIssueKind.forbiddenUsed
                ? GlossaryIssueKind.forbiddenUsed
                : GlossaryIssueKind.missingTranslation,
            term: issue.term,
            expected: issue.expected,
            found: issue.found,
          ),
      ];
    } catch (_) {
      return const [];
    }
  }

  @override
  Future<void> deleteBook(String bookId) async {
    await _glossary.deleteBook(bookId: bookId);
  }

  @override
  Future<void> deleteEntry({
    required String bookId,
    required String entryId,
  }) async {
    await _glossary.deleteEntry(bookId: bookId, entryId: entryId);
  }

  @override
  Future<String> exportEntries({
    required String bookId,
    required GlossaryExchangeFormat format,
  }) {
    return _glossary.exportEntries(
      bookId: bookId,
      format: switch (format) {
        GlossaryExchangeFormat.csv => rt.GlossaryExchangeFormat.csv,
        GlossaryExchangeFormat.tbx => rt.GlossaryExchangeFormat.tbx,
      },
    );
  }

  @override
  Future<GlossaryImportSummary> importEntries({
    required String bookId,
    required String content,
    required GlossaryExchangeFormat format,
  }) async {
    final report = await _glossary.importEntries(
      bookId: bookId,
      content: content,
      format: switch (format) {
        GlossaryExchangeFormat.csv => rt.GlossaryExchangeFormat.csv,
        GlossaryExchangeFormat.tbx => rt.GlossaryExchangeFormat.tbx,
      },
    );
    return GlossaryImportSummary(
      inserted: report.inserted,
      updated: report.updated,
      skipped: report.skipped,
    );
  }

  @override
  Future<List<GlossaryBookRecord>> listBooks() async {
    try {
      final books = await _glossary.listBooks();
      return [
        for (final book in books)
          GlossaryBookRecord(
            id: book.id,
            name: book.name,
            enabled: book.enabled,
            entryCount: book.entryCount,
            sourceLanguage: book.sourceLanguage,
            targetLanguage: book.targetLanguage,
          ),
      ];
    } catch (_) {
      return [
        const GlossaryBookRecord(
          id: 'corrupt',
          name: '',
          enabled: false,
          entryCount: 0,
          errorCode: 'glossary_corrupt',
        ),
      ];
    }
  }

  @override
  Future<List<GlossaryEntryRecord>> listEntries({
    required String bookId,
    String query = '',
  }) async {
    try {
      final entries = await _glossary.listEntries(
        bookId: bookId,
        query: query.trim().isEmpty ? null : query.trim(),
        offset: 0,
        limit: 0,
      );
      return [
        for (final entry in entries)
          GlossaryEntryRecord(
            id: entry.id,
            term: entry.term,
            translation: entry.translation,
            forbidden: entry.forbidden,
            caseSensitive: entry.caseSensitive,
            wholeWord: entry.wholeWord,
            note: entry.note,
            hits: entry.hits,
          ),
      ];
    } catch (_) {
      return const [];
    }
  }

  @override
  Future<List<GlossaryMatchHit>> matchText({
    required String text,
    String? sourceLanguage,
    String? targetLanguage,
  }) async {
    try {
      final matches = await _glossary.matchText(
        text: text,
        sourceLanguage: sourceLanguage,
        targetLanguage: targetLanguage,
      );
      return [
        for (final match in matches)
          GlossaryMatchHit(
            bookId: match.bookId,
            entryId: match.entryId,
            term: match.term,
            matchedText: match.matchedText,
            translation: match.translation,
            forbidden: match.forbidden,
            start: match.start,
            end: match.end,
          ),
      ];
    } catch (_) {
      return const [];
    }
  }

  @override
  Future<GlossaryBookRecord> upsertBook(GlossaryBookDraft draft) async {
    final book = await _glossary.upsertBook(
      input: rt.GlossaryBookInput(
        id: draft.id,
        name: draft.name,
        enabled: draft.enabled,
        sourceLanguage: draft.sourceLanguage,
        targetLanguage: draft.targetLanguage,
      ),
    );
    return GlossaryBookRecord(
      id: book.id,
      name: book.name,
      enabled: book.enabled,
      entryCount: book.entryCount,
      sourceLanguage: book.sourceLanguage,
      targetLanguage: book.targetLanguage,
    );
  }

  @override
  Future<GlossaryEntryRecord> upsertEntry({
    required String bookId,
    required GlossaryEntryDraft draft,
  }) async {
    final entry = await _glossary.upsertEntry(
      bookId: bookId,
      input: rt.GlossaryEntryInput(
        id: draft.id,
        term: draft.term,
        translation: draft.translation,
        forbidden: draft.forbidden,
        note: draft.note,
        caseSensitive: draft.caseSensitive,
        wholeWord: draft.wholeWord,
      ),
    );
    return GlossaryEntryRecord(
      id: entry.id,
      term: entry.term,
      translation: entry.translation,
      forbidden: entry.forbidden,
      caseSensitive: entry.caseSensitive,
      wholeWord: entry.wholeWord,
      note: entry.note,
      hits: entry.hits,
    );
  }
}
