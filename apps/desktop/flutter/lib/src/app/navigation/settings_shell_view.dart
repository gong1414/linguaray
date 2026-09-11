import 'package:flutter/material.dart';
import 'package:linguaray_ui/linguaray_ui.dart' show BrandLogo;

import '../../shared/settings_labels.dart';

/// One grouped directory exposes every settings destination directly.
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
      _WorkArea(labels.translationGroup, [
        (SettingsSection.translation, labels.translationSettings),
        (SettingsSection.translationServices, labels.translationServices),
      ]),
      _WorkArea(
        labels.libraryGroup.isEmpty ? labels.history : labels.libraryGroup,
        [
          (SettingsSection.history, labels.history),
          (SettingsSection.favorites, labels.favorites),
          if (labels.glossary.isNotEmpty)
            (SettingsSection.glossary, labels.glossary),
          if (labels.vocabulary.isNotEmpty)
            (SettingsSection.vocabulary, labels.vocabulary),
        ],
      ),
      _WorkArea(labels.ocrGroup, [
        (SettingsSection.ocr, labels.ocrSettings),
        (SettingsSection.ocrServices, labels.ocrServices),
      ]),
      _WorkArea(labels.generalGroup, [
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
    return Material(
      color: colors.surfaceContainerLowest,
      child: LayoutBuilder(
        builder: (context, constraints) => Row(
          children: [
            Container(
              width: constraints.maxWidth < 900 ? 184 : 208,
              color: colors.surface,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Padding(
                    padding: const EdgeInsets.fromLTRB(20, 28, 16, 24),
                    child: Row(
                      children: [
                        const BrandLogo(size: 28),
                        const SizedBox(width: 10),
                        Expanded(
                          child: Text(
                            'LinguaRay',
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: theme.textTheme.titleMedium,
                          ),
                        ),
                      ],
                    ),
                  ),
                  Expanded(
                    child: ListView(
                      padding: const EdgeInsets.fromLTRB(12, 0, 12, 20),
                      children: [
                        for (final group in groups) ...[
                          Padding(
                            padding: const EdgeInsets.fromLTRB(10, 12, 8, 6),
                            child: Text(
                              group.label,
                              style: theme.textTheme.labelSmall,
                            ),
                          ),
                          for (final (destination, label) in group.pages)
                            Padding(
                              padding: const EdgeInsets.only(bottom: 2),
                              child: Semantics(
                                selected: destination == section,
                                child: TextButton(
                                  key: ValueKey(
                                    'settings-page-${destination.name}',
                                  ),
                                  onPressed: () =>
                                      onSectionSelected(destination),
                                  style: TextButton.styleFrom(
                                    alignment: Alignment.centerLeft,
                                    minimumSize: const Size(0, 36),
                                    padding: const EdgeInsets.symmetric(
                                      horizontal: 10,
                                      vertical: 8,
                                    ),
                                    backgroundColor: destination == section
                                        ? colors.primary
                                        : null,
                                    foregroundColor: destination == section
                                        ? colors.onPrimary
                                        : colors.onSurfaceVariant,
                                    shape: RoundedRectangleBorder(
                                      borderRadius: BorderRadius.circular(8),
                                    ),
                                  ),
                                  child: Row(
                                    children: [
                                      Icon(_icon(destination), size: 17),
                                      const SizedBox(width: 10),
                                      Expanded(
                                        child: Text(
                                          label,
                                          maxLines: 1,
                                          overflow: TextOverflow.ellipsis,
                                        ),
                                      ),
                                    ],
                                  ),
                                ),
                              ),
                            ),
                        ],
                      ],
                    ),
                  ),
                ],
              ),
            ),
            Expanded(child: child),
          ],
        ),
      ),
    );
  }

  static IconData _icon(SettingsSection section) => switch (section) {
    SettingsSection.translation => Icons.translate_rounded,
    SettingsSection.translationServices => Icons.view_list_outlined,
    SettingsSection.history => Icons.history_rounded,
    SettingsSection.favorites => Icons.star_outline_rounded,
    SettingsSection.glossary => Icons.menu_book_outlined,
    SettingsSection.vocabulary => Icons.bookmark_border_rounded,
    SettingsSection.ocr => Icons.document_scanner_outlined,
    SettingsSection.ocrServices => Icons.layers_outlined,
    SettingsSection.general => Icons.tune_rounded,
    SettingsSection.permissions => Icons.lock_outline_rounded,
    SettingsSection.dataTransfer => Icons.import_export_rounded,
    SettingsSection.integration => Icons.extension_outlined,
    SettingsSection.updates => Icons.system_update_alt_rounded,
    SettingsSection.about => Icons.info_outline_rounded,
  };
}

class _WorkArea {
  const _WorkArea(this.label, this.pages);
  final String label;
  final List<(SettingsSection, String)> pages;
}
