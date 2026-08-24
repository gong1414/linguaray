import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/ui/settings/settings_labels.dart';
import 'package:linguaray_desktop/src/ui/settings/views/general_settings_view.dart';

void main() {
  testWidgets('current general view exposes configured translation targets', (
    tester,
  ) async {
    var managed = false;
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: GeneralSettingsView(
            labels: _labels,
            preferences: const GeneralPreferences(
              launchAtLogin: false,
              showInMenuBar: true,
              language: 'en',
              themeMode: ThemePreference.system,
              translationTargets: [
                TranslationTargetRule(source: 'auto', target: 'zh-Hans'),
              ],
            ),
            languages: const [LanguageChoice(code: 'en', name: 'English')],
            translationLanguages: const [
              LanguageChoice(code: 'en', name: 'English'),
              LanguageChoice(code: 'zh-Hans', name: '简体中文'),
            ],
            onLaunchAtLoginChanged: (_) {},
            onShowInMenuBarChanged: (_) {},
            onLanguageChanged: (_) {},
            onThemeModeChanged: (_) {},
            onManageTranslationTargets: () => managed = true,
          ),
        ),
      ),
    );

    expect(find.text('Auto detect → 简体中文'), findsOneWidget);
    await tester.ensureVisible(find.text('Manage targets'));
    await tester.tap(find.text('Manage targets'));
    expect(managed, isTrue);
  });
}

const _labels = GeneralSettingsLabels(
  startup: 'Startup',
  launchAtLogin: 'Launch at login',
  showInMenuBar: 'Show in menu bar',
  appearance: 'Appearance',
  language: 'Language',
  theme: 'Theme',
  light: 'Light',
  dark: 'Dark',
  system: 'System',
  translationTargets: 'Translation targets',
  translationTargetsHint: 'Configure language pairs.',
  noTranslationTargets: 'No targets',
  manageTranslationTargets: 'Manage targets',
  autoDetect: 'Auto detect',
);
