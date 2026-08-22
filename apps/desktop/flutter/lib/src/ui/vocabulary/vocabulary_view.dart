import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../shared/status_message.dart';

final class VocabularyViewLabels {
  const VocabularyViewLabels({
    required this.title,
    required this.search,
    required this.all,
    required this.favorites,
    required this.emptyTitle,
    required this.emptyDescription,
    required this.noResults,
    required this.note,
    required this.delete,
    required this.favorite,
    required this.unfavorite,
    required this.retry,
    this.errorMessage,
  });

  final String title;
  final String search;
  final String all;
  final String favorites;
  final String emptyTitle;
  final String emptyDescription;
  final String noResults;
  final String note;
  final String delete;
  final String favorite;
  final String unfavorite;
  final String retry;
  final String Function(String? code)? errorMessage;
}

class VocabularyView extends StatelessWidget {
  const VocabularyView({
    required this.labels,
    required this.snapshot,
    required this.onQueryChanged,
    required this.onFilterChanged,
    required this.onFavorite,
    required this.onDelete,
    required this.onEditNote,
    required this.onRetry,
    super.key,
  });

  final VocabularyViewLabels labels;
  final VocabularySnapshot snapshot;
  final ValueChanged<String> onQueryChanged;
  final ValueChanged<VocabularyFilter> onFilterChanged;
  final void Function(VocabularyRecord entry, bool favorite) onFavorite;
  final ValueChanged<VocabularyRecord> onDelete;
  final ValueChanged<VocabularyRecord> onEditNote;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Material(
      color: theme.scaffoldBackgroundColor,
      child: Column(
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(20, 16, 20, 8),
            child: Text(labels.title, style: theme.textTheme.headlineMedium),
          ),
          Padding(
            padding: const EdgeInsets.fromLTRB(20, 0, 20, 8),
            child: Row(
              children: [
                Expanded(
                  child: SearchBar(
                    hintText: labels.search,
                    leading: const Icon(Icons.search_rounded, size: 18),
                    onChanged: onQueryChanged,
                  ),
                ),
                const SizedBox(width: 12),
                SegmentedButton<VocabularyFilter>(
                  segments: [
                    ButtonSegment(
                      value: VocabularyFilter.all,
                      label: Text(labels.all),
                    ),
                    ButtonSegment(
                      value: VocabularyFilter.favorites,
                      label: Text(labels.favorites),
                    ),
                  ],
                  selected: {snapshot.filter},
                  onSelectionChanged: (selection) {
                    if (selection.isNotEmpty) onFilterChanged(selection.first);
                  },
                ),
              ],
            ),
          ),
          Expanded(child: _body(context)),
        ],
      ),
    );
  }

  Widget _body(BuildContext context) {
    if (snapshot.loading && snapshot.entries.isEmpty) {
      return const Center(child: CircularProgressIndicator());
    }
    if (snapshot.errorCode != null && snapshot.entries.isEmpty) {
      return Padding(
        padding: const EdgeInsets.all(24),
        child: StatusMessage(
          kind: StatusKind.error,
          title: labels.errorMessage?.call(snapshot.errorCode) ?? labels.retry,
          action: OutlinedButton(onPressed: onRetry, child: Text(labels.retry)),
        ),
      );
    }
    if (snapshot.entries.isEmpty) {
      return Padding(
        padding: const EdgeInsets.all(24),
        child: StatusMessage(
          kind: StatusKind.info,
          title: snapshot.query.isEmpty ? labels.emptyTitle : labels.noResults,
          body: snapshot.query.isEmpty ? labels.emptyDescription : null,
        ),
      );
    }
    return ListView.separated(
      padding: const EdgeInsets.fromLTRB(12, 0, 12, 24),
      itemCount: snapshot.entries.length,
      separatorBuilder: (_, _) => const Divider(height: 1),
      itemBuilder: (context, index) {
        final entry = snapshot.entries[index];
        return ListTile(
          title: Text(entry.word),
          subtitle: Text(
            [
              entry.translation,
              if (entry.note != null && entry.note!.isNotEmpty) entry.note!,
            ].join('\n'),
          ),
          isThreeLine: entry.note?.isNotEmpty == true,
          trailing: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              IconButton(
                tooltip: entry.favorite ? labels.unfavorite : labels.favorite,
                onPressed: () => onFavorite(entry, !entry.favorite),
                icon: Icon(
                  entry.favorite
                      ? Icons.star_rounded
                      : Icons.star_outline_rounded,
                ),
              ),
              IconButton(
                tooltip: labels.note,
                onPressed: () => onEditNote(entry),
                icon: const Icon(Icons.notes_outlined),
              ),
              IconButton(
                tooltip: labels.delete,
                onPressed: () => onDelete(entry),
                icon: const Icon(Icons.delete_outline_rounded),
              ),
            ],
          ),
        );
      },
    );
  }
}
