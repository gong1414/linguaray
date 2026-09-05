import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';

final class DictionaryLookupDialogLabels {
  const DictionaryLookupDialogLabels({
    required this.title,
    required this.pronunciation,
    required this.speak,
    required this.definitions,
    required this.save,
    required this.saved,
    required this.close,
    required this.empty,
    required this.lookupFailed,
    required this.saveFailed,
  });

  final String title;
  final String pronunciation;
  final String speak;
  final String definitions;
  final String save;
  final String saved;
  final String close;
  final String empty;
  final String lookupFailed;
  final String saveFailed;
}

class DictionaryLookupDialog extends StatefulWidget {
  const DictionaryLookupDialog({
    required this.labels,
    required this.lookup,
    required this.onSave,
    this.onSpeak,
    super.key,
  });

  final DictionaryLookupDialogLabels labels;
  final Future<DictionaryEntry?> lookup;
  final Future<void> Function(DictionaryEntry entry) onSave;
  final Future<void> Function(DictionaryEntry entry)? onSpeak;

  @override
  State<DictionaryLookupDialog> createState() => _DictionaryLookupDialogState();
}

class _DictionaryLookupDialogState extends State<DictionaryLookupDialog> {
  bool _saving = false;
  bool _saved = false;
  bool _speaking = false;
  String? _saveError;

  Future<void> _save(DictionaryEntry entry) async {
    if (_saving || _saved) return;
    setState(() {
      _saving = true;
      _saveError = null;
    });
    try {
      await widget.onSave(entry);
      if (!mounted) return;
      setState(() {
        _saving = false;
        _saved = true;
      });
    } catch (_) {
      if (!mounted) return;
      setState(() {
        _saving = false;
        _saveError = widget.labels.saveFailed;
      });
    }
  }

  Future<void> _speak(DictionaryEntry entry) async {
    if (_speaking || widget.onSpeak == null) return;
    setState(() => _speaking = true);
    try {
      await widget.onSpeak!(entry);
    } finally {
      if (mounted) setState(() => _speaking = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Text(widget.labels.title),
      content: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 480, maxHeight: 440),
        child: FutureBuilder<DictionaryEntry?>(
          future: widget.lookup,
          builder: (context, snapshot) {
            if (snapshot.connectionState != ConnectionState.done) {
              return const SizedBox(
                width: 320,
                height: 120,
                child: Center(child: CircularProgressIndicator()),
              );
            }
            if (snapshot.hasError) {
              return _DialogMessage(
                icon: Icons.error_outline_rounded,
                message: widget.labels.lookupFailed,
              );
            }
            final entry = snapshot.data;
            if (entry == null || entry.isEmpty) {
              return _DialogMessage(
                icon: Icons.menu_book_outlined,
                message: widget.labels.empty,
              );
            }
            return _DictionaryEntryBody(
              labels: widget.labels,
              entry: entry,
              saving: _saving,
              saved: _saved,
              saveError: _saveError,
              onSave: _vocabularyTranslation(entry).isEmpty
                  ? null
                  : () => _save(entry),
              speaking: _speaking,
              onSpeak: widget.onSpeak == null ? null : () => _speak(entry),
            );
          },
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: Text(widget.labels.close),
        ),
      ],
    );
  }
}

class _DictionaryEntryBody extends StatelessWidget {
  const _DictionaryEntryBody({
    required this.labels,
    required this.entry,
    required this.saving,
    required this.saved,
    required this.saveError,
    required this.onSave,
    required this.speaking,
    required this.onSpeak,
  });

  final DictionaryLookupDialogLabels labels;
  final DictionaryEntry entry;
  final bool saving;
  final bool saved;
  final String? saveError;
  final VoidCallback? onSave;
  final bool speaking;
  final VoidCallback? onSpeak;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return SingleChildScrollView(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(entry.word, style: theme.textTheme.headlineSmall),
              ),
              if (onSpeak != null)
                IconButton(
                  tooltip: labels.speak,
                  onPressed: speaking ? null : onSpeak,
                  icon: speaking
                      ? const SizedBox.square(
                          dimension: 18,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.volume_up_outlined),
                ),
            ],
          ),
          if (entry.providerName.trim().isNotEmpty) ...[
            const SizedBox(height: 2),
            Text(
              entry.providerName,
              style: theme.textTheme.labelMedium?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
          ],
          if (entry.pronunciations.isNotEmpty) ...[
            const SizedBox(height: 18),
            Text(labels.pronunciation, style: theme.textTheme.titleSmall),
            const SizedBox(height: 6),
            for (final pronunciation in entry.pronunciations)
              Text(
                [
                  if (pronunciation.accent?.trim().isNotEmpty ?? false)
                    pronunciation.accent!.trim(),
                  pronunciation.text.trim(),
                ].where((value) => value.isNotEmpty).join('  '),
              ),
          ],
          if (entry.translations.isNotEmpty) ...[
            const SizedBox(height: 18),
            for (final translation in entry.translations)
              Padding(
                padding: const EdgeInsets.only(bottom: 6),
                child: Text(translation, style: theme.textTheme.bodyLarge),
              ),
          ],
          if (entry.definitions.isNotEmpty) ...[
            const SizedBox(height: 12),
            Text(labels.definitions, style: theme.textTheme.titleSmall),
            const SizedBox(height: 6),
            for (final definition in entry.definitions)
              Padding(
                padding: const EdgeInsets.only(bottom: 10),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    if (definition.partOfSpeech?.trim().isNotEmpty ?? false)
                      Text(
                        definition.partOfSpeech!,
                        style: theme.textTheme.labelLarge?.copyWith(
                          color: theme.colorScheme.primary,
                        ),
                      ),
                    for (final value in definition.values) Text('• $value'),
                  ],
                ),
              ),
          ],
          const SizedBox(height: 10),
          if (saveError != null)
            Padding(
              padding: const EdgeInsets.only(bottom: 8),
              child: Text(
                saveError!,
                style: theme.textTheme.bodySmall?.copyWith(
                  color: theme.colorScheme.error,
                ),
              ),
            ),
          Align(
            alignment: Alignment.centerRight,
            child: FilledButton.tonalIcon(
              onPressed: saving || saved ? null : onSave,
              icon: saving
                  ? const SizedBox.square(
                      dimension: 16,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : Icon(
                      saved ? Icons.check_rounded : Icons.bookmark_add_outlined,
                    ),
              label: Text(saved ? labels.saved : labels.save),
            ),
          ),
        ],
      ),
    );
  }
}

class _DialogMessage extends StatelessWidget {
  const _DialogMessage({required this.icon, required this.message});

  final IconData icon;
  final String message;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 320,
      height: 120,
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(icon, size: 26),
          const SizedBox(height: 10),
          Text(message, textAlign: TextAlign.center),
        ],
      ),
    );
  }
}

String _vocabularyTranslation(DictionaryEntry entry) {
  for (final value in entry.translations) {
    if (value.trim().isNotEmpty) return value.trim();
  }
  for (final definition in entry.definitions) {
    for (final value in definition.values) {
      if (value.trim().isNotEmpty) return value.trim();
    }
  }
  return '';
}

String dictionaryVocabularyTranslation(DictionaryEntry entry) =>
    _vocabularyTranslation(entry);
