import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/widgetbook.dart';
import 'package:linguaray_ui/linguaray_ui.dart' show LinguaRayMaterialTheme;

import 'support/golden_comparator.dart';

void main() {
  installGoldenComparator();

  setUpAll(() async {
    await _loadFont('Golden Sans', 'resources/fonts/MiSans-Regular.ttf');
    await _loadFont('MiSans', 'resources/fonts/MiSans-Regular.ttf');
    await _loadFont('Golden Mono', 'resources/fonts/RobotoMono-Regular.ttf');
    await _loadFont('MaterialIcons', 'fonts/MaterialIcons-Regular.otf');
    await _loadFont(
      'packages/fluentui_system_icons/FluentSystemIcons-Regular',
      'packages/fluentui_system_icons/fonts/FluentSystemIcons-Regular.ttf',
    );
  });

  final targets = Platform.isWindows
      ? const [TargetPlatform.windows]
      : const [TargetPlatform.macOS];
  for (final target in targets) {
    final platform = target == TargetPlatform.windows ? 'windows' : 'macos';
    final states = buildCatalogGoldenStates(platform: target);
    for (final brightness in Brightness.values) {
      for (final entry in states.entries) {
        testWidgets('${entry.key} ${brightness.name} $platform', (
          tester,
        ) async {
          tester.view.devicePixelRatio = 1;
          tester.view.physicalSize = const Size(1000, 700);
          addTearDown(tester.view.reset);

          final baseTheme = LinguaRayMaterialTheme.forBrightness(brightness);
          final fixedTextTheme = baseTheme.textTheme.apply(
            fontFamily: 'Golden Sans',
          );
          final theme = baseTheme.copyWith(
            platform: target,
            textTheme: fixedTextTheme,
            listTileTheme: baseTheme.listTileTheme.copyWith(
              titleTextStyle: fixedTextTheme.titleMedium,
              subtitleTextStyle: fixedTextTheme.bodyMedium?.copyWith(
                color: baseTheme.colorScheme.onSurfaceVariant,
              ),
            ),
            navigationRailTheme: baseTheme.navigationRailTheme.copyWith(
              selectedLabelTextStyle: fixedTextTheme.labelMedium?.copyWith(
                color: baseTheme.colorScheme.primary,
                fontWeight: FontWeight.w600,
              ),
              unselectedLabelTextStyle: fixedTextTheme.labelMedium?.copyWith(
                color: baseTheme.colorScheme.onSurfaceVariant,
              ),
            ),
            chipTheme: baseTheme.chipTheme.copyWith(
              labelStyle: fixedTextTheme.labelMedium,
            ),
            filledButtonTheme: FilledButtonThemeData(
              style: baseTheme.filledButtonTheme.style?.copyWith(
                textStyle: WidgetStatePropertyAll(fixedTextTheme.labelLarge),
              ),
            ),
            outlinedButtonTheme: OutlinedButtonThemeData(
              style: baseTheme.outlinedButtonTheme.style?.copyWith(
                textStyle: WidgetStatePropertyAll(fixedTextTheme.labelLarge),
              ),
            ),
          );
          await tester.pumpWidget(
            MaterialApp(
              debugShowCheckedModeBanner: false,
              theme: theme,
              home: RepaintBoundary(
                key: const ValueKey('catalog-golden'),
                child: SizedBox(
                  width: 1000,
                  height: 700,
                  child: ColoredBox(
                    color: theme.scaffoldBackgroundColor,
                    child: Center(child: entry.value),
                  ),
                ),
              ),
            ),
          );
          await tester.pumpAndSettle();

          await expectLater(
            find.byKey(const ValueKey('catalog-golden')),
            matchesGoldenFile(
              'goldens/catalog/${entry.key}_${brightness.name}_$platform.png',
            ),
          );
        });
      }
    }
  }
}

Future<void> _loadFont(String family, String asset) async {
  final bytes = await rootBundle.load(asset);
  await (FontLoader(family)..addFont(Future.value(bytes))).load();
}
