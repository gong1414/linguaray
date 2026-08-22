import 'package:flutter/material.dart';
import 'package:linguaray_ui/linguaray_ui.dart'
    show BrandLogo, LinguaRayThemeContext;

enum WorkbenchDestinationId { translate, settings }

enum WindowChromeKind { macos, windows }

final class WorkbenchShellLabels {
  const WorkbenchShellLabels({
    required this.appName,
    required this.translate,
    required this.settings,
    required this.minimize,
    required this.maximize,
    required this.close,
  });

  final String appName;
  final String translate;
  final String settings;
  final String minimize;
  final String maximize;
  final String close;
}

class WorkbenchShellView extends StatelessWidget {
  const WorkbenchShellView({
    required this.labels,
    required this.chrome,
    required this.destination,
    required this.child,
    required this.onDestinationSelected,
    super.key,
    this.onMinimize,
    this.onToggleMaximize,
    this.onClose,
    this.onDragStart,
  });

  final WorkbenchShellLabels labels;
  final WindowChromeKind chrome;
  final WorkbenchDestinationId destination;
  final Widget child;
  final ValueChanged<WorkbenchDestinationId> onDestinationSelected;
  final VoidCallback? onMinimize;
  final VoidCallback? onToggleMaximize;
  final VoidCallback? onClose;
  final VoidCallback? onDragStart;

  @override
  Widget build(BuildContext context) {
    final metrics = context.metrics;
    final selectedIndex = destination == WorkbenchDestinationId.translate
        ? 0
        : 1;

    final rail = NavigationRail(
      selectedIndex: selectedIndex,
      onDestinationSelected: (index) => onDestinationSelected(
        index == 0
            ? WorkbenchDestinationId.translate
            : WorkbenchDestinationId.settings,
      ),
      leading: Padding(
        padding: EdgeInsets.only(
          top: chrome == WindowChromeKind.macos ? 28 : 8,
          bottom: 16,
        ),
        child: const BrandLogo(size: 28),
      ),
      destinations: [
        NavigationRailDestination(
          icon: const Icon(Icons.translate_outlined),
          selectedIcon: const Icon(Icons.translate_rounded),
          label: Text(labels.translate),
        ),
        NavigationRailDestination(
          icon: const Icon(Icons.settings_outlined),
          selectedIcon: const Icon(Icons.settings_rounded),
          label: Text(labels.settings),
        ),
      ],
    );

    final body = Row(
      children: [
        ColoredBox(color: context.brandColors.railSurface, child: rail),
        VerticalDivider(
          width: 1,
          color: Theme.of(context).colorScheme.outlineVariant,
        ),
        Expanded(child: child),
      ],
    );

    if (chrome == WindowChromeKind.macos) {
      return Material(
        color: Theme.of(context).scaffoldBackgroundColor,
        child: Padding(padding: EdgeInsets.zero, child: body),
      );
    }

    return Material(
      color: Theme.of(context).scaffoldBackgroundColor,
      child: Column(
        children: [
          _WindowsCaption(
            height: metrics.captionHeight,
            labels: labels,
            onMinimize: onMinimize,
            onToggleMaximize: onToggleMaximize,
            onClose: onClose,
            onDragStart: onDragStart,
          ),
          Divider(
            height: 1,
            color: Theme.of(context).colorScheme.outlineVariant,
          ),
          Expanded(child: body),
        ],
      ),
    );
  }
}

class _WindowsCaption extends StatelessWidget {
  const _WindowsCaption({
    required this.height,
    required this.labels,
    this.onMinimize,
    this.onToggleMaximize,
    this.onClose,
    this.onDragStart,
  });

  final double height;
  final WorkbenchShellLabels labels;
  final VoidCallback? onMinimize;
  final VoidCallback? onToggleMaximize;
  final VoidCallback? onClose;
  final VoidCallback? onDragStart;

  @override
  Widget build(BuildContext context) {
    final caption = SizedBox(
      height: height,
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 12),
        child: Row(
          children: [
            const BrandLogo(size: 16),
            const SizedBox(width: 8),
            Text(
              labels.appName,
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const Spacer(),
            IconButton(
              tooltip: labels.minimize,
              onPressed: onMinimize,
              icon: const Icon(Icons.remove, size: 18),
            ),
            IconButton(
              tooltip: labels.maximize,
              onPressed: onToggleMaximize,
              icon: const Icon(Icons.crop_square_outlined, size: 16),
            ),
            IconButton(
              tooltip: labels.close,
              onPressed: onClose,
              icon: const Icon(Icons.close, size: 18),
            ),
          ],
        ),
      ),
    );

    if (onDragStart == null) return caption;
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onPanStart: (_) => onDragStart!(),
      child: caption,
    );
  }
}
