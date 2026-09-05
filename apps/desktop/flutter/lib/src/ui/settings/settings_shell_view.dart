import 'package:flutter/material.dart';
import 'package:linguaray_ui/linguaray_ui.dart' show BrandLogo;

import 'settings_labels.dart';

/// Four work areas keep the rail short. Their pages stay in a horizontal bar.
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
    final colors = theme.colorScheme;
    final groups = <_WorkArea>[
      _WorkArea(labels.translationGroup, Icons.translate_rounded, [
        (SettingsSection.translation, labels.translationSettings),
        (SettingsSection.translationServices, labels.translationServices),
      ]),
      _WorkArea(
        labels.libraryGroup.isEmpty ? labels.history : labels.libraryGroup,
        Icons.auto_stories_outlined,
        [
          (SettingsSection.history, labels.history),
          (SettingsSection.favorites, labels.favorites),
          if (labels.glossary.isNotEmpty)
            (SettingsSection.glossary, labels.glossary),
          if (labels.vocabulary.isNotEmpty)
            (SettingsSection.vocabulary, labels.vocabulary),
        ],
      ),
      _WorkArea(labels.ocrGroup, Icons.document_scanner_outlined, [
        (SettingsSection.ocr, labels.ocrSettings),
        (SettingsSection.ocrServices, labels.ocrServices),
      ]),
      _WorkArea(labels.generalGroup, Icons.tune_rounded, [
        (SettingsSection.general, labels.general),
        (SettingsSection.permissions, labels.permissions),
        if (labels.dataTransfer.isNotEmpty)
          (SettingsSection.dataTransfer, labels.dataTransfer),
        if (labels.integration.isNotEmpty)
          (SettingsSection.integration, labels.integration),
        if (labels.updates.isNotEmpty)
          (SettingsSection.updates, labels.updates),
        (SettingsSection.about, labels.about),
      ]),
    ];
    final active = groups.firstWhere(
      (group) => group.pages.any((page) => page.$1 == section),
      orElse: () => groups.last,
    );
    return Material(
      color: colors.surfaceContainerLowest,
      child: Row(
        children: [
          Container(
            width: 80,
            decoration: BoxDecoration(
              color: colors.surface,
              border: Border(right: BorderSide(color: colors.outlineVariant)),
            ),
            child: Column(
              children: [
                const SizedBox(height: 26),
                const Tooltip(message: 'LinguaRay', child: BrandLogo(size: 32)),
                const SizedBox(height: 28),
                Expanded(
                  child: ListView(
                    padding: const EdgeInsets.symmetric(horizontal: 8),
                    children: [
                      for (final group in groups)
                        Padding(
                          padding: const EdgeInsets.only(bottom: 12),
                          child: Semantics(
                            selected: group == active,
                            child: Material(
                              color: group == active
                                  ? colors.primaryContainer
                                  : Colors.transparent,
                              borderRadius: BorderRadius.circular(8),
                              child: InkWell(
                                key: ValueKey(
                                  'work-area-${group.pages.first.$1.name}',
                                ),
                                borderRadius: BorderRadius.circular(8),
                                onTap: () =>
                                    onSectionSelected(group.pages.first.$1),
                                child: Padding(
                                  padding: const EdgeInsets.symmetric(
                                    vertical: 13,
                                  ),
                                  child: Column(
                                    children: [
                                      Icon(
                                        group.icon,
                                        size: 23,
                                        color: group == active
                                            ? colors.primary
                                            : colors.onSurfaceVariant,
                                      ),
                                      const SizedBox(height: 6),
                                      Text(
                                        group.label,
                                        maxLines: 1,
                                        overflow: TextOverflow.ellipsis,
                                        style: theme.textTheme.labelSmall
                                            ?.copyWith(
                                              color: group == active
                                                  ? colors.primary
                                                  : colors.onSurfaceVariant,
                                              fontWeight: group == active
                                                  ? FontWeight.w600
                                                  : FontWeight.w400,
                                            ),
                                      ),
                                    ],
                                  ),
                                ),
                              ),
                            ),
                          ),
                        ),
                    ],
                  ),
                ),
              ],
            ),
          ),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Container(
                  height: 66,
                  decoration: BoxDecoration(
                    border: Border(
                      bottom: BorderSide(color: colors.outlineVariant),
                    ),
                  ),
                  child: SingleChildScrollView(
                    scrollDirection: Axis.horizontal,
                    padding: const EdgeInsets.symmetric(horizontal: 24),
                    child: Row(
                      children: [
                        for (final (destination, label) in active.pages)
                          Padding(
                            padding: const EdgeInsets.only(right: 6),
                            child: Semantics(
                              selected: destination == section,
                              child: TextButton(
                                key: ValueKey(
                                  'settings-page-${destination.name}',
                                ),
                                onPressed: () => onSectionSelected(destination),
                                style: TextButton.styleFrom(
                                  backgroundColor: destination == section
                                      ? colors.surface
                                      : null,
                                  foregroundColor: destination == section
                                      ? colors.onSurface
                                      : colors.onSurfaceVariant,
                                  padding: const EdgeInsets.symmetric(
                                    horizontal: 16,
                                    vertical: 12,
                                  ),
                                  shape: RoundedRectangleBorder(
                                    borderRadius: BorderRadius.circular(8),
                                  ),
                                ),
                                child: Text(label),
                              ),
                            ),
                          ),
                      ],
                    ),
                  ),
                ),
                Expanded(child: child),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _WorkArea {
  const _WorkArea(this.label, this.icon, this.pages);
  final String label;
  final IconData icon;
  final List<(SettingsSection, String)> pages;
}
