import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:flutter/widgets.dart';

import 'ui.dart';

/// The 对比 N 个服务 / 收起对比 pill — accent-tinted with a rotating chevron.
///
/// Shared by the workbench translation page and the mini translator; the two
/// sites differ only in box metrics, which each call site passes so the
/// rendered result matches what the former private copies drew.
class CompareToggle extends StatelessWidget {
  const CompareToggle({
    super.key,
    required this.expanded,
    required this.label,
    required this.onPressed,
    this.height = 18,
    this.padding = const EdgeInsets.symmetric(horizontal: 9),
  });

  final bool expanded;

  /// 对比 N 个服务 when the services answered, 查看 N 个服务的原因 when none did.
  final String label;
  final VoidCallback onPressed;

  /// Box height; null sizes the pill to its content.
  final double? height;

  final EdgeInsetsGeometry padding;

  @override
  Widget build(BuildContext context) {
    final tokens = context.tokens;
    final colors = tokens.colors;
    final radius = BorderRadius.circular(tokens.radii.pill);

    return Pressable(
      onPressed: onPressed,
      borderRadius: radius,
      semanticsLabel: label,
      builder: (context, state) => AnimatedContainer(
        duration: kTransitionDuration,
        height: height,
        padding: padding,
        decoration: BoxDecoration(
          color: colors.accent.withValues(alpha: state.hovered ? 0.20 : 0.12),
          borderRadius: radius,
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              label,
              style: tokens.typography.sansStyle(
                fontSize: 11,
                fontWeight: FontWeight.w600,
                height: 1,
                color: colors.accentText,
              ),
            ),
            const SizedBox(width: 4),
            AnimatedRotation(
              turns: expanded ? 0.5 : 0,
              duration: kTransitionDuration,
              child: Icon(
                FluentIcons.chevron_down_20_regular,
                size: 10,
                color: colors.accentText,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
