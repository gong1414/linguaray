import 'package:flutter/widgets.dart';

import 'ui.dart' show NavItem;

/// Product navigation row with selection semantics and a compact leading icon.
class NavigationItem extends StatelessWidget {
  const NavigationItem({
    super.key,
    required this.label,
    required this.icon,
    this.selected = false,
    this.onTap,
  });

  final String label;
  final IconData icon;
  final bool selected;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      selected: selected,
      button: true,
      child: NavItem(
        active: selected,
        onPressed: onTap,
        icon: Icon(icon, size: 15),
        child: Text(label, maxLines: 1, overflow: TextOverflow.ellipsis),
      ),
    );
  }
}
