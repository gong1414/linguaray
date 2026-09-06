import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../shared/settings_page.dart';
import '../../../shared/status_message.dart';
import 'glossary_view_model.dart';

final class GlossaryViewLabels {
  const GlossaryViewLabels({
    required this.title,
    required this.importFile,
    required this.exportFile,
    required this.newBook,
    required this.addEntry,
    required this.renameBook,
    required this.enable,
    required this.disable,
    required this.delete,
    required this.noBooksTitle,
    required this.noBooksDescription,
    required this.search,
    required this.emptyTitle,
    required this.hits,
    required this.errorMessage,
    required this.noResultsTitle,
  });

  final String title;
  final String importFile;
  final String exportFile;
  final String newBook;
  final String addEntry;
  final String renameBook;
  final String enable;
  final String disable;
  final String delete;
  final String noBooksTitle;
  final String noBooksDescription;
  final String search;
  final String emptyTitle;
  final String hits;
  final String Function(String? code) errorMessage;
  final String Function(String query) noResultsTitle;
}

class GlossaryView extends StatelessWidget {
  const GlossaryView({
    required this.labels,
    required this.state,
    required this.onImport,
    required this.onExport,
    required this.onNewBook,
    required this.onAddEntry,
    required this.onSelectBook,
    required this.onRenameBook,
    required this.onToggleBook,
    required this.onDeleteBook,
    required this.onQueryChanged,
    required this.onEditEntry,
    required this.onDeleteEntry,
    super.key,
  });

  final GlossaryViewLabels labels;
  final GlossaryViewState state;
  final ValueChanged<GlossaryExchangeFormat> onImport;
  final ValueChanged<GlossaryExchangeFormat> onExport;
  final VoidCallback onNewBook;
  final VoidCallback onAddEntry;
  final ValueChanged<String> onSelectBook;
  final ValueChanged<GlossaryBookRecord> onRenameBook;
  final ValueChanged<GlossaryBookRecord> onToggleBook;
  final ValueChanged<GlossaryBookRecord> onDeleteBook;
  final ValueChanged<String> onQueryChanged;
  final ValueChanged<GlossaryEntryRecord> onEditEntry;
  final ValueChanged<GlossaryEntryRecord> onDeleteEntry;

  @override
  Widget build(BuildContext context) {
    final selectedBook = state.selectedBook;
    return SettingsPage(
      title: labels.title,
      actions: [
        PopupMenuButton<GlossaryExchangeFormat>(
          enabled: selectedBook != null,
          tooltip: labels.importFile,
          icon: const Icon(Icons.file_upload_outlined),
          onSelected: onImport,
          itemBuilder: (_) => [
            PopupMenuItem(
              value: GlossaryExchangeFormat.csv,
              child: Text('${labels.importFile} · CSV'),
            ),
            PopupMenuItem(
              value: GlossaryExchangeFormat.tbx,
              child: Text('${labels.importFile} · TBX'),
            ),
          ],
        ),
        PopupMenuButton<GlossaryExchangeFormat>(
          enabled: selectedBook != null,
          tooltip: labels.exportFile,
          icon: const Icon(Icons.file_download_outlined),
          onSelected: onExport,
          itemBuilder: (_) => [
            PopupMenuItem(
              value: GlossaryExchangeFormat.csv,
              child: Text('${labels.exportFile} · CSV'),
            ),
            PopupMenuItem(
              value: GlossaryExchangeFormat.tbx,
              child: Text('${labels.exportFile} · TBX'),
            ),
          ],
        ),
        OutlinedButton.icon(
          onPressed: onNewBook,
          icon: const Icon(Icons.create_new_folder_outlined, size: 18),
          label: Text(labels.newBook),
        ),
        FilledButton.icon(
          onPressed: selectedBook == null ? null : onAddEntry,
          icon: const Icon(Icons.add_rounded, size: 18),
          label: Text(labels.addEntry),
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
                title: labels.errorMessage(state.errorCode),
              ),
            ),
          Expanded(
            child: state.books.isEmpty
                ? StatusMessage(
                    title: labels.noBooksTitle,
                    body: labels.noBooksDescription,
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
                                onTap: () => onSelectBook(book.id),
                                trailing: PopupMenuButton<String>(
                                  onSelected: (action) {
                                    switch (action) {
                                      case 'rename':
                                        onRenameBook(book);
                                      case 'toggle':
                                        onToggleBook(book);
                                      case 'delete':
                                        onDeleteBook(book);
                                    }
                                  },
                                  itemBuilder: (_) => [
                                    PopupMenuItem(
                                      value: 'rename',
                                      child: Text(labels.renameBook),
                                    ),
                                    PopupMenuItem(
                                      value: 'toggle',
                                      child: Text(
                                        book.enabled
                                            ? labels.disable
                                            : labels.enable,
                                      ),
                                    ),
                                    PopupMenuItem(
                                      value: 'delete',
                                      child: Text(labels.delete),
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
                            ? Center(child: Text(labels.noBooksDescription))
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
                                      hintText: labels.search,
                                      leading: const Icon(
                                        Icons.search_rounded,
                                        size: 18,
                                      ),
                                      onChanged: onQueryChanged,
                                    ),
                                  ),
                                  Expanded(child: _entryBody()),
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

  Widget _entryBody() {
    if (state.loading) return const Center(child: CircularProgressIndicator());
    if (state.entries.isEmpty) {
      return Padding(
        padding: const EdgeInsets.only(left: 20),
        child: StatusMessage(
          title: state.query.isEmpty
              ? labels.emptyTitle
              : labels.noResultsTitle(state.query),
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
          onTap: () => onEditEntry(entry),
          trailing: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (entry.hits > 0) Text('${labels.hits} ${entry.hits}'),
              IconButton(
                tooltip: labels.delete,
                onPressed: () => onDeleteEntry(entry),
                icon: const Icon(Icons.delete_outline_rounded),
              ),
            ],
          ),
        );
      },
    );
  }
}
