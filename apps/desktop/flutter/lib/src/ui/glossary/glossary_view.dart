import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../shared/status_message.dart';

final class GlossaryViewLabels {
  const GlossaryViewLabels({
    required this.title,
    required this.newBook,
    required this.rename,
    required this.enable,
    required this.disable,
    required this.delete,
    required this.addEntry,
    required this.term,
    required this.translation,
    required this.forbidden,
    required this.search,
    required this.emptyTitle,
    required this.emptyDescription,
    required this.noBooksTitle,
    required this.noBooksDescription,
    required this.loading,
    required this.retry,
    required this.save,
    required this.cancel,
    required this.caseSensitive,
    required this.wholeWord,
    required this.corrupt,
  });

  final String title;
  final String newBook;
  final String rename;
  final String enable;
  final String disable;
  final String delete;
  final String addEntry;
  final String term;
  final String translation;
  final String forbidden;
  final String search;
  final String emptyTitle;
  final String emptyDescription;
  final String noBooksTitle;
  final String noBooksDescription;
  final String loading;
  final String retry;
  final String save;
  final String cancel;
  final String caseSensitive;
  final String wholeWord;
  final String corrupt;
}

class GlossaryView extends StatelessWidget {
  const GlossaryView({
    required this.labels,
    required this.books,
    required this.entries,
    required this.selectedBookId,
    required this.loading,
    required this.query,
    required this.onSelectBook,
    required this.onQueryChanged,
    required this.onCreateBook,
    required this.onRenameBook,
    required this.onToggleBook,
    required this.onDeleteBook,
    required this.onAddEntry,
    required this.onEditEntry,
    required this.onDeleteEntry,
    required this.onRetry,
    super.key,
    this.errorCode,
  });

  final GlossaryViewLabels labels;
  final List<GlossaryBookRecord> books;
  final List<GlossaryEntryRecord> entries;
  final String? selectedBookId;
  final bool loading;
  final String query;
  final String? errorCode;
  final ValueChanged<String> onSelectBook;
  final ValueChanged<String> onQueryChanged;
  final VoidCallback onCreateBook;
  final VoidCallback onRenameBook;
  final VoidCallback onToggleBook;
  final VoidCallback onDeleteBook;
  final VoidCallback onAddEntry;
  final ValueChanged<GlossaryEntryRecord> onEditEntry;
  final ValueChanged<GlossaryEntryRecord> onDeleteEntry;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final selected = books.where((book) => book.id == selectedBookId);
    final book = selected.isEmpty ? null : selected.first;

    return Material(
      color: theme.scaffoldBackgroundColor,
      child: Row(
        children: [
          SizedBox(
            width: 240,
            child: Column(
              children: [
                Padding(
                  padding: const EdgeInsets.fromLTRB(16, 16, 8, 8),
                  child: Row(
                    children: [
                      Expanded(
                        child: Text(
                          labels.title,
                          style: theme.textTheme.titleLarge,
                        ),
                      ),
                      IconButton(
                        tooltip: labels.newBook,
                        onPressed: onCreateBook,
                        icon: const Icon(Icons.add_rounded),
                      ),
                    ],
                  ),
                ),
                Expanded(
                  child: books.isEmpty
                      ? Padding(
                          padding: const EdgeInsets.all(16),
                          child: StatusMessage(
                            kind: StatusKind.info,
                            title: labels.noBooksTitle,
                            body: labels.noBooksDescription,
                          ),
                        )
                      : ListView(
                          children: [
                            for (final item in books)
                              ListTile(
                                selected: item.id == selectedBookId,
                                title: Text(item.name),
                                subtitle: Text('${item.entryCount}'),
                                trailing: item.enabled
                                    ? null
                                    : Text(labels.disable),
                                onTap: () => onSelectBook(item.id),
                              ),
                          ],
                        ),
                ),
              ],
            ),
          ),
          VerticalDivider(width: 1, color: theme.colorScheme.outlineVariant),
          Expanded(
            child: book == null
                ? const SizedBox.shrink()
                : Column(
                    children: [
                      Padding(
                        padding: const EdgeInsets.fromLTRB(16, 12, 16, 8),
                        child: Row(
                          children: [
                            Expanded(
                              child: Text(
                                book.name,
                                style: theme.textTheme.titleLarge,
                              ),
                            ),
                            TextButton(
                              onPressed: onToggleBook,
                              child: Text(
                                book.enabled ? labels.disable : labels.enable,
                              ),
                            ),
                            TextButton(
                              onPressed: onRenameBook,
                              child: Text(labels.rename),
                            ),
                            TextButton(
                              onPressed: onDeleteBook,
                              child: Text(labels.delete),
                            ),
                            FilledButton(
                              onPressed: onAddEntry,
                              child: Text(labels.addEntry),
                            ),
                          ],
                        ),
                      ),
                      if (book.errorCode != null)
                        Padding(
                          padding: const EdgeInsets.symmetric(horizontal: 16),
                          child: StatusMessage(
                            kind: StatusKind.warning,
                            title: labels.corrupt,
                          ),
                        ),
                      Padding(
                        padding: const EdgeInsets.fromLTRB(16, 0, 16, 8),
                        child: SearchBar(
                          hintText: labels.search,
                          leading: const Icon(Icons.search_rounded, size: 18),
                          onChanged: onQueryChanged,
                        ),
                      ),
                      Expanded(
                        child: loading
                            ? StatusMessage(
                                kind: StatusKind.progress,
                                title: labels.loading,
                              )
                            : entries.isEmpty
                            ? Padding(
                                padding: const EdgeInsets.all(24),
                                child: StatusMessage(
                                  kind: StatusKind.info,
                                  title: labels.emptyTitle,
                                  body: labels.emptyDescription,
                                ),
                              )
                            : ListView.separated(
                                itemCount: entries.length,
                                separatorBuilder: (_, _) =>
                                    const Divider(height: 1),
                                itemBuilder: (context, index) {
                                  final entry = entries[index];
                                  return ListTile(
                                    title: Text(entry.term),
                                    subtitle: Text(entry.translation),
                                    trailing: IconButton(
                                      tooltip: labels.delete,
                                      onPressed: () => onDeleteEntry(entry),
                                      icon: const Icon(
                                        Icons.delete_outline_rounded,
                                      ),
                                    ),
                                    onTap: () => onEditEntry(entry),
                                  );
                                },
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
