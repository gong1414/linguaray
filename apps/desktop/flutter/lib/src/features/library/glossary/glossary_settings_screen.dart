import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../i18n/i18n.dart';
import '../../../shared/i18n_labels.dart';
import '../../../shared/settings_page.dart';
import '../../../shared/status_message.dart';
import 'glossary_dialogs.dart';
import 'glossary_view_model.dart';

class GlossarySettingsScreen extends ConsumerWidget {
  const GlossarySettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(glossaryViewModelProvider);
    final page = t.workbench.glossary_page;
    final selectedBook = state.selectedBook;
    return SettingsPage(
      title: t.ui.shell.glossary,
      actions: [
        PopupMenuButton<GlossaryExchangeFormat>(
          enabled: selectedBook != null,
          tooltip: page.import_file,
          icon: const Icon(Icons.file_upload_outlined),
          onSelected: (format) =>
              unawaited(_importGlossary(context, ref, selectedBook!, format)),
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
              unawaited(_exportGlossary(context, ref, selectedBook!, format)),
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
          onPressed: () => unawaited(_editBook(context, ref)),
          icon: const Icon(Icons.create_new_folder_outlined, size: 18),
          label: Text(page.new_book),
        ),
        FilledButton.icon(
          onPressed: selectedBook == null
              ? null
              : () => unawaited(_editEntry(context, ref)),
          icon: const Icon(Icons.add_rounded, size: 18),
          label: Text(page.add_entry),
        ),
      ],
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (state.errorCode != null)
            Padding(
              padding: const EdgeInsets.only(bottom: 16),
              child: StatusMessage(
                kind: StatusKind.warning,
                title: appErrorMessage(state.errorCode),
              ),
            ),
          Expanded(
            child: state.books.isEmpty
                ? StatusMessage(
                    title: page.no_books_title,
                    body: page.no_books_description,
                  )
                : Row(
                    children: [
                      SizedBox(
                        width: 190,
                        child: ListView(
                          padding: const EdgeInsets.only(right: 12),
                          children: [
                            for (final book in state.books)
                              ListTile(
                                dense: true,
                                selected: book.id == state.selectedBookId,
                                title: Text(book.name),
                                subtitle: Text('${book.entryCount}'),
                                onTap: () => unawaited(
                                  ref
                                      .read(glossaryViewModelProvider.notifier)
                                      .selectBook(book.id),
                                ),
                                trailing: PopupMenuButton<String>(
                                  onSelected: (action) async {
                                    final notifier = ref.read(
                                      glossaryViewModelProvider.notifier,
                                    );
                                    if (action == 'rename') {
                                      await _editBook(context, ref, book);
                                    } else if (action == 'toggle') {
                                      await notifier.toggleBook(book);
                                    } else if (action == 'delete') {
                                      await _deleteBook(context, ref, book);
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
                                      child: Text(t.common.ui.button.delete),
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
                                      onChanged: (value) => unawaited(
                                        ref
                                            .read(
                                              glossaryViewModelProvider
                                                  .notifier,
                                            )
                                            .setQuery(value),
                                      ),
                                    ),
                                  ),
                                  Expanded(
                                    child: _entryBody(context, ref, state),
                                  ),
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
}

Widget _entryBody(
  BuildContext context,
  WidgetRef ref,
  GlossaryViewState state,
) {
  final page = t.workbench.glossary_page;
  if (state.loading) return const Center(child: CircularProgressIndicator());
  if (state.entries.isEmpty) {
    return Padding(
      padding: const EdgeInsets.only(left: 20),
      child: StatusMessage(
        title: state.query.isEmpty
            ? page.empty_title
            : page.no_results_title(query: state.query),
      ),
    );
  }
  return ListView.separated(
    padding: const EdgeInsets.only(left: 12),
    itemCount: state.entries.length,
    separatorBuilder: (_, _) => const Divider(height: 1),
    itemBuilder: (context, index) {
      final entry = state.entries[index];
      return ListTile(
        title: Text(entry.term),
        subtitle: Text(entry.translation),
        onTap: () => unawaited(_editEntry(context, ref, entry)),
        trailing: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            if (entry.hits > 0) Text('${page.hits} ${entry.hits}'),
            IconButton(
              tooltip: t.common.ui.button.delete,
              onPressed: () => unawaited(
                ref
                    .read(glossaryViewModelProvider.notifier)
                    .deleteEntry(entry.id),
              ),
              icon: const Icon(Icons.delete_outline_rounded),
            ),
          ],
        ),
      );
    },
  );
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
