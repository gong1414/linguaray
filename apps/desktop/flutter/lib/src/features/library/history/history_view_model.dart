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
  var _generation = 0;

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
    final generation = ++_generation;
    final filter = state.snapshot.filter;
    final query = state.snapshot.query;
    state = HistoryViewState(
      snapshot: HistorySnapshot(
        entries: state.snapshot.entries,
        counts: state.snapshot.counts,
        filter: filter,
        query: query,
        loading: true,
        errorCode: state.snapshot.errorCode,
      ),
      selectedIds: state.selectedIds,
    );
    try {
      final snapshot = await ref
          .read(historyRepositoryProvider)
          .load(filter: filter, query: query);
      if (generation != _generation) return;
      state = HistoryViewState(snapshot: snapshot);
    } catch (_) {
      if (generation != _generation) return;
      state = HistoryViewState(
        snapshot: HistorySnapshot(
          entries: state.snapshot.entries,
          counts: state.snapshot.counts,
          filter: filter,
          query: query,
          loading: false,
          errorCode: AppErrorCode.historyUnavailable.wireName,
        ),
      );
    }
  }

  Future<void> setFilter(HistoryFilter filter) async {
    state = HistoryViewState(
      snapshot: HistorySnapshot(
        entries: state.snapshot.entries,
        counts: state.snapshot.counts,
        filter: filter,
        query: state.snapshot.query,
        loading: true,
        errorCode: state.snapshot.errorCode,
      ),
    );
    await reload();
  }

  Future<void> setQuery(String query) async {
    state = HistoryViewState(
      snapshot: HistorySnapshot(
        entries: state.snapshot.entries,
        counts: state.snapshot.counts,
        filter: state.snapshot.filter,
        query: query,
        loading: true,
        errorCode: state.snapshot.errorCode,
      ),
    );
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
