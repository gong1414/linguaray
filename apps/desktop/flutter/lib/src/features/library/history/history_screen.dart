import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../app/dependencies.dart';
import '../../../i18n/i18n.dart';
import '../../../shared/i18n_labels.dart';
import 'history_view.dart';
import 'history_view_model.dart';

class HistoryScreen extends ConsumerStatefulWidget {
  const HistoryScreen({
    required this.initialFilter,
    super.key,
    this.lockFilter = false,
  });

  final HistoryFilter initialFilter;
  final bool lockFilter;

  @override
  ConsumerState<HistoryScreen> createState() => _HistoryScreenState();
}

class _HistoryScreenState extends ConsumerState<HistoryScreen> {
  @override
  void initState() {
    super.initState();
    scheduleMicrotask(
      () => ref
          .read(historyViewModelProvider.notifier)
          .setFilter(widget.initialFilter),
    );
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(historyViewModelProvider);
    final snapshot = state.snapshot;
    final page = t.workbench.history_page;
    return HistoryView(
      labels: HistoryViewLabels(
        title: t.workbench.history,
        all: page.all,
        favorites: page.favorites,
        edited: page.edited,
        search: page.search_placeholder,
        emptyTitle: page.empty_title,
        emptyDescription: page.empty_description,
        noResults: page.no_results(query: snapshot.query),
        loading: page.loading,
        retry: page.retry,
        delete: t.common.ui.button.delete,
        clear: page.clear_all,
        exitSelection: page.exit_select,
        clearConfirm: page.delete_message,
        select: page.select,
        open: page.copy_translation,
        favorite: page.favorite,
        unfavorite: page.unfavorite,
        edit: t.common.ui.button.edit,
        selectedCount: (count) => page.selected_count(count: count),
        errorMessage: appErrorMessage,
      ),
      snapshot: snapshot,
      selectedIds: state.selectedIds,
      showFilter: !widget.lockFilter,
      onQueryChanged: (value) => unawaited(
        ref.read(historyViewModelProvider.notifier).setQuery(value),
      ),
      onFilterChanged: (value) => unawaited(
        ref.read(historyViewModelProvider.notifier).setFilter(value),
      ),
      onOpen: (entry) => unawaited(
        ref.read(triggerControllerProvider).translateText(entry.source),
      ),
      onFavorite: (entry, favorite) => unawaited(
        ref
            .read(historyViewModelProvider.notifier)
            .toggleFavorite(entry, favorite),
      ),
      onEdit: (entry) => unawaited(_editHistory(context, ref, entry)),
      onDelete: (_) => unawaited(_confirmDeleteSelected(context, ref)),
      onClear: () => unawaited(_confirmClear(context, ref)),
      onRetry: () =>
          unawaited(ref.read(historyViewModelProvider.notifier).reload()),
      onToggleSelected: ref
          .read(historyViewModelProvider.notifier)
          .toggleSelected,
      onExitSelection: ref
          .read(historyViewModelProvider.notifier)
          .clearSelection,
    );
  }
}

Future<void> _editHistory(
  BuildContext context,
  WidgetRef ref,
  HistoryRecord entry,
) async {
  final source = TextEditingController(text: entry.source);
  final translation = TextEditingController(text: entry.translation);
  final result = await showDialog<(String, String)>(
    context: context,
    builder: (context) => AlertDialog(
      title: Text(t.workbench.history_page.edit_history_hint),
      content: SizedBox(
        width: 480,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: source,
              minLines: 2,
              maxLines: 5,
              decoration: InputDecoration(
                labelText: t.workbench.glossary_page.term,
              ),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: translation,
              minLines: 2,
              maxLines: 5,
              decoration: InputDecoration(
                labelText: t.workbench.glossary_page.translation,
              ),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: Text(t.common.ui.button.cancel),
        ),
        FilledButton(
          onPressed: () {
            final nextSource = source.text.trim();
            final nextTranslation = translation.text.trim();
            if (nextSource.isEmpty || nextTranslation.isEmpty) return;
            Navigator.pop(context, (nextSource, nextTranslation));
          },
          child: Text(t.common.ui.button.save),
        ),
      ],
    ),
  );
  source.dispose();
  translation.dispose();
  if (result == null ||
      (result.$1 == entry.source && result.$2 == entry.translation)) {
    return;
  }
  await ref
      .read(historyViewModelProvider.notifier)
      .edit(entry, result.$1, result.$2);
}

Future<void> _confirmDeleteSelected(BuildContext context, WidgetRef ref) async {
  final notifier = ref.read(historyViewModelProvider.notifier);
  final count = ref.read(historyViewModelProvider).selectedIds.length;
  if (count == 0) return;
  final confirmed = await showDialog<bool>(
    context: context,
    builder: (context) => AlertDialog(
      title: Text(t.workbench.history_page.delete_title_many(count: count)),
      content: Text(t.workbench.history_page.delete_confirm(count: count)),
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
  if (confirmed == true) await notifier.deleteSelected();
}

Future<void> _confirmClear(BuildContext context, WidgetRef ref) async {
  final confirmed = await showDialog<bool>(
    context: context,
    builder: (context) => AlertDialog(
      title: Text(
        t.workbench.history_page.delete_title_many(
          count: ref.read(historyViewModelProvider).snapshot.counts.all,
        ),
      ),
      content: Text(t.workbench.history_page.delete_message),
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
  if (confirmed == true) {
    await ref.read(historyViewModelProvider.notifier).clear();
  }
}
