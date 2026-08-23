import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/src/ui/settings/settings_labels.dart';
import 'package:linguaray_desktop/widgetbook.dart';
import 'package:linguaray_ui/linguaray_ui.dart' show LinguaRayMaterialTheme;

void main() {
  testWidgets('redesigned catalog states render without overflowing', (
    tester,
  ) async {
    tester.view
      ..devicePixelRatio = 1
      ..physicalSize = const Size(1000, 700);
    addTearDown(tester.view.reset);

    for (final child in const [
      HistoryCatalogPreview(empty: true),
      HistoryCatalogPreview(empty: false),
      UpdatesCatalogPreview(),
      SettingsCatalogPreview(section: SettingsSection.translation),
    ]) {
      await tester.pumpWidget(
        MaterialApp(theme: LinguaRayMaterialTheme.light(), home: child),
      );
      await tester.pump();
      expect(tester.takeException(), isNull);
    }
  });
}
