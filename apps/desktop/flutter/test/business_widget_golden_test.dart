// Per-block goldens for the 业务组件 — the widgets that compose the design
// system's atoms into LinguaRay's own vocabulary.
//
// The atoms' own goldens live in `packages/ui_flutter/test/golden_test.dart`;
// this suite is its twin on this side of the boundary, and shares its harness:
// each block renders on its own at DPR 1 into a few tens of kilobytes, so a
// regression names the block it broke and the image is small enough to look at.
// Refresh with `flutter test --update-goldens` after a deliberate visual change.
//
// The faces are the real ones, so a host without them skips the suite rather
// than reporting a wall of false diffs — `design_widget_alignment_test.dart` is
// the part that holds everywhere.

import 'dart:io';

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/src/widgets/avatar.dart';
import 'package:linguaray_desktop/src/widgets/blocks.dart';
import 'package:linguaray_desktop/src/widgets/data_display.dart';
import 'package:linguaray_desktop/src/widgets/list_tile.dart';
import 'package:linguaray_desktop/src/widgets/swap_pair.dart';
import 'package:linguaray_desktop/src/widgets/ui.dart';
import 'package:linguaray_ui/linguaray_ui.dart';
import 'package:linguaray_ui/testing.dart';

import 'support/golden_comparator.dart';

void main() {
  installGoldenComparator();

  final missing = [
    for (final entry in goldenHostFaces.entries)
      if (!File(entry.value).existsSync()) entry.key,
  ];

  group('goldens', () {
    setUpAll(loadGoldenFonts);

    /// Renders [child] at [width] on the theme's window surface and compares it
    /// with `goldens/<name>.png`.
    Future<void> expectGolden(
      WidgetTester tester,
      String name,
      Widget child, {
      double width = 380,
      DesignThemeName theme = DesignThemeName.studioLight,
    }) async {
      tester.view.devicePixelRatio = 1;
      tester.view.physicalSize = const Size(1200, 1200);
      addTearDown(tester.view.reset);

      await tester.pumpWidget(
        DesignThemeProvider(
          theme: theme,
          tokens: theme.tokens.copyWith(typography: goldenTypography),
          child: Directionality(
            textDirection: TextDirection.ltr,
            child: Align(
              alignment: Alignment.topLeft,
              child: RepaintBoundary(
                key: const ValueKey('golden'),
                child: Builder(
                  builder: (context) => Container(
                    width: width,
                    color: context.colors.window,
                    padding: const EdgeInsets.all(16),
                    child: child,
                  ),
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();

      await expectLater(
        find.byKey(const ValueKey('golden')),
        matchesGoldenFile('goldens/$name.png'),
      );
    }

    Widget column(List<Widget> children, {double gap = 10}) => Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        for (var i = 0; i < children.length; i++) ...[
          if (i > 0) SizedBox(height: gap),
          children[i],
        ],
      ],
    );

    testWidgets('translation blocks', (tester) async {
      await expectGolden(
        tester,
        'translation_blocks',
        width: 460,
        column([
          const TextBlock(
            label: Text('原文'),
            meta: Text('⌥⏎ 重译'),
            child: Text('Attention is all you need.'),
          ),
          const HighlightBlock(
            rule: HighlightRule.top,
            label: Text('内置模型 · 首选译文'),
            meta: Text('2 处术语已对齐'),
            child: Text('注意力就是你所需要的一切。'),
          ),
        ], gap: 0),
      );
    });

    testWidgets('list rows', (tester) async {
      await expectGolden(
        tester,
        'list_rows',
        width: 420,
        column([
          ListTile(
            leading: const Avatar(label: 'C', color: Color(0xFFD97757)),
            title: const Text('Claude'),
            meta: const Text('claude-sonnet-4-5 · 密钥有效'),
            badge: const Badge(child: Text('默认')),
            trailing: [Switch(checked: true, onChanged: (_) {})],
          ),
          ListTile(
            variant: ListTileVariant.row,
            tone: ListTileTone.warn,
            leading: const Avatar(label: 'D', color: Color(0xFF3A7BFD)),
            title: const Text('DeepL'),
            meta: const Text('密钥已过期'),
            onPressed: () {},
          ),
        ]),
      );
    });

    testWidgets('language capsule and gauge', (tester) async {
      await expectGolden(
        tester,
        'capsule_and_gauge',
        column([
          SwapPair(
            start: 'English',
            end: '简体中文',
            onSwap: () {},
            onStartPressed: () {},
            onEndPressed: () {},
          ),
          const SegmentGauge(filled: 2, partial: true),
        ]),
      );
    });
  }, skip: missing.isEmpty ? false : 'host is missing ${missing.join(', ')}');
}
