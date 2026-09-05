import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../i18n/i18n.dart';
import '../../../shared/i18n_labels.dart';
import 'glossary_dialogs.dart';
import 'glossary_view.dart';
import 'glossary_view_model.dart';

class GlossarySettingsScreen extends ConsumerWidget {
  const GlossarySettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(glossaryViewModelProvider);
    final page = t.workbench.glossary_page;
    return GlossaryView(
      labels: GlossaryViewLabels(
        title: t.ui.shell.glossary,
        importFile: page.import_file,
        exportFile: page.export_file,
        newBook: page.new_book,
        addEntry: page.add_entry,
        renameBook: page.rename_book,
        enable: page.enable,
        disable: page.disable,
        delete: t.common.ui.button.delete,
        noBooksTitle: page.no_books_title,
        noBooksDescription: page.no_books_description,
        search: page.search_placeholder,
        emptyTitle: page.empty_title,
        hits: page.hits,
        errorMessage: appErrorMessage,
        noResultsTitle: (query) => page.no_results_title(query: query),
      ),
      state: state,
      onImport: (format) {
        final book = state.selectedBook;
        if (book != null) {
          unawaited(_importGlossary(context, ref, book, format));
        }
      },
      onExport: (format) {
        final book = state.selectedBook;
        if (book != null) {
          unawaited(_exportGlossary(context, ref, book, format));
        }
      },
      onNewBook: () => unawaited(_editBook(context, ref)),
      onAddEntry: () => unawaited(_editEntry(context, ref)),
      onSelectBook: (id) => unawaited(
        ref.read(glossaryViewModelProvider.notifier).selectBook(id),
      ),
      onRenameBook: (book) => unawaited(_editBook(context, ref, book)),
      onToggleBook: (book) => unawaited(
        ref.read(glossaryViewModelProvider.notifier).toggleBook(book),
      ),
      onDeleteBook: (book) => unawaited(_deleteBook(context, ref, book)),
      onQueryChanged: (value) => unawaited(
        ref.read(glossaryViewModelProvider.notifier).setQuery(value),
      ),
      onEditEntry: (entry) => unawaited(_editEntry(context, ref, entry)),
      onDeleteEntry: (entry) => unawaited(
        ref.read(glossaryViewModelProvider.notifier).deleteEntry(entry.id),
      ),
    );
  }
}

Future<void> _editBook(
  BuildContext context,
  WidgetRef ref, [
  GlossaryBookRecord? book,
]) async {
  final languages = await ref
      .read(glossaryViewModelProvider.notifier)
      .loadLanguages();
  if (!context.mounted) return;
  final draft = await showDialog<GlossaryBookDraft>(
    context: context,
    builder: (context) => GlossaryBookDialog(book: book, languages: languages),
  );
  if (draft == null) return;
  await ref.read(glossaryViewModelProvider.notifier).upsertBook(draft);
}

Future<void> _deleteBook(
  BuildContext context,
  WidgetRef ref,
  GlossaryBookRecord book,
) async {
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
  await ref.read(glossaryViewModelProvider.notifier).deleteBook(book.id);
}

Future<void> _editEntry(
  BuildContext context,
  WidgetRef ref, [
  GlossaryEntryRecord? entry,
]) async {
  if (ref.read(glossaryViewModelProvider).selectedBookId == null) return;
  final draft = await showDialog<GlossaryEntryDraft>(
    context: context,
    builder: (context) => GlossaryEntryDialog(entry: entry),
  );
  if (draft == null) return;
  await ref.read(glossaryViewModelProvider.notifier).upsertEntry(draft);
}

Future<void> _importGlossary(
  BuildContext context,
  WidgetRef ref,
  GlossaryBookRecord book,
  GlossaryExchangeFormat format,
) async {
  final page = t.workbench.glossary_page;
  try {
    final report = await ref
        .read(glossaryViewModelProvider.notifier)
        .importBook(book, format);
    if (report == null || !context.mounted) return;
    await _showExchangeMessage(
      context,
      page.import_success(
        inserted: report.inserted,
        updated: report.updated,
        skipped: report.skipped,
      ),
    );
  } catch (_) {
    if (context.mounted) {
      await _showExchangeMessage(context, page.import_failed);
    }
  }
}

Future<void> _exportGlossary(
  BuildContext context,
  WidgetRef ref,
  GlossaryBookRecord book,
  GlossaryExchangeFormat format,
) async {
  final page = t.workbench.glossary_page;
  try {
    final saved = await ref
        .read(glossaryViewModelProvider.notifier)
        .exportBook(book, format);
    if (!saved || !context.mounted) return;
    await _showExchangeMessage(context, page.export_success);
  } catch (_) {
    if (context.mounted) {
      await _showExchangeMessage(context, page.export_failed);
    }
  }
}

Future<void> _showExchangeMessage(BuildContext context, String message) {
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
