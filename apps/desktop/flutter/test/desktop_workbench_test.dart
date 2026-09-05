import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/src/app/navigation/settings_shell_view.dart';
import 'package:linguaray_desktop/src/catalog/catalog.dart';
import 'package:linguaray_desktop/src/shared/i18n_labels.dart';
import 'package:linguaray_desktop/src/shared/settings_labels.dart';
import 'package:linguaray_ui/linguaray_ui.dart' show LinguaRayMaterialTheme;

void main() {
  testWidgets('work areas keep every settings destination reachable', (
    tester,
  ) async {
    var selected = SettingsSection.translation;
    await tester.pumpWidget(
      MaterialApp(
        theme: LinguaRayMaterialTheme.light(),
        home: StatefulBuilder(
          builder: (context, setState) => SettingsShellView(
            labels: settingsShellLabels(),
            section: selected,
            onSectionSelected: (value) => setState(() => selected = value),
            child: Text('page:${selected.name}'),
          ),
        ),
      ),
    );
    const areas = {
      SettingsSection.translation: [
        SettingsSection.translation,
        SettingsSection.translationServices,
      ],
      SettingsSection.history: [
        SettingsSection.history,
        SettingsSection.favorites,
        SettingsSection.glossary,
        SettingsSection.vocabulary,
      ],
      SettingsSection.ocr: [SettingsSection.ocr, SettingsSection.ocrServices],
      SettingsSection.general: [
        SettingsSection.general,
        SettingsSection.permissions,
        SettingsSection.dataTransfer,
        SettingsSection.integration,
        SettingsSection.updates,
        SettingsSection.about,
      ],
    };
    for (final entry in areas.entries) {
      await tester.tap(find.byKey(ValueKey('work-area-${entry.key.name}')));
      await tester.pumpAndSettle();
      for (final page in entry.value) {
        final tab = find.byKey(ValueKey('settings-page-${page.name}'));
        await tester.ensureVisible(tab);
        await tester.tap(tab);
        await tester.pumpAndSettle();
        expect(find.text('page:${page.name}'), findsOneWidget);
      }
    }
    expect(tester.takeException(), isNull);
  });

  for (final width in [720.0, 396.0]) {
    testWidgets('translation reading layout adapts at $width', (tester) async {
      tester.view
        ..devicePixelRatio = 1
        ..physicalSize = Size(width, 760);
      addTearDown(tester.view.reset);
      await tester.pumpWidget(
        MaterialApp(
          theme: LinguaRayMaterialTheme.light(),
          home: Center(
            child: QuickTranslateCatalogPreview(
              scenario: CatalogQuickScenario.success,
              width: width,
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      final source = tester.getRect(
        find.byKey(const ValueKey('quick-source-pane')),
      );
      final result = tester.getRect(
        find.byKey(const ValueKey('quick-result-pane')),
      );
      if (width >= 600) {
        expect(result.left, greaterThan(source.right - 1));
        expect(result.top, source.top);
      } else {
        expect(result.top, greaterThanOrEqualTo(source.bottom));
        expect(result.left, source.left);
      }
      expect(tester.takeException(), isNull);
    });
  }

  testWidgets('short display can scroll to result actions', (tester) async {
    tester.view
      ..devicePixelRatio = 1
      ..physicalSize = const Size(396, 260);
    addTearDown(tester.view.reset);
    await tester.pumpWidget(
      MaterialApp(
        theme: LinguaRayMaterialTheme.light(),
        home: const Center(
          child: QuickTranslateCatalogPreview(
            scenario: CatalogQuickScenario.longResult,
            width: 396,
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    await tester.ensureVisible(find.text('复制'));
    await tester.pumpAndSettle();
    expect(find.text('复制').hitTestable(), findsOneWidget);
    expect(tester.takeException(), isNull);
  });
}
