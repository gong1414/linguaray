import 'package:flutter/material.dart';

import 'settings_labels.dart';

/// Native-preferences inspired navigation shared by macOS and Windows.
/// It deliberately has no product header or main-window navigation.
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
    final sidebar = theme.colorScheme.surfaceContainerLow;

    return Material(
      color: theme.colorScheme.surface,
      child: Row(
        children: [
          Container(
            width: 188,
            color: sidebar,
            child: ListView(
              padding: const EdgeInsets.fromLTRB(10, 48, 10, 18),
              children: [
                _GroupLabel(labels.translationGroup),
                _Destination(
                  icon: Icons.adjust_rounded,
                  label: labels.translationSettings,
                  selected: section == SettingsSection.translation,
                  onTap: () => onSectionSelected(SettingsSection.translation),
                ),
                _Destination(
                  icon: Icons.inventory_2_outlined,
                  label: labels.translationServices,
                  selected: section == SettingsSection.translationServices,
                  onTap: () =>
                      onSectionSelected(SettingsSection.translationServices),
                ),
                _Destination(
                  icon: Icons.star_outline_rounded,
                  label: labels.favorites,
                  selected: section == SettingsSection.favorites,
                  onTap: () => onSectionSelected(SettingsSection.favorites),
                ),
                _Destination(
                  icon: Icons.history_rounded,
                  label: labels.history,
                  selected: section == SettingsSection.history,
                  onTap: () => onSectionSelected(SettingsSection.history),
                ),
                if (labels.glossary.isNotEmpty)
                  _Destination(
                    icon: Icons.menu_book_outlined,
                    label: labels.glossary,
                    selected: section == SettingsSection.glossary,
                    onTap: () => onSectionSelected(SettingsSection.glossary),
                  ),
                if (labels.vocabulary.isNotEmpty)
                  _Destination(
                    icon: Icons.bookmark_outline_rounded,
                    label: labels.vocabulary,
                    selected: section == SettingsSection.vocabulary,
                    onTap: () => onSectionSelected(SettingsSection.vocabulary),
                  ),
                const SizedBox(height: 12),
                _GroupLabel(labels.ocrGroup),
                _Destination(
                  icon: Icons.center_focus_strong_outlined,
                  label: labels.ocrSettings,
                  selected: section == SettingsSection.ocr,
                  onTap: () => onSectionSelected(SettingsSection.ocr),
                ),
                _Destination(
                  icon: Icons.inventory_2_outlined,
                  label: labels.ocrServices,
                  selected: section == SettingsSection.ocrServices,
                  onTap: () => onSectionSelected(SettingsSection.ocrServices),
                ),
                const SizedBox(height: 12),
                _GroupLabel(labels.generalGroup),
                _Destination(
                  icon: Icons.tune_rounded,
                  label: labels.general,
                  selected: section == SettingsSection.general,
                  onTap: () => onSectionSelected(SettingsSection.general),
                ),
                _Destination(
                  icon: Icons.verified_user_outlined,
                  label: labels.permissions,
                  selected: section == SettingsSection.permissions,
                  onTap: () => onSectionSelected(SettingsSection.permissions),
                ),
                if (labels.integration.isNotEmpty)
                  _Destination(
                    icon: Icons.integration_instructions_outlined,
                    label: labels.integration,
                    selected: section == SettingsSection.integration,
                    onTap: () => onSectionSelected(SettingsSection.integration),
                  ),
                if (labels.updates.isNotEmpty)
                  _Destination(
                    icon: Icons.system_update_alt_rounded,
                    label: labels.updates,
                    selected: section == SettingsSection.updates,
                    onTap: () => onSectionSelected(SettingsSection.updates),
                  ),
                _Destination(
                  icon: Icons.info_outline_rounded,
                  label: labels.about,
                  selected: section == SettingsSection.about,
                  onTap: () => onSectionSelected(SettingsSection.about),
                ),
              ],
            ),
          ),
          VerticalDivider(
            width: 1,
            thickness: 1,
            color: theme.dividerColor.withValues(alpha: 0.55),
          ),
          Expanded(
            child: Material(color: theme.colorScheme.surface, child: child),
          ),
        ],
      ),
    );
  }
}

class _GroupLabel extends StatelessWidget {
  const _GroupLabel(this.label);

  final String label;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.fromLTRB(10, 0, 8, 5),
    child: Text(
      label,
      style: Theme.of(context).textTheme.labelSmall?.copyWith(
        color: Theme.of(context).colorScheme.onSurfaceVariant,
        fontWeight: FontWeight.w600,
      ),
    ),
  );
}

class _Destination extends StatelessWidget {
  const _Destination({
    required this.icon,
    required this.label,
    required this.selected,
    required this.onTap,
  });

  final IconData icon;
  final String label;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    return Padding(
      padding: const EdgeInsets.only(bottom: 2),
      child: Material(
        color: selected ? colors.secondaryContainer : Colors.transparent,
        borderRadius: BorderRadius.circular(7),
        child: InkWell(
          borderRadius: BorderRadius.circular(7),
          onTap: onTap,
          child: SizedBox(
            height: 34,
            child: Row(
              children: [
                const SizedBox(width: 9),
                Icon(
                  icon,
                  size: 18,
                  color: selected
                      ? colors.onSecondaryContainer
                      : colors.onSurfaceVariant,
                ),
                const SizedBox(width: 9),
                Expanded(
                  child: Text(
                    label,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                      color: selected ? colors.onSecondaryContainer : null,
                      fontWeight: selected ? FontWeight.w600 : FontWeight.w400,
                    ),
                  ),
                ),
                const SizedBox(width: 6),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
