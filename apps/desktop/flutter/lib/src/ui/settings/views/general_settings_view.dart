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
  });

  final GeneralSettingsLabels labels;
  final GeneralPreferences preferences;
  final List<LanguageChoice> languages;
  final ValueChanged<bool> onLaunchAtLoginChanged;
  final ValueChanged<bool> onShowInMenuBarChanged;
  final ValueChanged<String> onLanguageChanged;
  final ValueChanged<ThemePreference> onThemeModeChanged;

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
      ],
    );
  }
}
