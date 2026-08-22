final class GlossaryBookRecord {
  const GlossaryBookRecord({
    required this.id,
    required this.name,
    required this.enabled,
    required this.entryCount,
    this.sourceLanguage,
    this.targetLanguage,
    this.errorCode,
  });

  final String id;
  final String name;
  final bool enabled;
  final int entryCount;
  final String? sourceLanguage;
  final String? targetLanguage;

  /// Set when this book file could not be read. Translation stays usable.
  final String? errorCode;
}

final class GlossaryBookDraft {
  const GlossaryBookDraft({
    this.id,
    required this.name,
    this.enabled = true,
    this.sourceLanguage,
    this.targetLanguage,
  });

  final String? id;
  final String name;
  final bool enabled;
  final String? sourceLanguage;
  final String? targetLanguage;
}

final class GlossaryEntryRecord {
  const GlossaryEntryRecord({
    required this.id,
    required this.term,
    required this.translation,
    required this.forbidden,
    required this.caseSensitive,
    required this.wholeWord,
    this.note,
    this.hits = 0,
  });

  final String id;
  final String term;
  final String translation;
  final List<String> forbidden;
  final bool caseSensitive;
  final bool wholeWord;
  final String? note;
  final int hits;
}

final class GlossaryEntryDraft {
  const GlossaryEntryDraft({
    this.id,
    required this.term,
    required this.translation,
    this.forbidden = const [],
    this.note,
    this.caseSensitive = false,
    this.wholeWord = true,
  });

  final String? id;
  final String term;
  final String translation;
  final List<String> forbidden;
  final String? note;
  final bool caseSensitive;
  final bool wholeWord;
}

final class GlossaryMatchHit {
  const GlossaryMatchHit({
    required this.bookId,
    required this.entryId,
    required this.term,
    required this.matchedText,
    required this.translation,
    required this.forbidden,
    required this.start,
    required this.end,
  });

  final String bookId;
  final String entryId;
  final String term;
  final String matchedText;
  final String translation;
  final List<String> forbidden;
  final int start;
  final int end;
}

enum GlossaryIssueKind { missingTranslation, forbiddenUsed }

final class GlossaryComplianceWarning {
  const GlossaryComplianceWarning({
    required this.bookId,
    required this.entryId,
    required this.kind,
    required this.term,
    required this.expected,
    this.found,
  });

  final String bookId;
  final String entryId;
  final GlossaryIssueKind kind;
  final String term;
  final String expected;
  final String? found;
}
