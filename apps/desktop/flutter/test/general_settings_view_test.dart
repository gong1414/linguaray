import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/features/preferences/general_settings_view.dart';
import 'package:linguaray_desktop/src/shared/settings_labels.dart';
import 'package:linguaray_ui/linguaray_ui.dart';

void main() {
  testWidgets(
    'general preferences use the active theme choices and startup controls',
    (tester) async {
      ThemePreference? selected;
      bool? launchAtLogin;
      await tester.pumpWidget(
        MaterialApp(
          theme: LinguaRayMaterialTheme.light(),
          home: GeneralSettingsView(
            pageTitle: 'General',
            labels: _labels,
            preferences: const GeneralPreferences(
              launchAtLogin: false,
              showInMenuBar: true,
              language: 'en',
              themeMode: ThemePreference.system,
            ),
            languages: const [LanguageChoice(code: 'en', name: 'English')],
            onLaunchAtLoginChanged: (value) => launchAtLogin = value,
            onShowInMenuBarChanged: (_) {},
            onLanguageChanged: (_) {},
            onThemeModeChanged: (value) => selected = value,
          ),
        ),
      );
      await tester.tap(find.byKey(const ValueKey('theme-choice-light')));
      expect(selected, ThemePreference.light);
      await tester.tap(find.byKey(const ValueKey('theme-choice-dark')));
      expect(selected, ThemePreference.dark);
      await tester.tap(find.byType(Switch).first);
      expect(launchAtLogin, isTrue);
      expect(tester.takeException(), isNull);
    },
  );
}

const _labels = GeneralSettingsLabels(
  startup: 'Startup',
  launchAtLogin: 'Launch at login',
  showInMenuBar: 'Show in menu bar',
  appearance: 'Appearance',
  language: 'Language',
  light: 'Light',
  dark: 'Dark',
  system: 'System',
);
