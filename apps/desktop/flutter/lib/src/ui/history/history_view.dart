import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../shared/settings_page.dart';
import '../shared/status_message.dart';

final class HistoryViewLabels {
  const HistoryViewLabels({
    required this.title,
    required this.all,
    required this.favorites,
    required this.edited,
    required this.search,
    required this.emptyTitle,
    required this.emptyDescription,
    required this.noResults,
    required this.loading,
    required this.retry,
    required this.delete,
    required this.clear,
    required this.exitSelection,
    required this.clearConfirm,
    required this.select,
    required this.open,
    required this.favorite,
    required this.unfavorite,
    required this.edit,
    required this.selectedCount,
    this.errorMessage,
  });

  final String title;
  final String all;
  final String favorites;
  final String edited;
  final String search;
  final String emptyTitle;
  final String emptyDescription;
  final String noResults;
  final String loading;
  final String retry;
  final String delete;
  final String clear;
  final String exitSelection;
  final String clearConfirm;
  final String select;
  final String open;
  final String favorite;
  final String unfavorite;
  final String edit;
  final String Function(int count) selectedCount;
  final String Function(String? code)? errorMessage;
}

class HistoryView extends StatelessWidget {
  const HistoryView({
    required this.labels,
    required this.snapshot,
    required this.selectedIds,
    required this.onQueryChanged,
    required this.onFilterChanged,
    required this.onOpen,
    required this.onFavorite,
    required this.onEdit,
    required this.onDelete,
    required this.onClear,
    required this.onRetry,
    required this.onToggleSelected,
    required this.onExitSelection,
    super.key,
    this.showFilter = true,
  });

  final HistoryViewLabels labels;
  final HistorySnapshot snapshot;
  final Set<String> selectedIds;
  final ValueChanged<String> onQueryChanged;
  final ValueChanged<HistoryFilter> onFilterChanged;
  final ValueChanged<HistoryRecord> onOpen;
  final void Function(HistoryRecord entry, bool favorite) onFavorite;
  final ValueChanged<HistoryRecord> onEdit;
  final ValueChanged<List<String>> onDelete;
  final VoidCallback onClear;
  final VoidCallback onRetry;
  final ValueChanged<String> onToggleSelected;
  final VoidCallback onExitSelection;
  final bool showFilter;

  @override
  Widget build(BuildContext context) {
    return SettingsPage(
      title: selectedIds.isEmpty
          ? labels.title
          : labels.selectedCount(selectedIds.length),
      actions: [
        if (selectedIds.isNotEmpty) ...[
          TextButton.icon(
            onPressed: () => onDelete(selectedIds.toList()),
            icon: const Icon(Icons.delete_outline_rounded, size: 18),
            label: Text(labels.delete),
          ),
          IconButton(
            tooltip: labels.exitSelection,
            onPressed: onExitSelection,
            icon: const Icon(Icons.close_rounded),
          ),
        ] else if (snapshot.entries.isNotEmpty)
          TextButton(onPressed: onClear, child: Text(labels.clear)),
      ],
      toolbar: Row(
        children: [
          Expanded(
            child: SearchBar(
              hintText: labels.search,
              leading: const Icon(Icons.search_rounded, size: 18),
              onChanged: onQueryChanged,
            ),
          ),
          if (showFilter) ...[
            const SizedBox(width: 12),
            SegmentedButton<HistoryFilter>(
              segments: [
                ButtonSegment(
                  value: HistoryFilter.all,
                  label: Text(labels.all),
                ),
                ButtonSegment(
                  value: HistoryFilter.favorites,
                  label: Text(labels.favorites),
                ),
                ButtonSegment(
                  value: HistoryFilter.edited,
                  label: Text(labels.edited),
                ),
              ],
              selected: {snapshot.filter},
              onSelectionChanged: (selection) {
                if (selection.isNotEmpty) {
                  onFilterChanged(selection.first);
                }
              },
            ),
          ],
        ],
      ),
      body: _body(context),
    );
  }

  Widget _body(BuildContext context) {
    if (snapshot.loading && snapshot.entries.isEmpty) {
      return StatusMessage(kind: StatusKind.progress, title: labels.loading);
    }
    if (snapshot.errorCode != null && snapshot.entries.isEmpty) {
      return Padding(
        padding: EdgeInsets.zero,
        child: StatusMessage(
          kind: StatusKind.error,
          title: labels.errorMessage?.call(snapshot.errorCode) ?? labels.retry,
          action: OutlinedButton(onPressed: onRetry, child: Text(labels.retry)),
        ),
      );
    }
    if (snapshot.entries.isEmpty) {
      return Padding(
        padding: EdgeInsets.zero,
        child: StatusMessage(
          kind: StatusKind.info,
          title: snapshot.query.isEmpty ? labels.emptyTitle : labels.noResults,
          body: snapshot.query.isEmpty ? labels.emptyDescription : null,
        ),
      );
    }
    return ListView.separated(
      padding: EdgeInsets.zero,
      itemCount: snapshot.entries.length,
      separatorBuilder: (_, _) => const Divider(height: 1),
      itemBuilder: (context, index) {
        final entry = snapshot.entries[index];
        return ListTile(
          selected: selectedIds.contains(entry.id),
          leading: selectedIds.isEmpty
              ? null
              : Checkbox(
                  value: selectedIds.contains(entry.id),
                  onChanged: (_) => onToggleSelected(entry.id),
                ),
          title: Text(
            entry.source,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
          ),
          subtitle: Text(
            entry.translation,
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
          ),
          trailing: Wrap(
            spacing: 0,
            children: [
              IconButton(
                tooltip: labels.edit,
                onPressed: () => onEdit(entry),
                icon: Icon(
                  entry.edited ? Icons.edit_rounded : Icons.edit_outlined,
                ),
              ),
              IconButton(
                tooltip: entry.favorite ? labels.unfavorite : labels.favorite,
                onPressed: () => onFavorite(entry, !entry.favorite),
                icon: Icon(
                  entry.favorite
                      ? Icons.star_rounded
                      : Icons.star_outline_rounded,
                ),
              ),
            ],
          ),
          onTap: selectedIds.isEmpty
              ? () => onOpen(entry)
              : () => onToggleSelected(entry.id),
          onLongPress: () => onToggleSelected(entry.id),
        );
      },
    );
  }
}
