import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../config/dependencies.dart';
import '../../i18n/i18n.dart';
import 'glossary_view.dart';

final glossaryViewModelProvider =
    NotifierProvider<GlossaryViewModel, GlossaryViewState>(
      GlossaryViewModel.new,
    );

final class GlossaryViewState {
  const GlossaryViewState({
    this.books = const [],
    this.entries = const [],
    this.selectedBookId,
    this.query = '',
    this.loading = true,
    this.errorCode,
  });

  final List<GlossaryBookRecord> books;
  final List<GlossaryEntryRecord> entries;
  final String? selectedBookId;
  final String query;
  final bool loading;
  final String? errorCode;
}

final class GlossaryViewModel extends Notifier<GlossaryViewState> {
  @override
  GlossaryViewState build() {
    scheduleMicrotask(reload);
    return const GlossaryViewState();
  }

  Future<void> reload() async {
    final repository = ref.read(glossaryRepositoryProvider);
    final books = await repository.listBooks();
    final selected =
        state.selectedBookId ?? (books.isEmpty ? null : books.first.id);
    final entries = selected == null
        ? const <GlossaryEntryRecord>[]
        : await repository.listEntries(bookId: selected, query: state.query);
    state = GlossaryViewState(
      books: books,
      entries: entries,
      selectedBookId: selected,
      query: state.query,
      loading: false,
      errorCode: books.any((book) => book.errorCode != null)
          ? AppErrorCode.glossaryCorrupt.wireName
          : null,
    );
  }

  Future<void> selectBook(String id) async {
    state = GlossaryViewState(
      books: state.books,
      entries: state.entries,
      selectedBookId: id,
      query: '',
      loading: true,
    );
    await reload();
  }

  Future<void> setQuery(String query) async {
    state = GlossaryViewState(
      books: state.books,
      entries: state.entries,
      selectedBookId: state.selectedBookId,
      query: query,
      loading: true,
    );
    await reload();
  }

  Future<void> createBook(String name) async {
    await ref
        .read(glossaryRepositoryProvider)
        .upsertBook(GlossaryBookDraft(name: name));
    await reload();
  }

  Future<void> renameBook(String name) async {
    final id = state.selectedBookId;
    if (id == null) return;
    final current = state.books.where((book) => book.id == id).firstOrNull;
    if (current == null) return;
    await ref
        .read(glossaryRepositoryProvider)
        .upsertBook(
          GlossaryBookDraft(
            id: current.id,
            name: name,
            enabled: current.enabled,
            sourceLanguage: current.sourceLanguage,
            targetLanguage: current.targetLanguage,
          ),
        );
    await reload();
  }

  Future<void> toggleBook() async {
    final id = state.selectedBookId;
    if (id == null) return;
    final current = state.books.where((book) => book.id == id).firstOrNull;
    if (current == null) return;
    await ref
        .read(glossaryRepositoryProvider)
        .upsertBook(
          GlossaryBookDraft(
            id: current.id,
            name: current.name,
            enabled: !current.enabled,
            sourceLanguage: current.sourceLanguage,
            targetLanguage: current.targetLanguage,
          ),
        );
    await reload();
  }

  Future<void> deleteBook() async {
    final id = state.selectedBookId;
    if (id == null) return;
    await ref.read(glossaryRepositoryProvider).deleteBook(id);
    state = GlossaryViewState(books: state.books, selectedBookId: null);
    await reload();
  }

  Future<void> saveEntry(GlossaryEntryDraft draft) async {
    final id = state.selectedBookId;
    if (id == null) return;
    await ref
        .read(glossaryRepositoryProvider)
        .upsertEntry(bookId: id, draft: draft);
    await reload();
  }

  Future<void> deleteEntry(String entryId) async {
    final id = state.selectedBookId;
    if (id == null) return;
    await ref
        .read(glossaryRepositoryProvider)
        .deleteEntry(bookId: id, entryId: entryId);
    await reload();
  }
}

class GlossaryScreen extends ConsumerWidget {
  const GlossaryScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(glossaryViewModelProvider);
    final page = t.workbench.glossary_page;
    return GlossaryView(
      labels: GlossaryViewLabels(
        title: t.workbench.glossary,
        newBook: page.new_book,
        rename: page.rename_book,
        enable: page.enable,
        disable: page.disable,
        delete: t.common.ui.button.delete,
        addEntry: page.add_entry,
        term: page.term,
        translation: page.translation,
        forbidden: page.forbidden,
        search: page.search_placeholder,
        emptyTitle: page.empty_title,
        emptyDescription: page.empty_description,
        noBooksTitle: page.no_books_title,
        noBooksDescription: page.no_books_description,
        loading: page.loading,
        retry: t.workbench.history_page.retry,
        save: t.common.ui.button.save,
        cancel: t.common.ui.button.cancel,
        caseSensitive: 'Aa',
        wholeWord: '[]',
        corrupt: t.ui.errors.glossary_corrupt,
      ),
      books: state.books,
      entries: state.entries,
      selectedBookId: state.selectedBookId,
      loading: state.loading,
      query: state.query,
      errorCode: state.errorCode,
      onSelectBook: (id) => unawaited(
        ref.read(glossaryViewModelProvider.notifier).selectBook(id),
      ),
      onQueryChanged: (value) => unawaited(
        ref.read(glossaryViewModelProvider.notifier).setQuery(value),
      ),
      onCreateBook: () => unawaited(_promptName(context, ref, create: true)),
      onRenameBook: () => unawaited(_promptName(context, ref, create: false)),
      onToggleBook: () =>
          unawaited(ref.read(glossaryViewModelProvider.notifier).toggleBook()),
      onDeleteBook: () =>
          unawaited(ref.read(glossaryViewModelProvider.notifier).deleteBook()),
      onAddEntry: () => unawaited(_editEntry(context, ref)),
      onEditEntry: (entry) => unawaited(_editEntry(context, ref, entry: entry)),
      onDeleteEntry: (entry) => unawaited(
        ref.read(glossaryViewModelProvider.notifier).deleteEntry(entry.id),
      ),
      onRetry: () =>
          unawaited(ref.read(glossaryViewModelProvider.notifier).reload()),
    );
  }
}

Future<void> _promptName(
  BuildContext context,
  WidgetRef ref, {
  required bool create,
}) async {
  final controller = TextEditingController();
  final name = await showDialog<String>(
    context: context,
    builder: (context) => AlertDialog(
      title: Text(
        create
            ? t.workbench.glossary_page.new_book
            : t.workbench.glossary_page.rename_book,
      ),
      content: TextField(controller: controller, autofocus: true),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: Text(t.common.ui.button.cancel),
        ),
        FilledButton(
          onPressed: () => Navigator.pop(context, controller.text.trim()),
          child: Text(t.common.ui.button.save),
        ),
      ],
    ),
  );
  if (name == null || name.isEmpty) return;
  if (create) {
    await ref.read(glossaryViewModelProvider.notifier).createBook(name);
  } else {
    await ref.read(glossaryViewModelProvider.notifier).renameBook(name);
  }
}

Future<void> _editEntry(
  BuildContext context,
  WidgetRef ref, {
  GlossaryEntryRecord? entry,
}) async {
  final term = TextEditingController(text: entry?.term ?? '');
  final translation = TextEditingController(text: entry?.translation ?? '');
  final forbidden = TextEditingController(
    text: entry?.forbidden.join(' / ') ?? '',
  );
  var caseSensitive = entry?.caseSensitive ?? false;
  var wholeWord = entry?.wholeWord ?? true;
  final saved = await showDialog<bool>(
    context: context,
    builder: (context) {
      return StatefulBuilder(
        builder: (context, setState) => AlertDialog(
          title: Text(t.workbench.glossary_page.add_entry),
          content: SizedBox(
            width: 420,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                TextField(
                  controller: term,
                  decoration: InputDecoration(
                    labelText: t.workbench.glossary_page.term,
                  ),
                ),
                TextField(
                  controller: translation,
                  decoration: InputDecoration(
                    labelText: t.workbench.glossary_page.translation,
                  ),
                ),
                TextField(
                  controller: forbidden,
                  decoration: InputDecoration(
                    labelText: t.workbench.glossary_page.forbidden,
                  ),
                ),
                SwitchListTile(
                  contentPadding: EdgeInsets.zero,
                  title: const Text('Aa'),
                  value: caseSensitive,
                  onChanged: (value) => setState(() => caseSensitive = value),
                ),
                SwitchListTile(
                  contentPadding: EdgeInsets.zero,
                  title: const Text('[]'),
                  value: wholeWord,
                  onChanged: (value) => setState(() => wholeWord = value),
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: Text(t.common.ui.button.cancel),
            ),
            FilledButton(
              onPressed: () => Navigator.pop(context, true),
              child: Text(t.common.ui.button.save),
            ),
          ],
        ),
      );
    },
  );
  if (saved != true) return;
  await ref
      .read(glossaryViewModelProvider.notifier)
      .saveEntry(
        GlossaryEntryDraft(
          id: entry?.id,
          term: term.text,
          translation: translation.text,
          forbidden: forbidden.text
              .split('/')
              .map((item) => item.trim())
              .where((item) => item.isNotEmpty)
              .toList(),
          caseSensitive: caseSensitive,
          wholeWord: wholeWord,
        ),
      );
}
