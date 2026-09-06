import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/features/library/glossary/data/runtime_glossary_repository.dart';
import 'package:linguaray_runtime/linguaray_runtime.dart' as rt;

void main() {
  late Directory dataDirectory;
  late RuntimeGlossaryRepository repository;

  setUp(() async {
    dataDirectory = await Directory.systemTemp.createTemp(
      'linguaray-glossary-repository-',
    );
    final runtime = rt.Runtime(dataDir: dataDirectory.path);
    repository = RuntimeGlossaryRepository(glossary: runtime.glossary());
  });

  tearDown(() async {
    if (dataDirectory.existsSync()) {
      await dataDirectory.delete(recursive: true);
    }
  });

  test('maps glossary books, entries, matches, and compliance', () async {
    final book = await repository.upsertBook(
      const GlossaryBookDraft(
        name: 'UI terms',
        enabled: true,
        sourceLanguage: 'en',
        targetLanguage: 'zh-Hans',
      ),
    );
    final entry = await repository.upsertEntry(
      bookId: book.id,
      draft: const GlossaryEntryDraft(
        term: 'window',
        translation: '窗口',
        forbidden: ['窗体'],
        caseSensitive: false,
        wholeWord: true,
      ),
    );

    final books = await repository.listBooks();
    expect(books.single.id, book.id);
    expect(books.single.entryCount, 1);

    final entries = await repository.listEntries(
      bookId: book.id,
      query: 'window',
    );
    expect(entries.single.id, entry.id);
    expect(entries.single.translation, '窗口');

    final matches = await repository.matchText(
      text: 'Open the window.',
      sourceLanguage: 'en',
      targetLanguage: 'zh-Hans',
    );
    expect(matches.single.term, 'window');

    final warnings = await repository.checkCompliance(
      source: 'Open the window.',
      translated: '打开窗体。',
      sourceLanguage: 'en',
      targetLanguage: 'zh-Hans',
    );
    expect(
      warnings.map((warning) => warning.kind),
      containsAll([
        GlossaryIssueKind.missingTranslation,
        GlossaryIssueKind.forbiddenUsed,
      ]),
    );
  });
}
