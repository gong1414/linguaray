import 'package:flutter/widgets.dart';
import 'package:linguaray_ui/src/theme/theme.dart';
import 'package:linguaray_ui/src/widgets/pressable.dart';

/// The flat 24px chrome button the mini-window toolbars use — pin, capture,
/// clipboard, settings. AppKit style: no box by default, an accent read when
/// [active], and a soft hover wash so the row still feels tappable.
///
/// The glyph is a plain [Icon] (usually a `FluentIcons.*_20_regular`); its
/// colour is fed through [IconTheme], so pass it without an explicit colour.
class IconButton extends StatelessWidget {
  const IconButton({
    super.key,
    required this.label,
    required this.icon,
    this.active = false,
    this.iconSize = 14,
    this.enabled = true,
    this.onPressed,
  });

  /// Accessible name, announced by screen readers.
  final String label;

  /// Persistent on-state — the pin button, for instance.
  final bool active;

  /// Kept separate from a null [onPressed] so a caller can grey the affordance
  /// out without having to drop the handler it already has.
  final bool enabled;
  final Widget icon;

  /// Glyph size, fed through [IconTheme]. It is set per call site — 18 in the
  /// mini-window toolbar, 16 in the sidebar header, 13 on the document zoom
  /// stepper.
  final double iconSize;

  final VoidCallback? onPressed;

  @override
  Widget build(BuildContext context) {
    final tokens = context.tokens;
    final colors = tokens.colors;
    final radius = BorderRadius.circular(tokens.radii.controlSm);
    final disabled = !enabled || onPressed == null;

    return Pressable(
      onPressed: enabled ? onPressed : null,
      enabled: !disabled,
      borderRadius: radius,
      semanticsLabel: label,
      builder: (context, state) {
        final foreground = active
            ? colors.accentText
            : (state.hovered ? colors.fg : colors.fgMuted);

        return Opacity(
          // The button carries no box by default, so a disabled one would
          // otherwise read as no change at all — dim the glyph and withhold
          // the hover wash instead.
          opacity: disabled ? 0.35 : 1,
          child: AnimatedContainer(
            duration: kTransitionDuration,
            width: 24,
            height: 24,
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: !active && state.hovered ? colors.subtle : null,
              borderRadius: radius,
            ),
            // `transition-colors` covers the glyph as well as the wash behind
            // it, and an [IconTheme] does not animate on its own.
            child: TweenAnimationBuilder<Color?>(
              duration: kTransitionDuration,
              tween: ColorTween(end: foreground),
              builder: (context, color, child) => IconTheme(
                data: IconThemeData(color: color, size: iconSize),
                child: child!,
              ),
              child: icon,
            ),
          ),
        );
      },
    );
  }
}
