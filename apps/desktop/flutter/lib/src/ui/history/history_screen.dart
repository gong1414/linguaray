import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../config/dependencies.dart';
import '../../i18n/i18n.dart';
import '../../platform/trigger_controller.dart';
import '../../services/app_windows.dart';
import '../i18n_labels.dart';
import 'history_view.dart';

final historyViewModelProvider =
    NotifierProvider<HistoryViewModel, HistorySnapshot>(HistoryViewModel.new);

final class HistoryViewModel extends Notifier<HistorySnapshot> {
  HistoryFilter _filter = HistoryFilter.all;
  String _query = '';
  final Set<String> selectedIds = {};

  @override
  HistorySnapshot build() {
    scheduleMicrotask(reload);
    return const HistorySnapshot(
      loading: true,
      entries: [],
      counts: HistoryCounts.empty(),
      filter: HistoryFilter.all,
      query: '',
    );
  }

  Future<void> reload() async {
    state = HistorySnapshot(
      entries: state.entries,
      counts: state.counts,
      filter: _filter,
      query: _query,
      loading: true,
    );
    state = await ref
        .read(historyRepositoryProvider)
        .load(filter: _filter, query: _query);
  }

  Future<void> setFilter(HistoryFilter filter) async {
    _filter = filter;
    await reload();
  }

  Future<void> setQuery(String query) async {
    _query = query;
    await reload();
  }

  Future<void> toggleFavorite(HistoryRecord entry, bool favorite) async {
    await ref
        .read(historyRepositoryProvider)
        .setFavorite(entryId: entry.id, favorite: favorite);
    await reload();
  }

  Future<void> deleteSelected() async {
    await ref.read(historyRepositoryProvider).delete(selectedIds.toList());
    selectedIds.clear();
    await reload();
  }

  Future<void> clear() async {
    await ref.read(historyRepositoryProvider).clear();
    selectedIds.clear();
    await reload();
  }

  void toggleSelected(String id) {
    if (!selectedIds.add(id)) selectedIds.remove(id);
  }
}

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
    final snapshot = ref.watch(historyViewModelProvider);
    final page = t.workbench.history_page;
    return HistoryView(
      labels: HistoryViewLabels(
        title: t.workbench.history,
        all: page.all,
        favorites: page.favorites,
        search: page.search_placeholder,
        emptyTitle: page.empty_title,
        emptyDescription: page.empty_description,
        noResults: page.no_results(query: snapshot.query),
        loading: page.loading,
        retry: page.retry,
        delete: t.common.ui.button.delete,
        clear: page.exit_select,
        clearConfirm: page.delete_message,
        select: page.select,
        open: page.copy_translation,
        favorite: page.favorite,
        unfavorite: page.unfavorite,
        errorMessage: appErrorMessage,
      ),
      snapshot: snapshot,
      selectedIds: ref.read(historyViewModelProvider.notifier).selectedIds,
      showFilter: !widget.lockFilter,
      onQueryChanged: (value) => unawaited(
        ref.read(historyViewModelProvider.notifier).setQuery(value),
      ),
      onFilterChanged: (value) => unawaited(
        ref.read(historyViewModelProvider.notifier).setFilter(value),
      ),
      onOpen: (entry) {
        triggerController.quickWindowRequest.value = QuickWindowRequest(
          text: entry.source,
          submit: true,
          clearExisting: true,
        );
        unawaited(showMiniTranslatorWindow());
      },
      onFavorite: (entry, favorite) => unawaited(
        ref
            .read(historyViewModelProvider.notifier)
            .toggleFavorite(entry, favorite),
      ),
      onDelete: (_) => unawaited(
        ref.read(historyViewModelProvider.notifier).deleteSelected(),
      ),
      onClear: () => unawaited(_confirmClear(context, ref)),
      onRetry: () =>
          unawaited(ref.read(historyViewModelProvider.notifier).reload()),
      onToggleSelected: ref
          .read(historyViewModelProvider.notifier)
          .toggleSelected,
    );
  }

  Future<void> _confirmClear(BuildContext context, WidgetRef ref) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(t.workbench.history_page.delete_title_many(count: 0)),
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
}
