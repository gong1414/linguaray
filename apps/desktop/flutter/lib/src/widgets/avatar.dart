import 'package:flutter/widgets.dart';

import 'ui.dart' show DesignThemeContext, DesignTypographyStyles;

enum AvatarSize { xs, sm, md, lg }

({double side, double typeSize}) _avatarMetrics(AvatarSize size) =>
    switch (size) {
      AvatarSize.xs => (side: 16, typeSize: 10),
      AvatarSize.sm => (side: 18, typeSize: 11),
      AvatarSize.md => (side: 24, typeSize: 12),
      AvatarSize.lg => (side: 26, typeSize: 13),
    };

/// Rounded monogram used wherever a service or source needs a compact marker.
class Avatar extends StatelessWidget {
  const Avatar({
    super.key,
    required this.label,
    required this.color,
    this.foregroundColor = const Color(0xFFFFFFFF),
    this.size = AvatarSize.sm,
  });

  final String label;
  final Color color;
  final Color foregroundColor;
  final AvatarSize size;

  @override
  Widget build(BuildContext context) {
    final tokens = context.tokens;
    final metrics = _avatarMetrics(size);

    return ExcludeSemantics(
      child: SizedBox.square(
        dimension: metrics.side,
        child: DecoratedBox(
          decoration: BoxDecoration(
            color: color,
            borderRadius: BorderRadius.circular(tokens.radii.avatar),
          ),
          child: Center(
            child: Text(
              label,
              maxLines: 1,
              style: tokens.typography.displayStyle(
                fontSize: metrics.typeSize,
                fontWeight: FontWeight.w700,
                height: 1,
                color: foregroundColor,
              ),
            ),
          ),
        ),
      ),
    );
  }
}
