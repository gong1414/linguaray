import 'package:flutter/material.dart';

import 'settings_labels.dart';

class SettingsShellView extends StatelessWidget {
  const SettingsShellView({
    required this.labels,
    required this.section,
    required this.child,
    required this.onSectionSelected,
    super.key,
  });

  final SettingsShellLabels labels;
  final SettingsSection section;
  final Widget child;
  final ValueChanged<SettingsSection> onSectionSelected;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final destinations = <(SettingsSection, IconData, String)>[
      (SettingsSection.general, Icons.tune_rounded, labels.general),
      (SettingsSection.services, Icons.translate_rounded, labels.services),
      (SettingsSection.providers, Icons.hub_outlined, labels.providers),
      (SettingsSection.shortcuts, Icons.keyboard_rounded, labels.shortcuts),
      (
        SettingsSection.permissions,
        Icons.verified_user_outlined,
        labels.permissions,
      ),
      if (labels.advanced.isNotEmpty)
        (SettingsSection.advanced, Icons.terminal_rounded, labels.advanced),
      if (labels.updates.isNotEmpty)
        (
          SettingsSection.updates,
          Icons.system_update_alt_rounded,
          labels.updates,
        ),
      (SettingsSection.about, Icons.info_outline_rounded, labels.about),
    ];

    return Material(
      color: theme.scaffoldBackgroundColor,
      child: LayoutBuilder(
        builder: (context, constraints) {
          final compact = constraints.maxWidth < 640;
          final nav = ListView(
            padding: const EdgeInsets.fromLTRB(8, 12, 8, 12),
            children: [
              Padding(
                padding: const EdgeInsets.fromLTRB(12, 4, 12, 12),
                child: Text(labels.title, style: theme.textTheme.titleLarge),
              ),
              for (final item in destinations)
                ListTile(
                  selected: item.$1 == section,
                  leading: Icon(item.$2),
                  title: Text(item.$3),
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(10),
                  ),
                  onTap: () => onSectionSelected(item.$1),
                ),
            ],
          );

          if (compact) {
            return Column(
              children: [
                SizedBox(height: 220, child: nav),
                Divider(color: theme.colorScheme.outlineVariant),
                Expanded(child: child),
              ],
            );
          }

          return Row(
            children: [
              SizedBox(width: 220, child: nav),
              VerticalDivider(
                width: 1,
                color: theme.colorScheme.outlineVariant,
              ),
              Expanded(child: child),
            ],
          );
        },
      ),
    );
  }
}
