import 'package:flutter/material.dart';
import 'package:linguaray_ui/linguaray_ui.dart'
    show BrandLogo, LinguaRayThemeContext;

enum WorkbenchDestinationId {
  translate,
  history,
  glossary,
  vocabulary,
  settings,
}

enum WindowChromeKind { macos, windows }

final class WorkbenchShellLabels {
  const WorkbenchShellLabels({
    required this.appName,
    required this.translate,
    required this.history,
    required this.glossary,
    required this.vocabulary,
    required this.settings,
    required this.minimize,
    required this.maximize,
    required this.close,
  });

  final String appName;
  final String translate;
  final String history;
  final String glossary;
  final String vocabulary;
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
    final theme = Theme.of(context);
    final metrics = context.metrics;
    final commandBar = _CommandBar(
      labels: labels,
      chrome: chrome,
      destination: destination,
      height: chrome == WindowChromeKind.windows
          ? metrics.captionHeight + 8
          : metrics.commandBarHeight,
      onDestinationSelected: onDestinationSelected,
      onMinimize: onMinimize,
      onToggleMaximize: onToggleMaximize,
      onClose: onClose,
      onDragStart: onDragStart,
    );

    return Material(
      color: theme.scaffoldBackgroundColor,
      child: Column(
        children: [
          commandBar,
          Divider(height: 1, color: theme.colorScheme.outlineVariant),
          Expanded(child: child),
        ],
      ),
    );
  }
}

class _CommandBar extends StatelessWidget {
  const _CommandBar({
    required this.labels,
    required this.chrome,
    required this.destination,
    required this.height,
    required this.onDestinationSelected,
    this.onMinimize,
    this.onToggleMaximize,
    this.onClose,
    this.onDragStart,
  });

  final WorkbenchShellLabels labels;
  final WindowChromeKind chrome;
  final WorkbenchDestinationId destination;
  final double height;
  final ValueChanged<WorkbenchDestinationId> onDestinationSelected;
  final VoidCallback? onMinimize;
  final VoidCallback? onToggleMaximize;
  final VoidCallback? onClose;
  final VoidCallback? onDragStart;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final metrics = context.metrics;
    final destinations = <(WorkbenchDestinationId, String)>[
      (WorkbenchDestinationId.translate, labels.translate),
      (WorkbenchDestinationId.history, labels.history),
      (WorkbenchDestinationId.glossary, labels.glossary),
      (WorkbenchDestinationId.vocabulary, labels.vocabulary),
    ];

    final bar = SizedBox(
      height: height,
      child: Padding(
        padding: EdgeInsets.only(
          left: chrome == WindowChromeKind.macos ? metrics.macTrafficInset : 12,
          right: 8,
        ),
        child: Row(
          children: [
            const BrandLogo(size: 18),
            const SizedBox(width: 8),
            Text(labels.appName, style: theme.textTheme.titleMedium),
            const SizedBox(width: 20),
            Expanded(
              child: SingleChildScrollView(
                scrollDirection: Axis.horizontal,
                child: Row(
                  children: [
                    for (final item in destinations)
                      _NavTab(
                        label: item.$2,
                        selected: destination == item.$1,
                        onPressed: () => onDestinationSelected(item.$1),
                      ),
                  ],
                ),
              ),
            ),
            IconButton(
              tooltip: labels.settings,
              isSelected: destination == WorkbenchDestinationId.settings,
              onPressed: () =>
                  onDestinationSelected(WorkbenchDestinationId.settings),
              icon: Icon(
                destination == WorkbenchDestinationId.settings
                    ? Icons.settings_rounded
                    : Icons.settings_outlined,
                size: 18,
              ),
            ),
            if (chrome == WindowChromeKind.windows) ...[
              IconButton(
                tooltip: labels.minimize,
                onPressed: onMinimize,
                icon: const Icon(Icons.remove, size: 16),
              ),
              IconButton(
                tooltip: labels.maximize,
                onPressed: onToggleMaximize,
                icon: const Icon(Icons.crop_square_outlined, size: 14),
              ),
              IconButton(
                tooltip: labels.close,
                onPressed: onClose,
                icon: const Icon(Icons.close, size: 16),
              ),
            ],
          ],
        ),
      ),
    );

    if (chrome != WindowChromeKind.windows || onDragStart == null) {
      return ColoredBox(color: theme.colorScheme.surface, child: bar);
    }
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onPanStart: (_) => onDragStart!(),
      child: ColoredBox(color: theme.colorScheme.surface, child: bar),
    );
  }
}

class _NavTab extends StatelessWidget {
  const _NavTab({
    required this.label,
    required this.selected,
    required this.onPressed,
  });

  final String label;
  final bool selected;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final color = selected
        ? theme.colorScheme.primary
        : theme.colorScheme.onSurfaceVariant;
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 2),
      child: TextButton(
        onPressed: onPressed,
        style: TextButton.styleFrom(
          foregroundColor: color,
          minimumSize: const Size(0, 32),
          padding: const EdgeInsets.symmetric(horizontal: 10),
        ),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(
              label,
              style: theme.textTheme.labelLarge?.copyWith(
                color: color,
                fontWeight: selected ? FontWeight.w600 : FontWeight.w500,
              ),
            ),
            const SizedBox(height: 4),
            AnimatedContainer(
              duration: const Duration(milliseconds: 120),
              height: 2,
              width: selected ? 22 : 0,
              color: selected ? theme.colorScheme.primary : Colors.transparent,
            ),
          ],
        ),
      ),
    );
  }
}
