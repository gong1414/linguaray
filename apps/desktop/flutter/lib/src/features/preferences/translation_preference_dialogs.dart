import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../i18n/i18n.dart';
import 'settings_view_model.dart';

Future<List<TranslationTargetRule>?> showTranslationTargetsDialog(
  BuildContext context,
  GeneralSettingsViewState state,
) {
  final preferences = state.preferences;
  if (preferences == null || state.translationLanguages.isEmpty) {
    return Future.value();
  }
  final targets = [...preferences.translationTargets];
  return showDialog<List<TranslationTargetRule>>(
    context: context,
    builder: (dialogContext) => StatefulBuilder(
      builder: (context, setDialogState) => AlertDialog(
        title: Text(t.settings.general.button.manage_targets),
        content: SizedBox(
          width: 500,
          height: 340,
          child: targets.isEmpty
              ? Center(
                  child: Text(t.settings.general.row.no_translation_targets),
                )
              : ListView.builder(
                  itemCount: targets.length,
                  itemBuilder: (context, index) {
                    final target = targets[index];
                    return SwitchListTile(
                      contentPadding: EdgeInsets.zero,
                      title: Text('${target.source} → ${target.target}'),
                      value: target.enabled,
                      onChanged: (value) => setDialogState(() {
                        targets[index] = TranslationTargetRule(
                          source: target.source,
                          target: target.target,
                          enabled: value,
                        );
                      }),
                      secondary: IconButton(
                        icon: const Icon(Icons.delete_outline_rounded),
                        onPressed: () =>
                            setDialogState(() => targets.removeAt(index)),
                      ),
                    );
                  },
                ),
        ),
        actions: [
          TextButton.icon(
            onPressed: () async {
              final added = await _showAddTranslationTargetDialog(
                context,
                state.translationLanguages,
              );
              if (added != null && context.mounted) {
                setDialogState(() => targets.add(added));
              }
            },
            icon: const Icon(Icons.add_rounded),
            label: Text(t.common.ui.button.add),
          ),
          TextButton(
            onPressed: () => Navigator.pop(dialogContext),
            child: Text(t.common.ui.button.cancel),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(dialogContext, targets),
            child: Text(t.common.ui.button.save),
          ),
        ],
      ),
    ),
  );
}

Future<List<String>?> showCommonLanguagesDialog(
  BuildContext context,
  GeneralSettingsViewState state,
) {
  final preferences = state.preferences;
  if (preferences == null) return Future.value();
  final selected = {...preferences.commonLanguages};
  return showDialog<List<String>>(
    context: context,
    builder: (dialogContext) => StatefulBuilder(
      builder: (context, setDialogState) => AlertDialog(
        title: Text(t.settings.general.button.manage_languages),
        content: SizedBox(
          width: 440,
          height: 430,
          child: ListView(
            children: [
              for (final language in state.translationLanguages)
                CheckboxListTile(
                  contentPadding: EdgeInsets.zero,
                  title: Text(language.name),
                  value: selected.contains(language.code),
                  onChanged: (value) => setDialogState(() {
                    value ?? false
                        ? selected.add(language.code)
                        : selected.remove(language.code);
                  }),
                ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(dialogContext),
            child: Text(t.common.ui.button.cancel),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(dialogContext, selected.toList()),
            child: Text(t.common.ui.button.save),
          ),
        ],
      ),
    ),
  );
}

Future<TranslationTargetRule?> _showAddTranslationTargetDialog(
  BuildContext context,
  List<LanguageChoice> languages,
) {
  var source = 'auto';
  var target = languages.first.code;
  return showDialog<TranslationTargetRule>(
    context: context,
    builder: (dialogContext) => StatefulBuilder(
      builder: (context, setDialogState) => AlertDialog(
        title: Text(t.settings.general.button.add_target),
        content: SizedBox(
          width: 420,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              DropdownButtonFormField<String>(
                initialValue: source,
                decoration: InputDecoration(
                  labelText: t.settings.general.editor.row.source_language,
                ),
                items: [
                  DropdownMenuItem(
                    value: 'auto',
                    child: Text(t.mini_translator.language.auto_detect),
                  ),
                  for (final language in languages)
                    DropdownMenuItem(
                      value: language.code,
                      child: Text(language.name),
                    ),
                ],
                onChanged: (value) =>
                    setDialogState(() => source = value ?? source),
              ),
              const SizedBox(height: 12),
              DropdownButtonFormField<String>(
                initialValue: target,
                decoration: InputDecoration(
                  labelText: t.settings.general.editor.row.target_language,
                ),
                items: [
                  for (final language in languages)
                    DropdownMenuItem(
                      value: language.code,
                      child: Text(language.name),
                    ),
                ],
                onChanged: (value) =>
                    setDialogState(() => target = value ?? target),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(dialogContext),
            child: Text(t.common.ui.button.cancel),
          ),
          FilledButton(
            onPressed: source == target
                ? null
                : () => Navigator.pop(
                    dialogContext,
                    TranslationTargetRule(
                      source: source,
                      target: target,
                      enabled: true,
                    ),
                  ),
            child: Text(t.common.ui.button.add),
          ),
        ],
      ),
    ),
  );
}
