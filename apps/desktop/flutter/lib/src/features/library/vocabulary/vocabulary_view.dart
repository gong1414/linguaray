import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../shared/settings_page.dart';
import '../../../shared/status_message.dart';

final class VocabularyViewLabels {
  const VocabularyViewLabels({
    required this.title,
    required this.all,
    required this.favorites,
    required this.search,
    required this.emptyTitle,
    required this.emptyDescription,
    required this.noResults,
    required this.retry,
    required this.favorite,
    required this.unfavorite,
    required this.delete,
    required this.errorMessage,
  });

  final String title;
  final String all;
  final String favorites;
  final String search;
  final String emptyTitle;
  final String emptyDescription;
  final String noResults;
  final String retry;
  final String favorite;
  final String unfavorite;
  final String delete;
  final String Function(String? code) errorMessage;
}

class VocabularyView extends StatelessWidget {
  const VocabularyView({
    required this.labels,
    required this.snapshot,
    required this.onFilterChanged,
    required this.onQueryChanged,
    required this.onRetry,
    required this.onEditNote,
    required this.onFavorite,
    required this.onDelete,
    super.key,
  });

  final VocabularyViewLabels labels;
  final VocabularySnapshot snapshot;
  final ValueChanged<VocabularyFilter> onFilterChanged;
  final ValueChanged<String> onQueryChanged;
  final VoidCallback onRetry;
  final ValueChanged<VocabularyRecord> onEditNote;
  final void Function(VocabularyRecord entry, bool favorite) onFavorite;
  final ValueChanged<VocabularyRecord> onDelete;

  @override
  Widget build(BuildContext context) {
    return SettingsPage(
      title: labels.title,
      actions: [
        SegmentedButton<VocabularyFilter>(
          segments: [
            ButtonSegment(value: VocabularyFilter.all, label: Text(labels.all)),
            ButtonSegment(
              value: VocabularyFilter.favorites,
              label: Text(labels.favorites),
            ),
          ],
          selected: {snapshot.filter},
          onSelectionChanged: (selection) {
            if (selection.isEmpty) return;
            onFilterChanged(selection.first);
          },
        ),
      ],
      toolbar: SearchBar(
        hintText: labels.search,
        leading: const Icon(Icons.search_rounded, size: 18),
        onChanged: onQueryChanged,
      ),
      body: _body(),
    );
  }

  Widget _body() {
    if (snapshot.loading && snapshot.entries.isEmpty) {
      return const Center(child: CircularProgressIndicator());
    }
    if (snapshot.errorCode != null && snapshot.entries.isEmpty) {
      return StatusMessage(
        kind: StatusKind.error,
        title: labels.errorMessage(snapshot.errorCode),
        action: OutlinedButton(onPressed: onRetry, child: Text(labels.retry)),
      );
    }
    if (snapshot.entries.isEmpty) {
      return StatusMessage(
        kind: StatusKind.info,
        title: snapshot.query.isEmpty ? labels.emptyTitle : labels.noResults,
        body: snapshot.query.isEmpty ? labels.emptyDescription : null,
      );
    }
    return ListView.separated(
      padding: EdgeInsets.zero,
      itemCount: snapshot.entries.length,
      separatorBuilder: (_, _) => const Divider(height: 1),
      itemBuilder: (context, index) {
        final entry = snapshot.entries[index];
        return ListTile(
          title: Text(entry.word),
          subtitle: Text(
            [
              entry.translation,
              if (entry.note?.isNotEmpty == true) entry.note!,
            ].join('\n'),
            maxLines: 3,
            overflow: TextOverflow.ellipsis,
          ),
          onTap: () => onEditNote(entry),
          trailing: Wrap(
            spacing: 0,
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
