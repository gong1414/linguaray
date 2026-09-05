import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../app/dependencies.dart';

final vocabularyViewModelProvider =
    NotifierProvider<VocabularyViewModel, VocabularySnapshot>(
      VocabularyViewModel.new,
    );

final class VocabularyViewModel extends Notifier<VocabularySnapshot> {
  VocabularyFilter _filter = VocabularyFilter.all;
  String _query = '';

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
    state = VocabularySnapshot(
      entries: state.entries,
      filter: _filter,
      query: _query,
      loading: true,
      errorCode: state.errorCode,
    );
    try {
      state = await ref
          .read(vocabularyRepositoryProvider)
          .load(filter: _filter, query: _query);
    } catch (_) {
      state = VocabularySnapshot(
        entries: state.entries,
        filter: _filter,
        query: _query,
        loading: false,
        errorCode: AppErrorCode.vocabularyUnavailable.wireName,
      );
    }
  }

  Future<void> setFilter(VocabularyFilter filter) async {
    _filter = filter;
    await reload();
  }

  Future<void> setQuery(String query) async {
    _query = query;
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
