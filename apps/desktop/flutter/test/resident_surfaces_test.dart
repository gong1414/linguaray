import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/src/ui/catalog/catalog.dart';
import 'package:linguaray_desktop/src/ui/settings/settings_labels.dart';
import 'package:linguaray_ui/linguaray_ui.dart' show LinguaRayMaterialTheme;

void main() {
  testWidgets('quick translate empty state stays within 396 width', (
    tester,
  ) async {
    tester.view
      ..devicePixelRatio = 1
      ..physicalSize = const Size(396, 640);
    addTearDown(tester.view.reset);

    await tester.pumpWidget(
      MaterialApp(
        theme: LinguaRayMaterialTheme.light(),
        home: const Center(
          child: QuickTranslateCatalogPreview(
            scenario: CatalogQuickScenario.empty,
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('快捷翻译'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('settings general and service empty states render', (
    tester,
  ) async {
    tester.view
      ..devicePixelRatio = 1
      ..physicalSize = const Size(840, 560);
    addTearDown(tester.view.reset);

    await tester.pumpWidget(
      MaterialApp(
        theme: LinguaRayMaterialTheme.light(),
        home: const SettingsCatalogPreview(section: SettingsSection.general),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('登录时启动'), findsOneWidget);

    await tester.pumpWidget(
      MaterialApp(
        theme: LinguaRayMaterialTheme.dark(),
        home: const SettingsCatalogPreview(
          section: SettingsSection.translationServices,
          servicesEmpty: true,
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('配置提供商'), findsWidgets);
    expect(tester.takeException(), isNull);
  });
}
