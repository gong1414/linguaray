// Per-block goldens for the widgets whose geometry the deck pins.
//
// These are assertions, not a picture book: each block renders on its own at
// DPR 1 into a few tens of kilobytes, so a regression names the block it broke
// and the image is small enough to look at. Refresh them with
// `flutter test --update-goldens` after a deliberate visual change.
//
// The faces are the native UI, CJK, mono and symbol fonts on the release host,
// registered under stable test aliases. The icon font comes out of the pub
// package, so chevrons and arrows are real glyphs too.
//
// The trade is that a host without those faces cannot reproduce the images, so
// the suite skips itself there rather than reporting a wall of false diffs.
// macOS and Windows keep separate baselines; `widget_metrics_test.dart` holds
// the platform-independent geometry.

import 'dart:io';

import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_ui/linguaray_ui.dart';
import 'package:linguaray_ui/testing.dart';

void main() {
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

    Widget row(List<Widget> children, {double gap = 8}) => Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (var i = 0; i < children.length; i++) ...[
          if (i > 0) SizedBox(width: gap),
          children[i],
        ],
      ],
    );

    testWidgets('button variants', (tester) async {
      await expectGolden(
        tester,
        'button_variants',
        column([
          row([
            Button(
              variant: ButtonVariant.primary,
              onPressed: () {},
              child: const Text('翻译'),
            ),
            Button(
              variant: ButtonVariant.secondary,
              onPressed: () {},
              child: const Text('朗读'),
            ),
            Button(onPressed: () {}, child: const Text('收藏')),
          ]),
          row([
            Button(
              variant: ButtonVariant.tint,
              onPressed: () {},
              child: const Text('对比'),
            ),
            Button(
              variant: ButtonVariant.quiet,
              onPressed: () {},
              child: const Text('设为首选'),
            ),
            Button(
              variant: ButtonVariant.plain,
              onPressed: () {},
              child: const Text('测试连接'),
            ),
            Button(
              variant: ButtonVariant.warning,
              onPressed: () {},
              child: const Text('冲突'),
            ),
          ]),
          // Disabled: the filled variants swap to the track fill, the text-only
          // ones just dim.
          row([
            const Button(
              variant: ButtonVariant.primary,
              enabled: false,
              child: Text('翻译'),
            ),
            const Button(enabled: false, child: Text('收藏')),
            const Button(
              variant: ButtonVariant.quiet,
              enabled: false,
              child: Text('设为首选'),
            ),
          ]),
        ]),
      );
    });

    testWidgets('button sizes', (tester) async {
      await expectGolden(
        tester,
        'button_sizes',
        row([
          for (final size in ButtonSize.values)
            Button(
              size: size,
              variant: ButtonVariant.primary,
              shortcut: const Text('⏎'),
              onPressed: () {},
              child: const Text('翻译'),
            ),
        ]),
      );
    });

    testWidgets('form controls', (tester) async {
      await expectGolden(
        tester,
        'form_controls',
        column([
          const Field(
            label: Text('API Key'),
            hint: Text('存放在系统钥匙串'),
            child: Input(placeholder: 'sk-…', mono: true),
          ),
          Select<String>(
            items: const [SelectItem(value: 'a', label: '标准')],
            value: 'a',
            onChanged: (_) {},
          ),
          SearchField(value: '', onChanged: (_) {}),
          SegmentedControl<String>(
            value: 'light',
            onChanged: (_) {},
            items: const [
              SegmentedItem(value: 'light', label: Text('浅色')),
              SegmentedItem(value: 'dark', label: Text('深色')),
              SegmentedItem(value: 'auto', label: Text('跟随')),
            ],
          ),
          row([
            const Switch(checked: true),
            const Switch(checked: false),
            const Switch(checked: true, size: SwitchSize.sm),
          ]),
          row([
            Checkbox(checked: true, onChanged: (_) {}, child: const Text('原文')),
            Radio(checked: true, onSelect: () {}, child: const Text('译文')),
          ], gap: 16),
        ]),
      );
    });

    testWidgets('chips', (tester) async {
      await expectGolden(
        tester,
        'chips',
        column([
          row([
            for (final tone in BadgeTone.values)
              Badge(tone: tone, child: const Text('默认')),
          ], gap: 6),
          row([
            const Badge(size: BadgeSize.sm, child: Text('3 SERVICES')),
            const Kbd('⌘K'),
            const Kbd('⌘F', variant: KbdVariant.key),
          ]),
          Tabs<String>(
            value: 'all',
            onChanged: (_) {},
            items: const [
              TabItem(value: 'all', label: Text('全部')),
              TabItem(value: 'starred', label: Text('收藏'), count: 64),
            ],
          ),
        ]),
      );
    });

    testWidgets('toasts', (tester) async {
      // No Spinner variant here: its repeat animation never settles under
      // pumpAndSettle. The in-flight story lives in the example app instead.
      await expectGolden(
        tester,
        'toasts',
        width: 460,
        column([
          const Toast(child: Text('已存至「下载」· 3.6 MB')),
          const Toast(tone: ToastTone.success, child: Text('已复制译文')),
          const Toast(
            tone: ToastTone.warn,
            child: Text('DeepL 超时 —— 已切换到 OpenAI 兜底'),
          ),
          Toast(
            tone: ToastTone.danger,
            onDismiss: () {},
            child: const Text('连接已断开 —— 正在重试'),
          ),
          Toast(
            action: Button(
              variant: ButtonVariant.quiet,
              onPressed: () {},
              child: const Text('撤销'),
            ),
            child: const Text('已存入生词本'),
          ),
        ]),
      );
    });

    testWidgets('sidebar group', (tester) async {
      await expectGolden(
        tester,
        'sidebar_group',
        width: 172,
        SizedBox(
          height: 220,
          child: Sidebar(
            footer: const SidebarCard(
              gap: 6,
              children: [Text('2.4.0'), Text('已是最新')],
            ),
            children: [
              SidebarGroup(
                first: true,
                label: const Text('工作区'),
                children: [
                  NavItem(
                    active: true,
                    onPressed: () {},
                    icon: const Icon(FluentIcons.translate_20_regular),
                    child: const Text('翻译'),
                  ),
                  NavItem(
                    onPressed: () {},
                    icon: const Icon(FluentIcons.book_20_regular),
                    child: const Text('术语库'),
                  ),
                ],
              ),
            ],
          ),
        ),
      );
    });

    testWidgets('surfaces in every theme', (tester) async {
      for (final theme in DesignThemeName.values) {
        await expectGolden(
          tester,
          'surfaces_${theme.id}',
          theme: theme,
          width: 300,
          column([
            const Callout(
              tone: CalloutTone.accent,
              child: Text('正在测试连接 · 已用 1.4s'),
            ),
            const Surface(tone: SurfaceTone.raised, child: Text('白卡片')),
            const Meter(label: Text('术语一致性'), value: 92),
          ]),
        );
      }
    });
  }, skip: missing.isEmpty ? false : 'host is missing ${missing.join(', ')}');
}
