import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../app/dependencies.dart';
import '../../../i18n/i18n.dart';
import '../../../shared/i18n_labels.dart';
import '../../../shared/settings_page.dart';
import '../../../shared/status_message.dart';

class VocabularySettingsScreen extends ConsumerStatefulWidget {
  const VocabularySettingsScreen({super.key});

  @override
  ConsumerState<VocabularySettingsScreen> createState() =>
      _VocabularySettingsScreenState();
}

class _VocabularySettingsScreenState
    extends ConsumerState<VocabularySettingsScreen> {
  VocabularySnapshot _snapshot = const VocabularySnapshot.empty();
  VocabularyFilter _filter = VocabularyFilter.all;
  String _query = '';

  @override
  void initState() {
    super.initState();
    unawaited(_reload());
  }

  Future<void> _reload() async {
    setState(() {
      _snapshot = VocabularySnapshot(
        entries: _snapshot.entries,
        filter: _filter,
        query: _query,
        loading: true,
      );
    });
    final snapshot = await ref
        .read(vocabularyRepositoryProvider)
        .load(filter: _filter, query: _query);
    if (mounted) setState(() => _snapshot = snapshot);
  }

  Future<void> _editNote(VocabularyRecord entry) async {
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
        .read(vocabularyRepositoryProvider)
        .updateNote(entryId: entry.id, note: note.isEmpty ? null : note);
    await _reload();
  }

  Future<void> _delete(VocabularyRecord entry) async {
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
    await ref.read(vocabularyRepositoryProvider).delete([entry.id]);
    await _reload();
  }

  @override
  Widget build(BuildContext context) {
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
          selected: {_filter},
          onSelectionChanged: (selection) {
            if (selection.isEmpty) return;
            _filter = selection.first;
            unawaited(_reload());
          },
        ),
      ],
      toolbar: SearchBar(
        hintText: labels.search,
        leading: const Icon(Icons.search_rounded, size: 18),
        onChanged: (value) {
          _query = value;
          unawaited(_reload());
        },
      ),
      body: _body(),
    );
  }

  Widget _body() {
    final labels = t.ui.vocabulary;
    if (_snapshot.loading && _snapshot.entries.isEmpty) {
      return const Center(child: CircularProgressIndicator());
    }
    if (_snapshot.errorCode != null && _snapshot.entries.isEmpty) {
      return Padding(
        padding: EdgeInsets.zero,
        child: StatusMessage(
          kind: StatusKind.error,
          title: appErrorMessage(_snapshot.errorCode),
          action: OutlinedButton(
            onPressed: () => unawaited(_reload()),
            child: Text(t.workbench.translation.retry),
          ),
        ),
      );
    }
    if (_snapshot.entries.isEmpty) {
      return Padding(
        padding: EdgeInsets.zero,
        child: StatusMessage(
          kind: StatusKind.info,
          title: _query.isEmpty ? labels.empty_title : labels.no_results,
          body: _query.isEmpty ? labels.empty_description : null,
        ),
      );
    }
    return ListView.separated(
      padding: EdgeInsets.zero,
      itemCount: _snapshot.entries.length,
      separatorBuilder: (_, _) => const Divider(height: 1),
      itemBuilder: (context, index) {
        final entry = _snapshot.entries[index];
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
          onTap: () => unawaited(_editNote(entry)),
          trailing: Wrap(
            spacing: 0,
            children: [
              IconButton(
                tooltip: entry.favorite ? labels.unfavorite : labels.favorite,
                onPressed: () async {
                  await ref
                      .read(vocabularyRepositoryProvider)
                      .setFavorite(
                        entryId: entry.id,
                        favorite: !entry.favorite,
                      );
                  await _reload();
                },
                icon: Icon(
                  entry.favorite
                      ? Icons.star_rounded
                      : Icons.star_outline_rounded,
                ),
              ),
              IconButton(
                tooltip: labels.delete,
                onPressed: () => unawaited(_delete(entry)),
                icon: const Icon(Icons.delete_outline_rounded),
              ),
            ],
          ),
        );
      },
    );
  }
}
