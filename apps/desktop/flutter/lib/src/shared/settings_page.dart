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
        padding: const EdgeInsets.fromLTRB(28, 28, 28, 24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            LayoutBuilder(
              builder: (context, constraints) {
                final stacked = constraints.maxWidth < 620;
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
            const SizedBox(height: 16),
            const Divider(),
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

/// Open sections group related settings without nesting cards inside pages.
class SettingsSectionBlock extends StatelessWidget {
  const SettingsSectionBlock({
    required this.title,
    required this.children,
    this.description,
    super.key,
  });

  final String title;
  final String? description;
  final List<Widget> children;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(bottom: 28),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text(title, style: Theme.of(context).textTheme.titleMedium),
        if (description != null) ...[
          const SizedBox(height: 6),
          Text(description!, style: Theme.of(context).textTheme.bodySmall),
        ],
        const SizedBox(height: 16),
        ...children,
      ],
    ),
  );
}

/// Settings actions wrap below their description when a desktop pane narrows.
class SettingsActionRow extends StatelessWidget {
  const SettingsActionRow({
    required this.title,
    required this.description,
    required this.action,
    this.leading,
    super.key,
  });

  final String title;
  final String description;
  final Widget action;
  final Widget? leading;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 20),
    child: LayoutBuilder(
      builder: (context, constraints) {
        final text = Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(title, style: Theme.of(context).textTheme.titleMedium),
            if (description.isNotEmpty) ...[
              const SizedBox(height: 6),
              Text(description, style: Theme.of(context).textTheme.bodySmall),
            ],
          ],
        );
        return Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (leading != null) ...[leading!, const SizedBox(width: 16)],
            Expanded(
              child: constraints.maxWidth < 540
                  ? Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [text, const SizedBox(height: 12), action],
                    )
                  : Row(
                      children: [
                        Expanded(child: text),
                        const SizedBox(width: 24),
                        action,
                      ],
                    ),
            ),
          ],
        );
      },
    ),
  );
}
