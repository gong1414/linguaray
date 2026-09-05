import 'package:flutter/material.dart';
import 'package:linguaray_desktop/src/ui/catalog/catalog.dart';
import 'package:linguaray_desktop/src/ui/settings/settings_labels.dart';
import 'package:linguaray_ui/linguaray_ui.dart' show LinguaRayMaterialTheme;
import 'package:widgetbook/widgetbook.dart';

export 'src/ui/catalog/catalog.dart';

void main() {
  runApp(
    Widgetbook.material(
      directories: [_catalog],
      addons: [
        ThemeAddon<ThemeData>(
          themes: [
            WidgetbookTheme(
              name: 'Light',
              data: LinguaRayMaterialTheme.light(),
            ),
            WidgetbookTheme(name: 'Dark', data: LinguaRayMaterialTheme.dark()),
          ],
          themeBuilder: (context, theme, child) =>
              Theme(data: theme, child: child),
        ),
        ViewportAddon(const [
          ViewportData(
            name: 'Settings · macOS',
            width: 1000,
            height: 700,
            pixelRatio: 2,
            platform: TargetPlatform.macOS,
          ),
          ViewportData(
            name: 'Settings · Windows',
            width: 1000,
            height: 700,
            pixelRatio: 1,
            platform: TargetPlatform.windows,
          ),
          ViewportData(
            name: 'Settings min · 840×560',
            width: 840,
            height: 560,
            pixelRatio: 1,
            platform: TargetPlatform.macOS,
          ),
          ViewportData(
            name: 'Compact desktop',
            width: 620,
            height: 760,
            pixelRatio: 1,
            platform: TargetPlatform.windows,
          ),
          ViewportData(
            name: 'Reading window',
            width: 720,
            height: 420,
            pixelRatio: 2,
            platform: TargetPlatform.macOS,
          ),
          ViewportData(
            name: 'Quick translate',
            width: 396,
            height: 640,
            pixelRatio: 2,
            platform: TargetPlatform.macOS,
          ),
          ViewportData(
            name: 'OCR window',
            width: 600,
            height: 520,
            pixelRatio: 2,
            platform: TargetPlatform.macOS,
          ),
        ]),
      ],
    ),
  );
}

final _catalog = WidgetbookCategory(
  name: 'LinguaRay',
  children: [
    WidgetbookComponent(
      name: 'History and updates',
      useCases: [
        WidgetbookUseCase(
          name: 'History · empty',
          builder: (_) => const HistoryCatalogPreview(empty: true),
        ),
        WidgetbookUseCase(
          name: 'History · populated',
          builder: (_) => const HistoryCatalogPreview(empty: false),
        ),
        WidgetbookUseCase(
          name: 'Updates · available',
          builder: (_) => const UpdatesCatalogPreview(),
        ),
      ],
    ),
    WidgetbookComponent(
      name: 'Quick translate',
      useCases: [
        for (final scenario in CatalogQuickScenario.values)
          WidgetbookUseCase(
            name: scenario.name,
            builder: (_) => QuickTranslateCatalogPreview(scenario: scenario),
          ),
      ],
    ),
    WidgetbookComponent(
      name: 'OCR',
      useCases: [
        for (final scenario in CatalogOcrScenario.values)
          WidgetbookUseCase(
            name: scenario.name,
            builder: (_) => OcrCatalogPreview(scenario: scenario),
          ),
      ],
    ),
    WidgetbookComponent(
      name: 'Settings',
      useCases: [
        for (final section in SettingsSection.values)
          WidgetbookUseCase(
            name: section.name,
            builder: (_) => SettingsCatalogPreview(section: section),
          ),
        WidgetbookUseCase(
          name: 'Translation services empty',
          builder: (_) => const SettingsCatalogPreview(
            section: SettingsSection.translationServices,
            servicesEmpty: true,
          ),
        ),
        WidgetbookUseCase(
          name: 'Services empty',
          builder: (_) => const SettingsCatalogPreview(
            section: SettingsSection.ocrServices,
            servicesEmpty: true,
          ),
        ),
        WidgetbookUseCase(
          name: 'Shortcut conflict',
          builder: (_) => const SettingsCatalogPreview(
            section: SettingsSection.translation,
            shortcutConflict: true,
          ),
        ),
        WidgetbookUseCase(
          name: 'Providers · configured',
          builder: (_) => const ProvidersCatalogPreview(),
        ),
        WidgetbookUseCase(
          name: 'Provider models · live discovery',
          builder: (_) => const ProviderModelsCatalogPreview(),
        ),
        WidgetbookUseCase(
          name: 'Provider models · authentication failure',
          builder: (_) => const ProviderModelsCatalogPreview(failed: true),
        ),
        WidgetbookUseCase(
          name: 'Provider editor · secret stored',
          builder: (_) =>
              const ProviderEditorCatalogPreview(secretStored: true),
        ),
        WidgetbookUseCase(
          name: 'Provider editor · test failure',
          builder: (_) => const ProviderEditorCatalogPreview(failed: true),
        ),
        WidgetbookUseCase(
          name: 'General · English',
          builder: (_) => const SettingsCatalogPreview(
            section: SettingsSection.general,
            english: true,
          ),
        ),
      ],
    ),
  ],
);
