final class SettingsShellLabels {
  const SettingsShellLabels({
    required this.title,
    required this.general,
    required this.services,
    required this.providers,
    required this.shortcuts,
    required this.permissions,
    required this.about,
  });

  final String title;
  final String general;
  final String services;
  final String providers;
  final String shortcuts;
  final String permissions;
  final String about;
}

enum SettingsSection {
  general,
  services,
  providers,
  shortcuts,
  permissions,
  about,
}

final class GeneralSettingsLabels {
  const GeneralSettingsLabels({
    required this.startup,
    required this.launchAtLogin,
    required this.showInMenuBar,
    required this.appearance,
    required this.language,
    required this.theme,
    required this.light,
    required this.dark,
    required this.system,
  });

  final String startup;
  final String launchAtLogin;
  final String showInMenuBar;
  final String appearance;
  final String language;
  final String theme;
  final String light;
  final String dark;
  final String system;
}

final class ServicesSettingsLabels {
  const ServicesSettingsLabels({
    required this.title,
    required this.empty,
    required this.loading,
    required this.translation,
    required this.ocr,
    required this.enabled,
    required this.makeDefault,
    required this.isDefault,
    required this.configureProviders,
    required this.commonLanguages,
    required this.defaultService,
  });

  final String title;
  final String empty;
  final String loading;
  final String translation;
  final String ocr;
  final String enabled;
  final String makeDefault;
  final String isDefault;
  final String configureProviders;
  final String commonLanguages;
  final String defaultService;
}

final class ProvidersSettingsLabels {
  const ProvidersSettingsLabels({
    required this.title,
    required this.empty,
    required this.loading,
    required this.add,
    required this.edit,
    required this.delete,
    required this.deleteConfirmTitle,
    required this.deleteConfirmBody,
    required this.secretStored,
    required this.secretPlaceholder,
    required this.save,
    required this.cancel,
    required this.test,
    required this.testing,
    required this.testPassed,
    required this.testFailed,
    required this.idLabel,
    required this.typeLabel,
    required this.validationMissing,
    required this.saveFailed,
  });

  final String title;
  final String empty;
  final String loading;
  final String add;
  final String edit;
  final String delete;
  final String deleteConfirmTitle;
  final String deleteConfirmBody;
  final String secretStored;
  final String secretPlaceholder;
  final String save;
  final String cancel;
  final String test;
  final String testing;
  final String testPassed;
  final String testFailed;
  final String idLabel;
  final String typeLabel;
  final String validationMissing;
  final String saveFailed;
}

final class ShortcutsSettingsLabels {
  const ShortcutsSettingsLabels({
    required this.title,
    required this.record,
    required this.recording,
    required this.clear,
    required this.reset,
    required this.resetConfirmTitle,
    required this.resetConfirmBody,
    required this.registered,
    required this.unregistered,
    required this.invalid,
    required this.conflict,
    required this.cancel,
    required this.confirm,
  });

  final String title;
  final String record;
  final String recording;
  final String clear;
  final String reset;
  final String resetConfirmTitle;
  final String resetConfirmBody;
  final String registered;
  final String unregistered;
  final String invalid;
  final String Function(String label) conflict;
  final String cancel;
  final String confirm;
}

final class PermissionsSettingsLabels {
  const PermissionsSettingsLabels({
    required this.title,
    required this.accessibility,
    required this.accessibilityHint,
    required this.screenRecording,
    required this.screenRecordingHint,
    required this.granted,
    required this.denied,
    required this.notRequired,
    required this.unknown,
    required this.grant,
    required this.recheck,
    required this.windowsNote,
  });

  final String title;
  final String accessibility;
  final String accessibilityHint;
  final String screenRecording;
  final String screenRecordingHint;
  final String granted;
  final String denied;
  final String notRequired;
  final String unknown;
  final String grant;
  final String recheck;
  final String windowsNote;
}

final class AboutSettingsLabels {
  const AboutSettingsLabels({
    required this.title,
    required this.copyVersion,
    required this.copied,
    required this.license,
    required this.website,
    required this.changelog,
    required this.issues,
    required this.copyright,
  });

  final String title;
  final String copyVersion;
  final String copied;
  final String license;
  final String website;
  final String changelog;
  final String issues;
  final String copyright;
}
