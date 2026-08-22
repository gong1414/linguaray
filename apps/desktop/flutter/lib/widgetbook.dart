// ignore_for_file: depend_on_referenced_packages

import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:flutter/material.dart' hide Badge, Divider;
import 'package:widgetbook/widgetbook.dart';

import 'src/theme/app_theme.dart';
import 'src/theme/product_tokens.dart';
import 'src/widgets/navigation_item.dart';
import 'src/widgets/settings_page.dart';
import 'src/widgets/ui.dart'
    show
        Badge,
        Button,
        ButtonVariant,
        DesignThemeContext,
        DesignThemeProvider,
        DesignTokens,
        DesignTypographyStyles,
        PreferenceRow,
        PreferenceSection,
        SidebarGroup,
        Spinner,
        SpinnerSize,
        Surface,
        SurfacePadding,
        SurfaceTone;
import 'src/widgets/workbench.dart';

void main() {
  runApp(
    Widgetbook.material(
      directories: [_catalog],
      addons: [
        ThemeAddon<DesignTokens>(
          themes: [
            WidgetbookTheme(
              name: 'macOS Light',
              data: tokensFor(Brightness.light),
            ),
            WidgetbookTheme(
              name: 'macOS Dark',
              data: tokensFor(Brightness.dark),
            ),
          ],
          themeBuilder: (context, tokens, child) => Theme(
            data: appThemeData(tokens),
            child: DesignThemeProvider(tokens: tokens, child: child),
          ),
        ),
        ViewportAddon(const [
          ViewportData(
            name: 'Quick · macOS',
            width: 396,
            height: 560,
            pixelRatio: 2,
            platform: TargetPlatform.macOS,
          ),
          ViewportData(
            name: 'Workbench · macOS',
            width: 840,
            height: 560,
            pixelRatio: 2,
            platform: TargetPlatform.macOS,
          ),
          ViewportData(
            name: 'Workbench · Windows',
            width: 840,
            height: 560,
            pixelRatio: 1,
            platform: TargetPlatform.windows,
          ),
        ]),
      ],
    ),
  );
}

final _catalog = WidgetbookCategory(
  name: 'LinguaRay v2',
  children: [
    WidgetbookComponent(
      name: 'Quick translator',
      useCases: [
        WidgetbookUseCase(
          name: 'Empty · 中文',
          builder: (_) => const _QuickTranslatorState(
            title: '快捷翻译',
            body: '输入、粘贴或使用全局快捷键开始翻译',
          ),
        ),
        WidgetbookUseCase(
          name: 'Loading · English',
          builder: (_) => const _QuickTranslatorState(
            title: 'Quick translate',
            body: 'Translating…',
            loading: true,
          ),
        ),
        WidgetbookUseCase(
          name: 'Success · Long text',
          builder: (_) => const _QuickTranslatorState(
            title: '英 → 简中',
            source:
                'The quick window stays close to the pointer and remains inside the current display work area.',
            body: '快捷翻译窗靠近鼠标显示，并始终限制在当前显示器的工作区域内。',
          ),
        ),
        WidgetbookUseCase(
          name: 'Multiple providers',
          builder: (_) => const _QuickTranslatorState(
            title: '英 → 简中',
            source: 'Privacy-first translation.',
            body: '隐私优先的翻译。',
            compare: true,
          ),
        ),
        WidgetbookUseCase(
          name: 'Error',
          builder: (_) => const _QuickTranslatorState(
            title: '截图 OCR',
            body: '未配置 OCR 服务。请在设置中添加并启用一个 OCR 服务。',
            error: true,
          ),
        ),
      ],
    ),
    WidgetbookComponent(
      name: 'Workbench',
      useCases: [
        WidgetbookUseCase(
          name: 'Empty · 中文',
          builder: (_) => const _WorkbenchState(
            source: '输入或粘贴需要翻译的文本',
            translation: '译文会显示在这里',
          ),
        ),
        WidgetbookUseCase(
          name: 'Success · English',
          builder: (_) => const _WorkbenchState(
            source: 'A stable interface should make every state inspectable.',
            translation: '稳定的界面应该让每一种状态都可检查。',
          ),
        ),
      ],
    ),
    WidgetbookComponent(
      name: 'Settings',
      useCases: [
        WidgetbookUseCase(
          name: 'Providers',
          builder: (_) => const _SettingsState(),
        ),
        WidgetbookUseCase(
          name: 'Permissions · granted',
          builder: (_) => const _PermissionState(granted: true),
        ),
        WidgetbookUseCase(
          name: 'Permissions · denied',
          builder: (_) => const _PermissionState(granted: false),
        ),
      ],
    ),
  ],
);

@visibleForTesting
Map<String, Widget> buildCatalogGoldenStates({
  TargetPlatform platform = TargetPlatform.macOS,
}) =>
    {
      'quick_empty': const _QuickTranslatorState(
        title: '快捷翻译',
        body: '输入、粘贴或使用全局快捷键开始翻译',
      ),
      'workbench_success': _WorkbenchState(
        source: 'A stable interface should make every state inspectable.',
        translation: '稳定的界面应该让每一种状态都可检查。',
        platform: platform,
      ),
      'settings_providers': const _SettingsState(),
      'permissions_denied': const _PermissionState(granted: false),
    };

class _QuickTranslatorState extends StatelessWidget {
  const _QuickTranslatorState({
    required this.title,
    required this.body,
    this.source,
    this.loading = false,
    this.error = false,
    this.compare = false,
  });

  final String title;
  final String? source;
  final String body;
  final bool loading;
  final bool error;
  final bool compare;

  @override
  Widget build(BuildContext context) {
    final tokens = context.tokens;
    final colors = tokens.colors;
    return ColoredBox(
      color: colors.window,
      child: Center(
        child: SizedBox(
          width: 396,
          child: Surface(
            tone: SurfaceTone.raised,
            padding: SurfacePadding.lg,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Row(
                  children: [
                    Text(
                      title,
                      style: tokens.typography.displayStyle(
                        fontSize: 13,
                        fontWeight: FontWeight.w700,
                        color: colors.fg,
                      ),
                    ),
                    const Spacer(),
                    const Badge(child: Text('LinguaRay')),
                  ],
                ),
                if (source != null) ...[
                  const SizedBox(height: 16),
                  Text(
                    source!,
                    style: tokens.typography.sansStyle(
                      fontSize: 12,
                      height: 1.5,
                      color: colors.fgSubtle,
                    ),
                  ),
                ],
                const SizedBox(height: 16),
                if (loading)
                  const Align(
                    alignment: Alignment.centerLeft,
                    child: Spinner(size: SpinnerSize.sm),
                  )
                else
                  Text(
                    body,
                    style: tokens.typography.translationStyle(
                      color: error ? colors.danger : colors.fg,
                    ),
                  ),
                if (compare) ...[
                  const SizedBox(height: 14),
                  Text(
                    'System · 隐私优先翻译。',
                    style: tokens.typography.sansStyle(
                      fontSize: 11,
                      color: colors.fgSubtle,
                    ),
                  ),
                ],
                const SizedBox(height: 18),
                Row(
                  children: [
                    Button(
                      variant: ButtonVariant.primary,
                      onPressed: loading ? null : () {},
                      child: Text(error ? '打开设置' : '复制'),
                    ),
                    const SizedBox(width: 8),
                    Button(
                      variant: ButtonVariant.secondary,
                      onPressed: () {},
                      child: const Text('工作台'),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _WorkbenchState extends StatelessWidget {
  const _WorkbenchState({
    required this.source,
    required this.translation,
    this.platform,
  });

  final String source;
  final String translation;
  final TargetPlatform? platform;

  @override
  Widget build(BuildContext context) {
    final colors = context.tokens.colors;
    return SizedBox(
      width: 840,
      height: 560,
      child: Workbench(
        targetPlatform: platform,
        sidebar: [
          SidebarGroup(
            first: true,
            label: const Text('工作区'),
            children: [
              NavigationItem(
                label: '翻译',
                icon: FluentIcons.translate_20_regular,
                selected: true,
                onTap: () {},
              ),
              NavigationItem(
                label: '设置',
                icon: FluentIcons.settings_20_regular,
                onTap: () {},
              ),
            ],
          ),
        ],
        child: ColoredBox(
          color: colors.window,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const WorkbenchToolbar(title: '翻译'),
              Expanded(
                child: Row(
                  children: [
                    Expanded(child: _TextPane(label: '原文', text: source)),
                    Expanded(
                      child: _TextPane(label: '译文', text: translation),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _TextPane extends StatelessWidget {
  const _TextPane({required this.label, required this.text});

  final String label;
  final String text;

  @override
  Widget build(BuildContext context) {
    final tokens = context.tokens;
    return Padding(
      padding: const EdgeInsets.all(22),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(label, style: tokens.typography.labelStyle()),
          const SizedBox(height: 14),
          Text(text, style: tokens.typography.translationStyle()),
        ],
      ),
    );
  }
}

class _SettingsState extends StatelessWidget {
  const _SettingsState();

  @override
  Widget build(BuildContext context) {
    return SettingsPage(
      children: [
        PreferenceSection(
          label: const Text('服务商'),
          children: [
            const PreferenceRow(
              title: Text('System'),
              subtitle: Text('系统翻译与 OCR'),
              trailing: [Badge(child: Text('已启用'))],
            ),
            PreferenceRow(
              title: const Text('OpenAI Compatible'),
              subtitle: const Text('密钥保存在系统安全存储中'),
              trailing: [
                Button(
                  variant: ButtonVariant.secondary,
                  onPressed: () {},
                  child: const Text('配置'),
                ),
              ],
            ),
          ],
        ),
      ],
    );
  }
}

class _PermissionState extends StatelessWidget {
  const _PermissionState({required this.granted});

  final bool granted;

  @override
  Widget build(BuildContext context) {
    return SettingsPage(
      children: [
        PreferenceSection(
          label: const Text('系统权限'),
          children: [
            for (final title in ['辅助功能', '屏幕录制'])
              PreferenceRow(
                title: Text(title),
                subtitle: Text(granted ? '系统已授权' : '需要授权后才能使用此功能'),
                trailing: [
                  Button(
                    variant: ButtonVariant.secondary,
                    onPressed: () {},
                    child: Text(granted ? '重新检查' : '授权'),
                  ),
                ],
              ),
          ],
        ),
      ],
    );
  }
}
