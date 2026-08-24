enum SettingsSection {
  translation,
  translationServices,
  favorites,
  history,
  glossary,
  vocabulary,
  ocr,
  ocrServices,
  general,
  permissions,
  integration,
  dataTransfer,
  updates,
  about,
}

enum SettingsDestination {
  settingsTranslation('/settings/translation', SettingsSection.translation),
  settingsTranslationServices(
    '/settings/services/translation',
    SettingsSection.translationServices,
  ),
  settingsFavorites('/settings/favorites', SettingsSection.favorites),
  settingsHistory('/settings/history', SettingsSection.history),
  settingsGlossary('/settings/glossary', SettingsSection.glossary),
  settingsVocabulary('/settings/vocabulary', SettingsSection.vocabulary),
  settingsOcr('/settings/ocr', SettingsSection.ocr),
  settingsOcrServices('/settings/services/ocr', SettingsSection.ocrServices),
  settingsGeneral('/settings/general', SettingsSection.general),
  settingsPermissions('/settings/permissions', SettingsSection.permissions),
  settingsDataTransfer('/settings/data-transfer', SettingsSection.dataTransfer),
  settingsIntegration('/settings/integration', SettingsSection.integration),
  settingsUpdates('/settings/updates', SettingsSection.updates),
  settingsAbout('/settings/about', SettingsSection.about);

  const SettingsDestination(this.location, this.section);

  final String location;
  final SettingsSection section;
}

SettingsDestination settingsDestinationForLocation(String location) {
  final path = Uri.tryParse(location)?.path ?? location;
  for (final destination in SettingsDestination.values) {
    if (path == destination.location ||
        path.startsWith('${destination.location}/')) {
      return destination;
    }
  }
  return SettingsDestination.settingsGeneral;
}

extension SettingsSectionNavigation on SettingsSection {
  SettingsDestination get destination {
    for (final destination in SettingsDestination.values) {
      if (destination.section == this) return destination;
    }
    throw StateError('No destination registered for $this');
  }
}
