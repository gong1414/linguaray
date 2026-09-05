import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../i18n/i18n.dart';
import '../../../shared/i18n_labels.dart';
import 'vocabulary_view.dart';
import 'vocabulary_view_model.dart';

class VocabularySettingsScreen extends ConsumerWidget {
  const VocabularySettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final snapshot = ref.watch(vocabularyViewModelProvider);
    final labels = t.ui.vocabulary;
    return VocabularyView(
      labels: VocabularyViewLabels(
        title: labels.title,
        all: labels.all,
        favorites: labels.favorites,
        search: labels.search,
        emptyTitle: labels.empty_title,
        emptyDescription: labels.empty_description,
        noResults: labels.no_results,
        retry: t.workbench.translation.retry,
        favorite: labels.favorite,
        unfavorite: labels.unfavorite,
        delete: labels.delete,
        errorMessage: appErrorMessage,
      ),
      snapshot: snapshot,
      onFilterChanged: (filter) => unawaited(
        ref.read(vocabularyViewModelProvider.notifier).setFilter(filter),
      ),
      onQueryChanged: (value) => unawaited(
        ref.read(vocabularyViewModelProvider.notifier).setQuery(value),
      ),
      onRetry: () =>
          unawaited(ref.read(vocabularyViewModelProvider.notifier).reload()),
      onEditNote: (entry) => unawaited(_editNote(context, ref, entry)),
      onFavorite: (entry, favorite) => unawaited(
        ref
            .read(vocabularyViewModelProvider.notifier)
            .setFavorite(entry, favorite),
      ),
      onDelete: (entry) => unawaited(_delete(context, ref, entry)),
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
      content: TextField(
        controller: controller,
        autofocus: true,
        minLines: 2,
        maxLines: 5,
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: Text(t.common.ui.button.cancel),
        ),
        FilledButton(
          onPressed: () => Navigator.pop(context, controller.text.trim()),
          child: Text(t.common.ui.button.save),
        ),
      ],
    ),
  );
  controller.dispose();
  if (note == null) return;
  await ref
      .read(vocabularyViewModelProvider.notifier)
      .updateNote(entry, note.isEmpty ? null : note);
}

Future<void> _delete(
  BuildContext context,
  WidgetRef ref,
  VocabularyRecord entry,
) async {
  final confirmed = await showDialog<bool>(
    context: context,
    builder: (context) => AlertDialog(
      title: Text(t.ui.vocabulary.delete),
      content: Text(entry.word),
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
  if (confirmed != true) return;
  await ref.read(vocabularyViewModelProvider.notifier).delete(entry.id);
}
