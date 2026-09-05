import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/src/i18n/i18n.dart';
import 'package:linguaray_desktop/widgetbook.dart';
import 'package:linguaray_ui/linguaray_ui.dart' show LinguaRayMaterialTheme;
import 'package:linguaray_ui/testing.dart' show loadGoldenFonts;

import 'support/golden_comparator.dart';

void main() {
  installGoldenComparator();

  setUpAll(() async {
    await LocaleSettings.setLocaleRaw('zh-Hans');
    await loadGoldenFonts();
    await _loadFont('MaterialIcons', 'fonts/MaterialIcons-Regular.otf');
  });

  // Native font files determine these baselines. Rendering a Windows theme
  // with macOS fonts must not overwrite the Windows snapshots.
  final nativePlatform = Platform.isWindows ? 'windows' : 'macos';
  final requestedPlatform = Platform.environment['LINGUARAY_GOLDEN_PLATFORM'];
  if (requestedPlatform != null && requestedPlatform != nativePlatform) {
    throw StateError(
      'Refresh $requestedPlatform goldens on a $requestedPlatform host.',
    );
  }
  final targets = Platform.isWindows
      ? const [TargetPlatform.windows]
      : const [TargetPlatform.macOS];
  for (final target in targets) {
    final platform = target == TargetPlatform.windows ? 'windows' : 'macos';
    final states = buildCatalogGoldenStates();
    for (final brightness in Brightness.values) {
      for (final entry in states.entries) {
        testWidgets('${entry.key} ${brightness.name} $platform', (
          tester,
        ) async {
          tester.view.devicePixelRatio = 1;
          tester.view.physicalSize = const Size(1000, 700);
          addTearDown(tester.view.reset);

          final theme = LinguaRayMaterialTheme.forBrightness(
            brightness,
            platform: target,
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
