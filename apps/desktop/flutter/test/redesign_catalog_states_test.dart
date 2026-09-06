import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/src/catalog/catalog.dart';
import 'package:linguaray_desktop/src/shared/settings_labels.dart';
import 'package:linguaray_desktop/src/shared/settings_page.dart';
import 'package:linguaray_ui/linguaray_ui.dart';

void main() {
  for (final size in [const Size(1000, 700), const Size(840, 560)]) {
    for (final brightness in Brightness.values) {
      testWidgets('all settings share their frame at $size / $brightness', (
        tester,
      ) async {
        tester.view
          ..devicePixelRatio = 1
          ..physicalSize = size;
        addTearDown(tester.view.reset);
        Offset? origin;
        for (final section in SettingsSection.values) {
          await tester.pumpWidget(
            MaterialApp(
              theme: LinguaRayMaterialTheme.forBrightness(brightness),
              home: SettingsCatalogPreview(
                key: ValueKey(section),
                section: section,
              ),
            ),
          );
          await tester.pumpAndSettle();
          expect(tester.takeException(), isNull, reason: section.name);
          final page = find.byType(SettingsPage);
          expect(page, findsOneWidget, reason: section.name);
          final frame = tester.widget<SettingsPage>(page);
          final title = find
              .descendant(of: page, matching: find.text(frame.title))
              .first;
          final position = tester.getTopLeft(title);
          origin ??= position;
          expect(position, origin, reason: 'Title position: ${section.name}');
          final material = tester.widget<Material>(
            find.descendant(of: page, matching: find.byType(Material)).first,
          );
          expect(
            material.color,
            LinguaRayMaterialTheme.forBrightness(brightness)
                .colorScheme
                .surfaceContainerLowest,
          );
        }
      });
    }
  }
}
