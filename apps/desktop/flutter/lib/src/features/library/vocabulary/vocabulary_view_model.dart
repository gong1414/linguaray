import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../app/dependencies.dart';

final vocabularyViewModelProvider =
    NotifierProvider<VocabularyViewModel, VocabularySnapshot>(
      VocabularyViewModel.new,
    );

final class VocabularyViewModel extends Notifier<VocabularySnapshot> {
  var _generation = 0;

  @override
  VocabularySnapshot build() {
    scheduleMicrotask(reload);
    return const VocabularySnapshot(
      entries: [],
      filter: VocabularyFilter.all,
      query: '',
      loading: true,
    );
  }

  Future<void> reload() async {
    final generation = ++_generation;
    final filter = state.filter;
    final query = state.query;
    state = VocabularySnapshot(
      entries: state.entries,
      filter: filter,
      query: query,
      loading: true,
      errorCode: state.errorCode,
    );
    try {
      final snapshot = await ref
          .read(vocabularyRepositoryProvider)
          .load(filter: filter, query: query);
      if (generation != _generation) return;
      state = snapshot;
    } catch (_) {
      if (generation != _generation) return;
      state = VocabularySnapshot(
        entries: state.entries,
        filter: filter,
        query: query,
        loading: false,
        errorCode: AppErrorCode.vocabularyUnavailable.wireName,
      );
    }
  }

  Future<void> setFilter(VocabularyFilter filter) async {
    state = VocabularySnapshot(
      entries: state.entries,
      filter: filter,
      query: state.query,
      loading: true,
      errorCode: state.errorCode,
    );
    await reload();
  }

  Future<void> setQuery(String query) async {
    state = VocabularySnapshot(
      entries: state.entries,
      filter: state.filter,
      query: query,
      loading: true,
      errorCode: state.errorCode,
    );
    await reload();
  }

  Future<void> setFavorite(VocabularyRecord entry, bool favorite) async {
    await ref
        .read(vocabularyRepositoryProvider)
        .setFavorite(entryId: entry.id, favorite: favorite);
    await reload();
  }

  Future<void> updateNote(VocabularyRecord entry, String? note) async {
    await ref
        .read(vocabularyRepositoryProvider)
        .updateNote(entryId: entry.id, note: note);
    await reload();
  }

  Future<void> delete(String entryId) async {
    await ref.read(vocabularyRepositoryProvider).delete([entryId]);
    await reload();
  }
}
