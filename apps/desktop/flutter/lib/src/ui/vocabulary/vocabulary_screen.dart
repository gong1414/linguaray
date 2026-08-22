import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../config/dependencies.dart';
import '../../i18n/i18n.dart';
import '../i18n_labels.dart';
import 'vocabulary_view.dart';

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
    state = await ref
        .read(vocabularyRepositoryProvider)
        .load(filter: _filter, query: _query);
  }

  Future<void> setFilter(VocabularyFilter filter) async {
    _filter = filter;
    await reload();
  }

  Future<void> setQuery(String query) async {
    _query = query;
    await reload();
  }

  Future<void> toggleFavorite(VocabularyRecord entry, bool favorite) async {
    await ref
        .read(vocabularyRepositoryProvider)
        .setFavorite(entryId: entry.id, favorite: favorite);
    await reload();
  }

  Future<void> delete(VocabularyRecord entry) async {
    await ref.read(vocabularyRepositoryProvider).delete([entry.id]);
    await reload();
  }

  Future<void> updateNote(VocabularyRecord entry, String note) async {
    await ref
        .read(vocabularyRepositoryProvider)
        .updateNote(entryId: entry.id, note: note);
    await reload();
  }
}

class VocabularyScreen extends ConsumerWidget {
  const VocabularyScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final snapshot = ref.watch(vocabularyViewModelProvider);
    final labels = t.ui.vocabulary;
    return VocabularyView(
      labels: VocabularyViewLabels(
        title: labels.title,
        search: labels.search,
        all: labels.all,
        favorites: labels.favorites,
        emptyTitle: labels.empty_title,
        emptyDescription: labels.empty_description,
        noResults: labels.no_results,
        note: labels.note,
        delete: labels.delete,
        favorite: labels.favorite,
        unfavorite: labels.unfavorite,
        retry: t.workbench.history_page.retry,
        errorMessage: appErrorMessage,
      ),
      snapshot: snapshot,
      onQueryChanged: (value) => unawaited(
        ref.read(vocabularyViewModelProvider.notifier).setQuery(value),
      ),
      onFilterChanged: (value) => unawaited(
        ref.read(vocabularyViewModelProvider.notifier).setFilter(value),
      ),
      onFavorite: (entry, favorite) => unawaited(
        ref
            .read(vocabularyViewModelProvider.notifier)
            .toggleFavorite(entry, favorite),
      ),
      onDelete: (entry) => unawaited(
        ref.read(vocabularyViewModelProvider.notifier).delete(entry),
      ),
      onEditNote: (entry) => unawaited(_editNote(context, ref, entry)),
      onRetry: () =>
          unawaited(ref.read(vocabularyViewModelProvider.notifier).reload()),
    );
  }
}

Future<void> _editNote(
  BuildContext context,
  WidgetRef ref,
  VocabularyRecord entry,
) async {
  final controller = TextEditingController(text: entry.note ?? '');
  final note = await showDialog<String>(
    context: context,
    builder: (context) => AlertDialog(
      title: Text(t.ui.vocabulary.note),
      content: TextField(controller: controller, maxLines: 4),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: Text(t.common.ui.button.cancel),
        ),
        FilledButton(
          onPressed: () => Navigator.pop(context, controller.text),
          child: Text(t.common.ui.button.save),
        ),
      ],
    ),
  );
  if (note == null) return;
  await ref.read(vocabularyViewModelProvider.notifier).updateNote(entry, note);
}
