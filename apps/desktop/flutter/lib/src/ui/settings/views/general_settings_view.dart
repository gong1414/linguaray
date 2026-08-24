import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../utils/platform_util.dart';
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
    this.onManageTranslationTargets,
    this.showTranslationSections = true,
    this.pageTitle,
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
  final VoidCallback? onManageTranslationTargets;
  final bool showTranslationSections;
  final String? pageTitle;

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.fromLTRB(30, 48, 30, 36),
      children: [
        if (pageTitle != null) ...[
          Text(pageTitle!, style: Theme.of(context).textTheme.titleLarge),
          const SizedBox(height: 22),
        ],
        Text(labels.startup, style: Theme.of(context).textTheme.titleMedium),
        const SizedBox(height: 8),
        _GeneralCard(
          children: [
            SwitchListTile(
              contentPadding: const EdgeInsets.symmetric(horizontal: 12),
              title: Text(labels.launchAtLogin),
              value: preferences.launchAtLogin,
              onChanged: onLaunchAtLoginChanged,
            ),
            if (!kIsWindows)
              SwitchListTile(
                contentPadding: const EdgeInsets.symmetric(horizontal: 12),
                title: Text(labels.showInMenuBar),
                value: preferences.showInMenuBar,
                onChanged: onShowInMenuBarChanged,
              ),
          ],
        ),
        const SizedBox(height: 24),
        Text(labels.appearance, style: Theme.of(context).textTheme.titleMedium),
        const SizedBox(height: 8),
        _GeneralCard(
          children: [
            ListTile(
              contentPadding: const EdgeInsets.symmetric(horizontal: 12),
              title: Text(labels.language),
              trailing: DropdownButton<String>(
                value:
                    languages.any((item) => item.code == preferences.language)
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
              contentPadding: const EdgeInsets.symmetric(horizontal: 12),
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
                  if (selection.isNotEmpty) {
                    onThemeModeChanged(selection.first);
                  }
                },
              ),
            ),
          ],
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
        if (showTranslationSections && labels.input.isNotEmpty) ...[
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
        if (showTranslationSections &&
            labels.translationBehaviour.isNotEmpty) ...[
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
        if (showTranslationSections &&
            labels.translationTargets.isNotEmpty) ...[
          const SizedBox(height: 16),
          Row(
            children: [
              Expanded(
                child: Text(
                  labels.translationTargets,
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ),
              TextButton.icon(
                onPressed: onManageTranslationTargets,
                icon: const Icon(Icons.tune_rounded, size: 18),
                label: Text(labels.manageTranslationTargets),
              ),
            ],
          ),
          Text(
            labels.translationTargetsHint,
            style: Theme.of(context).textTheme.bodySmall,
          ),
          const SizedBox(height: 6),
          if (preferences.translationTargets.isEmpty)
            ListTile(
              contentPadding: EdgeInsets.zero,
              title: Text(labels.noTranslationTargets),
            )
          else
            for (final target in preferences.translationTargets)
              ListTile(
                contentPadding: EdgeInsets.zero,
                leading: Icon(
                  target.enabled
                      ? Icons.arrow_forward_rounded
                      : Icons.pause_rounded,
                  size: 18,
                ),
                title: Text(
                  '${_languageName(target.source, source: true)} → '
                  '${_languageName(target.target)}',
                ),
              ),
        ],
        if (showTranslationSections &&
            labels.commonLanguages.isNotEmpty &&
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

  String _languageName(String code, {bool source = false}) {
    if (source && code == 'auto') return labels.autoDetect;
    for (final language in translationLanguages) {
      if (language.code == code) return language.name;
    }
    return code;
  }
}

class _GeneralCard extends StatelessWidget {
  const _GeneralCard({required this.children});

  final List<Widget> children;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Material(
      color: theme.colorScheme.surfaceContainerLowest,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: theme.colorScheme.outlineVariant),
      ),
      clipBehavior: Clip.antiAlias,
      child: Column(
        children: [
          for (final (index, child) in children.indexed) ...[
            if (index > 0)
              Divider(
                height: 1,
                indent: 12,
                endIndent: 12,
                color: theme.dividerColor.withValues(alpha: 0.5),
              ),
            child,
          ],
        ],
      ),
    );
  }
}
