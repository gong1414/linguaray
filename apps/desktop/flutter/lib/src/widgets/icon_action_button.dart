import 'package:flutter/material.dart' hide IconButton;

import 'ui.dart' show IconButton, kTransitionDuration;

/// Adapts an [IconData] to the design system toolbar button.
class IconActionButton extends StatelessWidget {
  const IconActionButton({
    super.key,
    required this.icon,
    this.tooltip,
    required this.onPressed,
    this.selected = false,
    this.iconTurns = 0,
    this.iconSize = 14,
  });

  final IconData icon;
  final String? tooltip;
  final VoidCallback? onPressed;
  final bool selected;

  final double iconSize;
  final double iconTurns;

  @override
  Widget build(BuildContext context) {
    final iconWidget = AnimatedRotation(
      turns: iconTurns,
      duration: kTransitionDuration,
      child: Icon(icon, size: iconSize),
    );
    final control = IconButton(
      label: tooltip ?? '',
      active: selected,
      iconSize: iconSize,
      onPressed: onPressed,
      icon: iconWidget,
    );

    final message = tooltip;
    return message == null ? control : Tooltip(message: message, child: control);
  }
}
