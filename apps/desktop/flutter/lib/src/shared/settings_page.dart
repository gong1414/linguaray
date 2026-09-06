import 'package:flutter/material.dart';

/// One page frame for every settings destination, including searchable lists.
/// The shell owns navigation; this frame owns the page background, title,
/// actions, and content insets. A body can keep its own lazy scroll position.
class SettingsPage extends StatelessWidget {
  const SettingsPage({
    required this.title,
    this.actions = const [],
    this.toolbar,
    this.children = const [],
    this.body,
    super.key,
  }) : assert(body == null || children.length == 0);

  final String title;
  final List<Widget> actions;
  final Widget? toolbar;
  final List<Widget> children;
  final Widget? body;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Material(
      color: theme.colorScheme.surfaceContainerLowest,
      child: Padding(
        padding: const EdgeInsets.fromLTRB(32, 32, 32, 32),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            LayoutBuilder(
              builder: (context, constraints) {
                final stacked = constraints.maxWidth < 760;
                final buttons = Wrap(
                  spacing: 8,
                  runSpacing: 8,
                  crossAxisAlignment: WrapCrossAlignment.center,
                  children: actions,
                );
                return Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    ConstrainedBox(
                      constraints: const BoxConstraints(minHeight: 48),
                      child: Row(
                        children: [
                          Expanded(
                            child: Semantics(
                              header: true,
                              child: Text(
                                title,
                                style: theme.textTheme.titleLarge,
                              ),
                            ),
                          ),
                          if (!stacked && actions.isNotEmpty) ...[
                            const SizedBox(width: 16),
                            buttons,
                          ],
                        ],
                      ),
                    ),
                    if (stacked && actions.isNotEmpty) ...[
                      const SizedBox(height: 8),
                      Align(alignment: Alignment.centerRight, child: buttons),
                    ],
                  ],
                );
              },
            ),
            const SizedBox(height: 24),
            if (toolbar != null) ...[toolbar!, const SizedBox(height: 20)],
            Expanded(
              child:
                  body ??
                  ListView(padding: EdgeInsets.zero, children: children),
            ),
          ],
        ),
      ),
    );
  }
}
