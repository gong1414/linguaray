import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../app/dependencies.dart';
import '../../../i18n/i18n.dart';
import '../../../shared/i18n_labels.dart';
import '../../../shared/settings_page.dart';
import '../../../shared/status_message.dart';
import 'glossary_dialogs.dart';

class GlossarySettingsScreen extends ConsumerStatefulWidget {
  const GlossarySettingsScreen({super.key});

  @override
  ConsumerState<GlossarySettingsScreen> createState() =>
      _GlossarySettingsScreenState();
}

class _GlossarySettingsScreenState
    extends ConsumerState<GlossarySettingsScreen> {
  List<GlossaryBookRecord> _books = const [];
  List<GlossaryEntryRecord> _entries = const [];
  String? _selectedBookId;
  String _query = '';
  bool _loading = true;
  String? _errorCode;

  GlossaryRepository get _repository => ref.read(glossaryRepositoryProvider);

  @override
  void initState() {
    super.initState();
    unawaited(_reloadBooks());
  }

  Future<void> _reloadBooks({String? select}) async {
    if (mounted) setState(() => _loading = true);
    try {
      final loadedBooks = await _repository.listBooks();
      final books = loadedBooks
          .where((book) => book.errorCode == null)
          .toList(growable: false);
      final requested = select ?? _selectedBookId;
      final selected = books.any((book) => book.id == requested)
          ? requested
          : books.firstOrNull?.id;
      if (!mounted) return;
      setState(() {
        _books = books;
        _selectedBookId = selected;
        _errorCode = loadedBooks.any((book) => book.errorCode != null)
            ? AppErrorCode.glossaryCorrupt.wireName
            : null;
      });
      await _reloadEntries();
    } catch (_) {
      if (!mounted) return;
      setState(() {
        _loading = false;
        _errorCode = AppErrorCode.glossaryCorrupt.wireName;
      });
    }
  }

  Future<void> _reloadEntries() async {
    final bookId = _selectedBookId;
    if (bookId == null) {
      if (mounted) {
        setState(() {
          _entries = const [];
          _loading = false;
        });
      }
      return;
    }
    final entries = await _repository.listEntries(
      bookId: bookId,
      query: _query,
    );
    if (!mounted || bookId != _selectedBookId) return;
    setState(() {
      _entries = entries;
      _loading = false;
    });
  }

  Future<void> _editBook([GlossaryBookRecord? book]) async {
    List<LanguageOption> languages = const [];
    try {
      languages = (await ref.read(loadTranslationCatalogProvider)()).languages;
    } catch (_) {
      // Language scoping remains optional if the catalog is temporarily down.
    }
    if (!mounted) return;
    final draft = await showDialog<GlossaryBookDraft>(
      context: context,
      builder: (context) =>
          GlossaryBookDialog(book: book, languages: languages),
    );
    if (draft == null) return;
    final saved = await _repository.upsertBook(draft);
    await _reloadBooks(select: saved.id);
  }

  Future<void> _deleteBook(GlossaryBookRecord book) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(t.common.ui.button.delete),
        content: Text(
          t.workbench.glossary_page.delete_book_confirm(
            name: book.name,
            count: book.entryCount,
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: Text(t.common.ui.button.cancel),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(t.common.ui.button.delete),
          ),
        ],
      ),
    );
    if (confirmed != true) return;
    await _repository.deleteBook(book.id);
    await _reloadBooks();
  }

  Future<void> _editEntry([GlossaryEntryRecord? entry]) async {
    final bookId = _selectedBookId;
    if (bookId == null) return;
    final draft = await showDialog<GlossaryEntryDraft>(
      context: context,
      builder: (context) => GlossaryEntryDialog(entry: entry),
    );
    if (draft == null) return;
    await _repository.upsertEntry(bookId: bookId, draft: draft);
    await _reloadEntries();
  }

  Future<void> _importGlossary(
    GlossaryBookRecord book,
    GlossaryExchangeFormat format,
  ) async {
    try {
      final report = await ref
          .read(glossaryExchangeControllerProvider)
          .importBook(book.id, format);
      if (report == null) return;
      await _reloadBooks(select: book.id);
      if (!mounted) return;
      await _showExchangeMessage(
        t.workbench.glossary_page.import_success(
          inserted: report.inserted,
          updated: report.updated,
          skipped: report.skipped,
        ),
      );
    } catch (_) {
      if (mounted) {
        await _showExchangeMessage(t.workbench.glossary_page.import_failed);
      }
    }
  }

  Future<void> _exportGlossary(
    GlossaryBookRecord book,
    GlossaryExchangeFormat format,
  ) async {
    try {
      final saved = await ref
          .read(glossaryExchangeControllerProvider)
          .exportBook(book, format);
      if (!saved) return;
      if (mounted) {
        await _showExchangeMessage(t.workbench.glossary_page.export_success);
      }
    } catch (_) {
      if (mounted) {
        await _showExchangeMessage(t.workbench.glossary_page.export_failed);
      }
    }
  }

  Future<void> _showExchangeMessage(String message) {
    return showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        content: Text(message),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(t.common.ui.button.ok),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final page = t.workbench.glossary_page;
    final selectedBook = _books
        .where((book) => book.id == _selectedBookId)
        .firstOrNull;
    return SettingsPage(
      title: t.ui.shell.glossary,
      actions: [
        PopupMenuButton<GlossaryExchangeFormat>(
          enabled: selectedBook != null,
          tooltip: page.import_file,
          icon: const Icon(Icons.file_upload_outlined),
          onSelected: (format) =>
              unawaited(_importGlossary(selectedBook!, format)),
          itemBuilder: (_) => [
            PopupMenuItem(
              value: GlossaryExchangeFormat.csv,
              child: Text('${page.import_file} · CSV'),
            ),
            PopupMenuItem(
              value: GlossaryExchangeFormat.tbx,
              child: Text('${page.import_file} · TBX'),
            ),
          ],
        ),
        PopupMenuButton<GlossaryExchangeFormat>(
          enabled: selectedBook != null,
          tooltip: page.export_file,
          icon: const Icon(Icons.file_download_outlined),
          onSelected: (format) =>
              unawaited(_exportGlossary(selectedBook!, format)),
          itemBuilder: (_) => [
            PopupMenuItem(
              value: GlossaryExchangeFormat.csv,
              child: Text('${page.export_file} · CSV'),
            ),
            PopupMenuItem(
              value: GlossaryExchangeFormat.tbx,
              child: Text('${page.export_file} · TBX'),
            ),
          ],
        ),
        OutlinedButton.icon(
          onPressed: () => unawaited(_editBook()),
          icon: const Icon(Icons.create_new_folder_outlined, size: 18),
          label: Text(page.new_book),
        ),
        FilledButton.icon(
          onPressed: selectedBook == null
              ? null
              : () => unawaited(_editEntry()),
          icon: const Icon(Icons.add_rounded, size: 18),
          label: Text(page.add_entry),
        ),
      ],
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (_errorCode != null)
            Padding(
              padding: const EdgeInsets.only(bottom: 16),
              child: StatusMessage(
                kind: StatusKind.warning,
                title: appErrorMessage(_errorCode),
              ),
            ),
          Expanded(
            child: _books.isEmpty
                ? StatusMessage(
                    title: page.no_books_title,
                    body: page.no_books_description,
                  )
                : Row(
                    children: [
                      SizedBox(
                        width: 190,
                        child: _books.isEmpty
                            ? Center(child: Text(page.no_books_title))
                            : ListView(
                                padding: const EdgeInsets.only(right: 12),
                                children: [
                                  for (final book in _books)
                                    ListTile(
                                      dense: true,
                                      selected: book.id == _selectedBookId,
                                      title: Text(book.name),
                                      subtitle: Text('${book.entryCount}'),
                                      onTap: () {
                                        setState(
                                          () => _selectedBookId = book.id,
                                        );
                                        unawaited(_reloadEntries());
                                      },
                                      trailing: PopupMenuButton<String>(
                                        onSelected: (action) async {
                                          if (action == 'rename') {
                                            await _editBook(book);
                                          } else if (action == 'toggle') {
                                            await _repository.upsertBook(
                                              GlossaryBookDraft(
                                                id: book.id,
                                                name: book.name,
                                                enabled: !book.enabled,
                                                sourceLanguage:
                                                    book.sourceLanguage,
                                                targetLanguage:
                                                    book.targetLanguage,
                                              ),
                                            );
                                            await _reloadBooks(select: book.id);
                                          } else if (action == 'delete') {
                                            await _deleteBook(book);
                                          }
                                        },
                                        itemBuilder: (_) => [
                                          PopupMenuItem(
                                            value: 'rename',
                                            child: Text(page.rename_book),
                                          ),
                                          PopupMenuItem(
                                            value: 'toggle',
                                            child: Text(
                                              book.enabled
                                                  ? page.disable
                                                  : page.enable,
                                            ),
                                          ),
                                          PopupMenuItem(
                                            value: 'delete',
                                            child: Text(
                                              t.common.ui.button.delete,
                                            ),
                                          ),
                                        ],
                                      ),
                                    ),
                                ],
                              ),
                      ),
                      const VerticalDivider(width: 1),
                      Expanded(
                        child: selectedBook == null
                            ? Center(child: Text(page.no_books_description))
                            : Column(
                                children: [
                                  Padding(
                                    padding: const EdgeInsets.fromLTRB(
                                      20,
                                      0,
                                      0,
                                      20,
                                    ),
                                    child: SearchBar(
                                      hintText: page.search_placeholder,
                                      leading: const Icon(
                                        Icons.search_rounded,
                                        size: 18,
                                      ),
                                      onChanged: (value) {
                                        _query = value;
                                        unawaited(_reloadEntries());
                                      },
                                    ),
                                  ),
                                  Expanded(child: _entryBody(selectedBook)),
                                ],
                              ),
                      ),
                    ],
                  ),
          ),
        ],
      ),
    );
  }

  Widget _entryBody(GlossaryBookRecord book) {
    final page = t.workbench.glossary_page;
    if (_loading) return const Center(child: CircularProgressIndicator());
    if (_entries.isEmpty) {
      return Padding(
        padding: const EdgeInsets.only(left: 20),
        child: StatusMessage(
          title: _query.isEmpty
              ? page.empty_title
              : page.no_results_title(query: _query),
        ),
      );
    }
    return ListView.separated(
      padding: const EdgeInsets.only(left: 12),
      itemCount: _entries.length,
      separatorBuilder: (_, _) => const Divider(height: 1),
      itemBuilder: (context, index) {
        final entry = _entries[index];
        return ListTile(
          title: Text(entry.term),
          subtitle: Text(entry.translation),
          onTap: () => unawaited(_editEntry(entry)),
          trailing: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (entry.hits > 0) Text('${page.hits} ${entry.hits}'),
              IconButton(
                tooltip: t.common.ui.button.delete,
                onPressed: () async {
                  await _repository.deleteEntry(
                    bookId: book.id,
                    entryId: entry.id,
                  );
                  await _reloadEntries();
                },
                icon: const Icon(Icons.delete_outline_rounded),
              ),
            ],
          ),
        );
      },
    );
  }
}
