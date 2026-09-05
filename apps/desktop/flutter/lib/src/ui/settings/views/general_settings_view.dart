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
      padding: const EdgeInsets.fromLTRB(32, 36, 32, 36),
      children: [
        if (pageTitle != null) ...[
          Text(pageTitle!, style: Theme.of(context).textTheme.titleLarge),
          const SizedBox(height: 22),
        ],
        LayoutBuilder(
          builder: (context, constraints) {
            final appearance = _SettingsBlock(
              title: labels.appearance,
              icon: Icons.palette_outlined,
              children: [
                Row(
                  children: [
                    for (final (index, mode)
                        in ThemePreference.values.indexed) ...[
                      if (index > 0) const SizedBox(width: 10),
                      Expanded(
                        child: _ThemeChoice(
                          mode: mode,
                          label: switch (mode) {
                            ThemePreference.light => labels.light,
                            ThemePreference.dark => labels.dark,
                            ThemePreference.system => labels.system,
                          },
                          selected: preferences.themeMode == mode,
                          onTap: () => onThemeModeChanged(mode),
                        ),
                      ),
                    ],
                  ],
                ),
                const SizedBox(height: 24),
                const Divider(),
                const SizedBox(height: 12),
                Row(
                  children: [
                    Expanded(child: Text(labels.language)),
                    DropdownButton<String>(
                      underline: const SizedBox.shrink(),
                      style: Theme.of(context).textTheme.bodyMedium,
                      value:
                          languages.any(
                            (item) => item.code == preferences.language,
                          )
                          ? preferences.language
                          : languages.firstOrNull?.code,
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
                  ],
                ),
              ],
            );
            final startup = _SettingsBlock(
              title: labels.startup,
              icon: Icons.power_settings_new_rounded,
              children: [
                _ToggleSetting(
                  title: labels.launchAtLogin,
                  value: preferences.launchAtLogin,
                  onChanged: onLaunchAtLoginChanged,
                ),
                if (!kIsWindows) ...[
                  const Divider(),
                  _ToggleSetting(
                    title: labels.showInMenuBar,
                    value: preferences.showInMenuBar,
                    onChanged: onShowInMenuBarChanged,
                  ),
                ],
              ],
            );
            if (constraints.maxWidth < 650) {
              return Column(
                children: [appearance, const SizedBox(height: 20), startup],
              );
            }
            return Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Expanded(flex: 6, child: appearance),
                const SizedBox(width: 20),
                Expanded(flex: 5, child: startup),
              ],
            );
          },
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

class _SettingsBlock extends StatelessWidget {
  const _SettingsBlock({
    required this.title,
    required this.icon,
    required this.children,
  });
  final String title;
  final IconData icon;
  final List<Widget> children;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Container(
      padding: const EdgeInsets.all(22),
      decoration: BoxDecoration(
        color: theme.colorScheme.surface,
        borderRadius: BorderRadius.circular(18),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              Icon(icon, size: 19, color: theme.colorScheme.onSurfaceVariant),
              const SizedBox(width: 10),
              Expanded(child: Text(title, style: theme.textTheme.titleMedium)),
            ],
          ),
          const SizedBox(height: 24),
          ...children,
        ],
      ),
    );
  }
}

class _ToggleSetting extends StatelessWidget {
  const _ToggleSetting({
    required this.title,
    required this.value,
    required this.onChanged,
  });
  final String title;
  final bool value;
  final ValueChanged<bool> onChanged;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 10),
    child: Row(
      children: [
        Expanded(child: Text(title)),
        const SizedBox(width: 12),
        Switch(value: value, onChanged: onChanged),
      ],
    ),
  );
}

class _ThemeChoice extends StatelessWidget {
  const _ThemeChoice({
    required this.mode,
    required this.label,
    required this.selected,
    required this.onTap,
  });
  final ThemePreference mode;
  final String label;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Semantics(
      selected: selected,
      button: true,
      child: InkWell(
        key: ValueKey('theme-choice-${mode.name}'),
        onTap: onTap,
        borderRadius: BorderRadius.circular(10),
        child: Column(
          children: [
            Container(
              height: 82,
              padding: const EdgeInsets.all(4),
              decoration: BoxDecoration(
                borderRadius: BorderRadius.circular(10),
                border: Border.all(
                  color: selected
                      ? theme.colorScheme.primary
                      : theme.colorScheme.outlineVariant,
                  width: selected ? 2 : 1,
                ),
              ),
              child: ClipRRect(
                borderRadius: BorderRadius.circular(5),
                child: CustomPaint(
                  painter: _ThemePreview(mode),
                  child: const SizedBox.expand(),
                ),
              ),
            ),
            const SizedBox(height: 9),
            Text(
              label,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: theme.textTheme.labelMedium?.copyWith(
                color: selected
                    ? theme.colorScheme.primary
                    : theme.colorScheme.onSurfaceVariant,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// Miniatures communicate the actual rail / content hierarchy of each theme.
class _ThemePreview extends CustomPainter {
  const _ThemePreview(this.mode);
  final ThemePreference mode;

  @override
  void paint(Canvas canvas, Size size) {
    void draw(bool dark) {
      final paint = Paint();
      canvas.drawRect(
        Offset.zero & size,
        paint..color = dark ? const Color(0xFF2E2925) : Colors.white,
      );
      canvas.drawRect(
        Rect.fromLTWH(0, 0, size.width * .19, size.height),
        paint..color = dark ? const Color(0xFF24211F) : const Color(0xFFF0EDE9),
      );
      canvas.drawRRect(
        RRect.fromRectAndRadius(
          Rect.fromLTWH(4, 10, size.width * .19 - 8, 12),
          const Radius.circular(2),
        ),
        paint..color = const Color(0xFFE9A287),
      );
      for (var i = 0; i < 3; i++) {
        canvas.drawRRect(
          RRect.fromRectAndRadius(
            Rect.fromLTWH(
              size.width * .29,
              16 + i * 9,
              size.width * (i == 0 ? .49 : .37),
              3,
            ),
            const Radius.circular(1),
          ),
          paint
            ..color = dark ? const Color(0xFF7E7064) : const Color(0xFFDDD6CE),
        );
      }
    }

    draw(mode == ThemePreference.dark);
    if (mode == ThemePreference.system) {
      canvas.save();
      canvas.clipRect(
        Rect.fromLTWH(size.width / 2, 0, size.width / 2, size.height),
      );
      draw(true);
      canvas.restore();
    }
  }

  @override
  bool shouldRepaint(_ThemePreview oldDelegate) => oldDelegate.mode != mode;
}
