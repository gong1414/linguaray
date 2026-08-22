import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/src/ui/chrome/workbench_shell_view.dart';
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
      GlossaryCatalogPreview(empty: true),
      VocabularyCatalogPreview(empty: true),
      UpdatesCatalogPreview(),
      CatalogShellPreview(
        chrome: WindowChromeKind.windows,
        destination: WorkbenchDestinationId.history,
        child: HistoryCatalogPreview(empty: false),
      ),
    ]) {
      await tester.pumpWidget(
        MaterialApp(theme: LinguaRayMaterialTheme.light(), home: child),
      );
      await tester.pump();
      expect(tester.takeException(), isNull);
    }
  });
}
