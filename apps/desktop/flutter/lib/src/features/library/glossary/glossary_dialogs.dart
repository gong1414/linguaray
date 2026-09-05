import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../i18n/i18n.dart';

class GlossaryBookDialog extends StatefulWidget {
  const GlossaryBookDialog({required this.languages, super.key, this.book});

  final GlossaryBookRecord? book;
  final List<LanguageOption> languages;

  @override
  State<GlossaryBookDialog> createState() => _GlossaryBookDialogState();
}

class _GlossaryBookDialogState extends State<GlossaryBookDialog> {
  late final TextEditingController _name = TextEditingController(
    text: widget.book?.name ?? '',
  );
  late String? _sourceLanguage = widget.book?.sourceLanguage;
  late String? _targetLanguage = widget.book?.targetLanguage;

  @override
  void dispose() {
    _name.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final page = t.workbench.glossary_page;
    final languageItems = <DropdownMenuItem<String?>>[
      DropdownMenuItem(value: null, child: Text(t.ui.vocabulary.all)),
      for (final language in widget.languages)
        DropdownMenuItem(value: language.code, child: Text(language.name)),
    ];
    return AlertDialog(
      title: Text(widget.book == null ? page.new_book : page.rename_book),
      content: SizedBox(
        width: 420,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: _name,
              autofocus: true,
              decoration: InputDecoration(
                labelText: page.name,
                hintText: page.name_placeholder,
              ),
            ),
            const SizedBox(height: 12),
            DropdownButtonFormField<String?>(
              initialValue: _sourceLanguage,
              decoration: InputDecoration(labelText: page.source_language),
              items: languageItems,
              onChanged: (value) => setState(() => _sourceLanguage = value),
            ),
            const SizedBox(height: 12),
            DropdownButtonFormField<String?>(
              initialValue: _targetLanguage,
              decoration: InputDecoration(labelText: page.target_language),
              items: languageItems,
              onChanged: (value) => setState(() => _targetLanguage = value),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: Text(t.common.ui.button.cancel),
        ),
        FilledButton(
          onPressed: () {
            final name = _name.text.trim();
            if (name.isEmpty ||
                _sourceLanguage != null && _sourceLanguage == _targetLanguage) {
              return;
            }
            Navigator.pop(
              context,
              GlossaryBookDraft(
                id: widget.book?.id,
                name: name,
                enabled: widget.book?.enabled ?? true,
                sourceLanguage: _sourceLanguage,
                targetLanguage: _targetLanguage,
              ),
            );
          },
          child: Text(t.common.ui.button.save),
        ),
      ],
    );
  }
}

class GlossaryEntryDialog extends StatefulWidget {
  const GlossaryEntryDialog({super.key, this.entry});

  final GlossaryEntryRecord? entry;

  @override
  State<GlossaryEntryDialog> createState() => _GlossaryEntryDialogState();
}

class _GlossaryEntryDialogState extends State<GlossaryEntryDialog> {
  late final TextEditingController _term = TextEditingController(
    text: widget.entry?.term ?? '',
  );
  late final TextEditingController _translation = TextEditingController(
    text: widget.entry?.translation ?? '',
  );
  late final TextEditingController _forbidden = TextEditingController(
    text: widget.entry?.forbidden.join(' / ') ?? '',
  );

  @override
  void dispose() {
    _term.dispose();
    _translation.dispose();
    _forbidden.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final page = t.workbench.glossary_page;
    return AlertDialog(
      title: Text(page.add_entry),
      content: SizedBox(
        width: 420,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: _term,
              autofocus: true,
              decoration: InputDecoration(labelText: page.term),
            ),
            const SizedBox(height: 10),
            TextField(
              controller: _translation,
              decoration: InputDecoration(labelText: page.translation),
            ),
            const SizedBox(height: 10),
            TextField(
              controller: _forbidden,
              decoration: InputDecoration(
                labelText: page.forbidden_label,
                hintText: page.forbidden_placeholder_full,
              ),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: Text(t.common.ui.button.cancel),
        ),
        FilledButton(
          onPressed: () {
            final term = _term.text.trim();
            final translation = _translation.text.trim();
            if (term.isEmpty || translation.isEmpty) return;
            Navigator.pop(
              context,
              GlossaryEntryDraft(
                id: widget.entry?.id,
                term: term,
                translation: translation,
                forbidden: _forbidden.text
                    .split(RegExp(r'[/,\n]'))
                    .map((value) => value.trim())
                    .where((value) => value.isNotEmpty)
                    .toList(),
                note: widget.entry?.note,
                caseSensitive: widget.entry?.caseSensitive ?? false,
                wholeWord: widget.entry?.wholeWord ?? true,
              ),
            );
          },
          child: Text(t.common.ui.button.save),
        ),
      ],
    );
  }
}
