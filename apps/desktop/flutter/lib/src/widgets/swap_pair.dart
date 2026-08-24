import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:flutter/widgets.dart';

import 'ui.dart'
    show
        DesignThemeContext,
        DesignTypographyStyles,
        Pressable,
        kTransitionDuration;

enum SwapPairSize { sm, md }

/// Source/target language picker shared by translation surfaces.
class SwapPair extends StatelessWidget {
  const SwapPair({
    super.key,
    required this.start,
    required this.end,
    this.onSwap,
    this.onStartPressed,
    this.startOpen = false,
    this.onEndPressed,
    this.endOpen = false,
    this.startKey,
    this.endKey,
    this.size = SwapPairSize.md,
    this.swapSemanticsLabel = '交换',
  });

  final String start;
  final String end;
  final VoidCallback? onSwap;
  final VoidCallback? onStartPressed;
  final bool startOpen;
  final VoidCallback? onEndPressed;
  final bool endOpen;
  final Key? startKey;
  final Key? endKey;
  final SwapPairSize size;
  final String swapSemanticsLabel;

  @override
  Widget build(BuildContext context) {
    final tokens = context.tokens;
    final radius = BorderRadius.circular(tokens.radii.control);
    final swapSide = size == SwapPairSize.md ? 20.0 : 18.0;

    return DecoratedBox(
      decoration: BoxDecoration(
        color: context.colors.control,
        borderRadius: radius,
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 5),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            _LanguageEndpoint(
              key: startKey,
              value: start,
              expanded: startOpen,
              onPressed: onStartPressed,
            ),
            const SizedBox(width: 8),
            _SwapControl(
              side: swapSide,
              semanticsLabel: swapSemanticsLabel,
              onPressed: onSwap,
            ),
            const SizedBox(width: 8),
            _LanguageEndpoint(
              key: endKey,
              value: end,
              cjk: true,
              expanded: endOpen,
              onPressed: onEndPressed,
            ),
          ],
        ),
      ),
    );
  }
}

class _SwapControl extends StatelessWidget {
  const _SwapControl({
    required this.side,
    required this.semanticsLabel,
    this.onPressed,
  });

  final double side;
  final String semanticsLabel;
  final VoidCallback? onPressed;

  @override
  Widget build(BuildContext context) {
    final tokens = context.tokens;
    final radius = BorderRadius.circular(tokens.radii.chip);
    return Pressable(
      semanticsLabel: semanticsLabel,
      onPressed: onPressed,
      borderRadius: radius,
      builder: (context, state) {
        final enabled = onPressed != null;
        final foreground = !enabled
            ? context.colors.fgFaint
            : state.hovered
            ? context.colors.fg
            : context.colors.fgTertiary;
        return AnimatedContainer(
          duration: kTransitionDuration,
          width: side,
          height: side,
          alignment: Alignment.center,
          decoration: BoxDecoration(
            color: context.colors.window,
            borderRadius: radius,
          ),
          child: Icon(
            FluentIcons.arrow_swap_20_regular,
            size: 12,
            color: foreground,
          ),
        );
      },
    );
  }
}

class _LanguageEndpoint extends StatelessWidget {
  const _LanguageEndpoint({
    super.key,
    required this.value,
    this.cjk = false,
    this.expanded = false,
    this.onPressed,
  });

  final String value;
  final bool cjk;
  final bool expanded;
  final VoidCallback? onPressed;

  @override
  Widget build(BuildContext context) {
    final tokens = context.tokens;
    final style = cjk
        ? tokens.typography.cjkStyle(
            fontSize: 12,
            fontWeight: FontWeight.w600,
            height: 1,
            color: context.colors.fg,
          )
        : tokens.typography.sansStyle(
            fontSize: 12,
            fontWeight: FontWeight.w600,
            height: 1,
            color: context.colors.fg,
          );
    final label = Text(
      value,
      maxLines: 1,
      overflow: TextOverflow.ellipsis,
      style: style,
    );

    if (onPressed == null) {
      return Padding(
        padding: EdgeInsetsDirectional.only(start: 5, end: cjk ? 5 : 0),
        child: label,
      );
    }

    final radius = BorderRadius.circular(tokens.radii.chip);
    return Pressable(
      semanticsLabel: value,
      onPressed: onPressed,
      borderRadius: radius,
      builder: (context, state) => AnimatedContainer(
        duration: kTransitionDuration,
        padding: const EdgeInsetsDirectional.fromSTEB(6, 4, 4, 4),
        decoration: BoxDecoration(
          color: expanded || state.hovered ? context.colors.window : null,
          borderRadius: radius,
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            label,
            const SizedBox(width: 4),
            Icon(
              FluentIcons.chevron_down_20_regular,
              size: 9,
              color: context.colors.fgTertiary,
            ),
          ],
        ),
      ),
    );
  }
}
