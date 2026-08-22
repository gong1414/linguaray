import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/src/theme/app_theme.dart';
import 'package:linguaray_desktop/src/widgets/ui.dart'
    show DesignFont, DesignThemeProvider, DesignTokens, DesignTypography;
import 'package:linguaray_desktop/widgetbook.dart';

import 'support/golden_comparator.dart';

const _fixedTypography = DesignTypography(
  display: DesignFont(family: 'Golden Sans'),
  sans: DesignFont(family: 'Golden Sans'),
  cjk: DesignFont(family: 'Golden Sans'),
  label: DesignFont(family: 'Golden Sans'),
  mono: DesignFont(family: 'Golden Mono'),
);

void main() {
  installGoldenComparator();

  setUpAll(() async {
    await _loadFont('Golden Sans', 'resources/fonts/MiSans-Regular.ttf');
    await _loadFont('Golden Mono', 'resources/fonts/RobotoMono-Regular.ttf');
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
        testWidgets('${entry.key} ${brightness.name} $platform',
            (tester) async {
          tester.view.devicePixelRatio = 1;
          tester.view.physicalSize = const Size(1000, 700);
          addTearDown(tester.view.reset);

          final DesignTokens tokens = tokensFor(brightness).copyWith(
            typography: _fixedTypography,
          );
          await tester.pumpWidget(
            Theme(
              data: appThemeData(tokens),
              child: DesignThemeProvider(
                tokens: tokens,
                child: Directionality(
                  textDirection: TextDirection.ltr,
                  child: RepaintBoundary(
                    key: const ValueKey('catalog-golden'),
                    child: SizedBox(
                      width: 1000,
                      height: 700,
                      child: ColoredBox(
                        color: tokens.colors.canvas,
                        child: Center(
                          child: entry.value,
                        ),
                      ),
                    ),
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
