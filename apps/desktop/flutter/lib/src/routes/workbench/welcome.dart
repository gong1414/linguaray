import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:flutter/widgets.dart';
import 'package:go_router/go_router.dart';

import '../../platform/onboarding_controller.dart';
import '../../platform/permission_controller.dart';
import '../../platform/platform_types.dart';
import '../../widgets/ui.dart'
    show
        BrandLogo,
        Button,
        ButtonVariant,
        DesignThemeContext,
        DesignTypographyStyles;
import '../../widgets/workbench.dart' show WorkbenchToolbar;

class WorkbenchWelcomePage extends StatelessWidget {
  const WorkbenchWelcomePage({super.key});

  bool _isChinese(BuildContext context) =>
      Localizations.localeOf(context).languageCode == 'zh';

  @override
  Widget build(BuildContext context) {
    final zh = _isChinese(context);
    final tokens = context.tokens;
    final colors = tokens.colors;

    return ColoredBox(
      color: colors.window,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const WorkbenchToolbar(title: 'LinguaRay'),
          Expanded(
            child: LayoutBuilder(
              builder: (context, constraints) {
                final horizontalPadding =
                    constraints.maxWidth < 400 ? 12.0 : 42.0;
                return SingleChildScrollView(
                  padding: EdgeInsets.fromLTRB(
                    horizontalPadding,
                    30,
                    horizontalPadding,
                    36,
                  ),
                  child: Center(
                    child: ConstrainedBox(
                      constraints: const BoxConstraints(maxWidth: 620),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: [
                          const Align(
                            alignment: Alignment.centerLeft,
                            child: BrandLogo(size: 42),
                          ),
                          const SizedBox(height: 18),
                          Text(
                            zh ? '欢迎使用 LinguaRay' : 'Welcome to LinguaRay',
                            style: tokens.typography.displayStyle(
                              fontSize: 28,
                              fontWeight: FontWeight.w700,
                              height: 1.15,
                              color: colors.fg,
                            ),
                          ),
                          const SizedBox(height: 10),
                          Text(
                            zh
                                ? '隐私优先的桌面翻译工具。核心翻译与系统能力由 Rust 提供，界面使用 Flutter。'
                                : 'A privacy-first desktop translator with a Rust core and a Flutter interface.',
                            style: tokens.typography.sansStyle(
                              fontSize: 13,
                              height: 1.5,
                              color: colors.fgSubtle,
                            ),
                          ),
                          const SizedBox(height: 26),
                          Wrap(
                            spacing: 10,
                            runSpacing: 10,
                            children: [
                              _CapabilityCard(
                                icon: FluentIcons.keyboard_20_regular,
                                title: zh ? '全局快捷键' : 'Global shortcuts',
                                body: zh
                                    ? '从任何应用打开快捷翻译窗。'
                                    : 'Open quick translate from any app.',
                              ),
                              _CapabilityCard(
                                icon: FluentIcons
                                    .text_underline_double_20_regular,
                                title: zh ? '划词翻译' : 'Selection translate',
                                body: zh
                                    ? '读取选中文本并恢复剪贴板。'
                                    : 'Read selected text and restore the clipboard.',
                              ),
                              _CapabilityCard(
                                icon: FluentIcons.scan_camera_20_regular,
                                title: zh ? '截图 OCR' : 'Capture OCR',
                                body: zh
                                    ? '框选屏幕区域并识别文字。'
                                    : 'Capture a region and recognize its text.',
                              ),
                            ],
                          ),
                          const SizedBox(height: 24),
                          ListenableBuilder(
                            listenable: permissionController,
                            builder: (context, _) => _PermissionSummary(
                              chinese: zh,
                              snapshot: permissionController.snapshot,
                            ),
                          ),
                          const SizedBox(height: 24),
                          Wrap(
                            spacing: 10,
                            runSpacing: 10,
                            children: [
                              Button(
                                variant: ButtonVariant.primary,
                                onPressed: () {
                                  onboardingController.complete();
                                  context.go('/translate');
                                },
                                child: Text(zh ? '开始使用' : 'Get started'),
                              ),
                              Button(
                                variant: ButtonVariant.secondary,
                                onPressed: () => permissionController.refresh(),
                                child: Text(zh ? '重新检查权限' : 'Recheck access'),
                              ),
                            ],
                          ),
                        ],
                      ),
                    ),
                  ),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}

class _CapabilityCard extends StatelessWidget {
  const _CapabilityCard({
    required this.icon,
    required this.title,
    required this.body,
  });

  final IconData icon;
  final String title;
  final String body;

  @override
  Widget build(BuildContext context) {
    final tokens = context.tokens;
    final colors = tokens.colors;
    return Container(
      width: 182,
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: colors.card,
        border: Border.all(color: colors.hairlineStrong),
        borderRadius: BorderRadius.circular(tokens.radii.box),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(icon, size: 18, color: colors.accentText),
          const SizedBox(height: 12),
          Text(
            title,
            style: tokens.typography.sansStyle(
              fontSize: 12,
              fontWeight: FontWeight.w700,
              color: colors.fg,
            ),
          ),
          const SizedBox(height: 5),
          Text(
            body,
            style: tokens.typography.sansStyle(
              fontSize: 11,
              height: 1.4,
              color: colors.fgSubtle,
            ),
          ),
        ],
      ),
    );
  }
}

class _PermissionSummary extends StatelessWidget {
  const _PermissionSummary({
    required this.chinese,
    required this.snapshot,
  });

  final bool chinese;
  final PermissionSnapshot snapshot;

  String _label(PermissionState state) => switch (state) {
        PermissionState.granted => chinese ? '已授权' : 'Granted',
        PermissionState.denied => chinese ? '未授权' : 'Not granted',
        PermissionState.notRequired => chinese ? '无需授权' : 'Not required',
        PermissionState.unknown => chinese ? '检查中' : 'Checking',
      };

  @override
  Widget build(BuildContext context) {
    final tokens = context.tokens;
    final colors = tokens.colors;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
      decoration: BoxDecoration(
        color: colors.card,
        border: Border.all(color: colors.hairlineStrong),
        borderRadius: BorderRadius.circular(tokens.radii.box),
      ),
      child: Row(
        children: [
          Expanded(
            child: Text(
              '${chinese ? '辅助功能' : 'Accessibility'} · ${_label(snapshot.accessibility)}',
              style: tokens.typography.sansStyle(
                fontSize: 11,
                color: colors.fgSubtle,
              ),
            ),
          ),
          Expanded(
            child: Text(
              '${chinese ? '屏幕录制' : 'Screen recording'} · ${_label(snapshot.screenRecording)}',
              style: tokens.typography.sansStyle(
                fontSize: 11,
                color: colors.fgSubtle,
              ),
            ),
          ),
        ],
      ),
    );
  }
}
