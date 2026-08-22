import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../settings_labels.dart';

class GeneralSettingsView extends StatelessWidget {
  const GeneralSettingsView({
    required this.labels,
    required this.preferences,
    required this.languages,
    required this.onLaunchAtLoginChanged,
    required this.onShowInMenuBarChanged,
    required this.onLanguageChanged,
    required this.onThemeModeChanged,
    super.key,
    this.translationLanguages = const [],
    this.errorCode,
    this.onRetry,
    this.onCommonLanguagesChanged,
    this.onInputSubmitModeChanged,
    this.onAutoCopyChanged,
    this.onDoubleClickCopyChanged,
  });

  final GeneralSettingsLabels labels;
  final GeneralPreferences preferences;
  final List<LanguageChoice> languages;
  final List<LanguageChoice> translationLanguages;
  final String? errorCode;
  final VoidCallback? onRetry;
  final ValueChanged<bool> onLaunchAtLoginChanged;
  final ValueChanged<bool> onShowInMenuBarChanged;
  final ValueChanged<String> onLanguageChanged;
  final ValueChanged<ThemePreference> onThemeModeChanged;
  final ValueChanged<List<String>>? onCommonLanguagesChanged;
  final ValueChanged<InputSubmitMode>? onInputSubmitModeChanged;
  final ValueChanged<bool>? onAutoCopyChanged;
  final ValueChanged<bool>? onDoubleClickCopyChanged;

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.fromLTRB(20, 16, 24, 24),
      children: [
        Text(labels.startup, style: Theme.of(context).textTheme.titleMedium),
        SwitchListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(labels.launchAtLogin),
          value: preferences.launchAtLogin,
          onChanged: onLaunchAtLoginChanged,
        ),
        SwitchListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(labels.showInMenuBar),
          value: preferences.showInMenuBar,
          onChanged: onShowInMenuBarChanged,
        ),
        const SizedBox(height: 16),
        Text(labels.appearance, style: Theme.of(context).textTheme.titleMedium),
        const SizedBox(height: 8),
        ListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(labels.language),
          trailing: DropdownButton<String>(
            value: languages.any((item) => item.code == preferences.language)
                ? preferences.language
                : (languages.isEmpty ? null : languages.first.code),
            onChanged: (value) {
              if (value != null) onLanguageChanged(value);
            },
            items: [
              for (final language in languages)
                DropdownMenuItem(
                  value: language.code,
                  child: Text(language.name),
                ),
            ],
          ),
        ),
        if (errorCode != null)
          Padding(
            padding: const EdgeInsets.only(bottom: 12),
            child: ListTile(
              contentPadding: EdgeInsets.zero,
              title: Text(labels.error),
              subtitle: Text(
                labels.errorMessage?.call(errorCode) ?? labels.error,
              ),
              trailing: onRetry == null
                  ? null
                  : TextButton(onPressed: onRetry, child: Text(labels.retry)),
            ),
          ),
        ListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(labels.theme),
          trailing: SegmentedButton<ThemePreference>(
            segments: [
              ButtonSegment(
                value: ThemePreference.light,
                label: Text(labels.light),
              ),
              ButtonSegment(
                value: ThemePreference.dark,
                label: Text(labels.dark),
              ),
              ButtonSegment(
                value: ThemePreference.system,
                label: Text(labels.system),
              ),
            ],
            selected: {preferences.themeMode},
            onSelectionChanged: (selection) {
              if (selection.isNotEmpty) onThemeModeChanged(selection.first);
            },
          ),
        ),
        if (labels.input.isNotEmpty) ...[
          const SizedBox(height: 16),
          Text(labels.input, style: Theme.of(context).textTheme.titleMedium),
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(labels.submitEnter),
            value: preferences.inputSubmitMode == InputSubmitMode.enter,
            onChanged: onInputSubmitModeChanged == null
                ? null
                : (value) => onInputSubmitModeChanged!(
                    value
                        ? InputSubmitMode.enter
                        : InputSubmitMode.commandEnter,
                  ),
          ),
        ],
        if (labels.translationBehaviour.isNotEmpty) ...[
          const SizedBox(height: 16),
          Text(
            labels.translationBehaviour,
            style: Theme.of(context).textTheme.titleMedium,
          ),
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(labels.autoCopyOcr),
            value: preferences.autoCopyDetectedText,
            onChanged: onAutoCopyChanged,
          ),
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(labels.doubleClickCopy),
            value: preferences.doubleClickCopyResult,
            onChanged: onDoubleClickCopyChanged,
          ),
        ],
        if (labels.commonLanguages.isNotEmpty &&
            translationLanguages.isNotEmpty) ...[
          const SizedBox(height: 16),
          Text(
            labels.commonLanguages,
            style: Theme.of(context).textTheme.titleMedium,
          ),
          const SizedBox(height: 8),
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [
              for (final language in translationLanguages)
                FilterChip(
                  label: Text(language.name),
                  selected: preferences.commonLanguages.contains(language.code),
                  onSelected: onCommonLanguagesChanged == null
                      ? null
                      : (selected) {
                          final next = [...preferences.commonLanguages];
                          if (selected) {
                            next.add(language.code);
                          } else {
                            next.remove(language.code);
                          }
                          onCommonLanguagesChanged!(next);
                        },
                ),
            ],
          ),
        ],
      ],
    );
  }
}
