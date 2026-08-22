import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/widgetbook.dart';
import 'package:linguaray_ui/linguaray_ui.dart' show LinguaRayMaterialTheme;

void main() {
  testWidgets('compact desktop layout stays usable and updates its input', (
    tester,
  ) async {
    tester.view
      ..devicePixelRatio = 1
      ..physicalSize = const Size(620, 760);
    addTearDown(tester.view.reset);

    await tester.pumpWidget(
      MaterialApp(
        theme: LinguaRayMaterialTheme.light(),
        home: const TranslationCatalogPreview(
          scenario: CatalogTranslationScenario.success,
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('输入翻译'), findsOneWidget);
    expect(find.text('稳定的界面应该让每一种状态都可检查。'), findsOneWidget);
    expect(
      tester.getTopLeft(find.byKey(const ValueKey('translation-result'))).dy,
      greaterThan(
        tester
            .getTopLeft(find.byKey(const ValueKey('translation-source-input')))
            .dy,
      ),
    );
    expect(tester.takeException(), isNull);

    final input = find.byKey(const ValueKey('translation-source-input'));
    await tester.enterText(input, 'Hello from a compact window.');
    await tester.pump();
    expect(find.text('Hello from a compact window.'), findsOneWidget);

    await tester.tap(find.byKey(const ValueKey('clear-source')));
    await tester.pump();
    expect(find.text('Hello from a compact window.'), findsNothing);
    expect(tester.takeException(), isNull);
  });
}
