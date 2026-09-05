import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../app/dependencies.dart';

final historyViewModelProvider =
    NotifierProvider<HistoryViewModel, HistoryViewState>(HistoryViewModel.new);

final class HistoryViewState {
  const HistoryViewState({required this.snapshot, this.selectedIds = const {}});

  final HistorySnapshot snapshot;
  final Set<String> selectedIds;
}

final class HistoryViewModel extends Notifier<HistoryViewState> {
  HistoryFilter _filter = HistoryFilter.all;
  String _query = '';

  @override
  HistoryViewState build() {
    scheduleMicrotask(reload);
    return const HistoryViewState(
      snapshot: HistorySnapshot(
        loading: true,
        entries: [],
        counts: HistoryCounts.empty(),
        filter: HistoryFilter.all,
        query: '',
      ),
    );
  }

  Future<void> reload() async {
    state = HistoryViewState(
      snapshot: HistorySnapshot(
        entries: state.snapshot.entries,
        counts: state.snapshot.counts,
        filter: _filter,
        query: _query,
        loading: true,
        errorCode: state.snapshot.errorCode,
      ),
    );
    final snapshot = await ref
        .read(historyRepositoryProvider)
        .load(filter: _filter, query: _query);
    state = HistoryViewState(snapshot: snapshot);
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

  Future<void> edit(
    HistoryRecord entry,
    String source,
    String translation,
  ) async {
    await ref
        .read(historyRepositoryProvider)
        .upsert(
          HistoryRecordDraft(
            id: entry.id,
            source: source,
            translation: translation,
            sourceLanguage: entry.sourceLanguage,
            targetLanguage: entry.targetLanguage,
            serviceId: entry.serviceId,
            serviceName: entry.serviceName,
            edited: true,
          ),
        );
    await reload();
  }

  Future<void> deleteSelected() async {
    await ref
        .read(historyRepositoryProvider)
        .delete(state.selectedIds.toList());
    await reload();
  }

  Future<void> clear() async {
    await ref.read(historyRepositoryProvider).clear();
    await reload();
  }

  void toggleSelected(String id) {
    final selectedIds = {...state.selectedIds};
    if (!selectedIds.add(id)) selectedIds.remove(id);
    state = HistoryViewState(
      snapshot: state.snapshot,
      selectedIds: selectedIds,
    );
  }

  void clearSelection() {
    state = HistoryViewState(snapshot: state.snapshot);
  }
}
