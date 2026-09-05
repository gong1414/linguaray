import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../i18n/i18n.dart';
import '../../../shared/i18n_labels.dart';
import '../../../shared/settings_page.dart';
import '../../../shared/status_message.dart';
import 'vocabulary_view_model.dart';

class VocabularySettingsScreen extends ConsumerWidget {
  const VocabularySettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final snapshot = ref.watch(vocabularyViewModelProvider);
    final labels = t.ui.vocabulary;
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
            unawaited(
              ref
                  .read(vocabularyViewModelProvider.notifier)
                  .setFilter(selection.first),
            );
          },
        ),
      ],
      toolbar: SearchBar(
        hintText: labels.search,
        leading: const Icon(Icons.search_rounded, size: 18),
        onChanged: (value) => unawaited(
          ref.read(vocabularyViewModelProvider.notifier).setQuery(value),
        ),
      ),
      body: _body(context, ref, snapshot),
    );
  }
}

Widget _body(BuildContext context, WidgetRef ref, VocabularySnapshot snapshot) {
  final labels = t.ui.vocabulary;
  if (snapshot.loading && snapshot.entries.isEmpty) {
    return const Center(child: CircularProgressIndicator());
  }
  if (snapshot.errorCode != null && snapshot.entries.isEmpty) {
    return Padding(
      padding: EdgeInsets.zero,
      child: StatusMessage(
        kind: StatusKind.error,
        title: appErrorMessage(snapshot.errorCode),
        action: OutlinedButton(
          onPressed: () => unawaited(
            ref.read(vocabularyViewModelProvider.notifier).reload(),
          ),
          child: Text(t.workbench.translation.retry),
        ),
      ),
    );
  }
  if (snapshot.entries.isEmpty) {
    return Padding(
      padding: EdgeInsets.zero,
      child: StatusMessage(
        kind: StatusKind.info,
        title: snapshot.query.isEmpty ? labels.empty_title : labels.no_results,
        body: snapshot.query.isEmpty ? labels.empty_description : null,
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
        title: Text(entry.word),
        subtitle: Text(
          [
            entry.translation,
            if (entry.note?.isNotEmpty == true) entry.note!,
          ].join('\n'),
          maxLines: 3,
          overflow: TextOverflow.ellipsis,
        ),
        onTap: () => unawaited(_editNote(context, ref, entry)),
        trailing: Wrap(
          spacing: 0,
          children: [
            IconButton(
              tooltip: entry.favorite ? labels.unfavorite : labels.favorite,
              onPressed: () => unawaited(
                ref
                    .read(vocabularyViewModelProvider.notifier)
                    .setFavorite(entry, !entry.favorite),
              ),
              icon: Icon(
                entry.favorite
                    ? Icons.star_rounded
                    : Icons.star_outline_rounded,
              ),
            ),
            IconButton(
              tooltip: labels.delete,
              onPressed: () => unawaited(_delete(context, ref, entry)),
              icon: const Icon(Icons.delete_outline_rounded),
            ),
          ],
        ),
      );
    },
  );
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
