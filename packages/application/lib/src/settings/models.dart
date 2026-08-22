enum ThemePreference { light, dark, system }

enum AccessState { checking, granted, denied, notRequired, unknown }

enum ShortcutStatus {
  registered,
  unregistered,
  recording,
  invalid,
  localDuplicate,
  osConflict,
}

enum ProviderTestStatus { idle, testing, passed, failed }

final class GeneralPreferences {
  const GeneralPreferences({
    required this.launchAtLogin,
    required this.showInMenuBar,
    required this.language,
    required this.themeMode,
  });

  final bool launchAtLogin;
  final bool showInMenuBar;
  final String language;
  final ThemePreference themeMode;
}

final class AccessSnapshot {
  const AccessSnapshot({
    required this.accessibility,
    required this.screenRecording,
  });

  const AccessSnapshot.unknown()
    : accessibility = AccessState.unknown,
      screenRecording = AccessState.unknown;

  const AccessSnapshot.notRequired()
    : accessibility = AccessState.notRequired,
      screenRecording = AccessState.notRequired;

  final AccessState accessibility;
  final AccessState screenRecording;

  bool get needsAttention =>
      accessibility == AccessState.denied ||
      screenRecording == AccessState.denied;
}

final class LanguageChoice {
  const LanguageChoice({required this.code, required this.name});

  final String code;
  final String name;
}

final class ServiceRecord {
  const ServiceRecord({
    required this.id,
    required this.name,
    required this.providerId,
    required this.providerName,
    required this.kind,
    required this.enabled,
    required this.isDefault,
  });

  final String id;
  final String name;
  final String providerId;
  final String providerName;
  final String kind;
  final bool enabled;
  final bool isDefault;
}

final class ProviderFieldSpec {
  const ProviderFieldSpec({
    required this.key,
    required this.label,
    required this.secret,
    required this.requiredField,
    this.placeholder,
  });

  final String key;
  final String label;
  final bool secret;
  final bool requiredField;
  final String? placeholder;
}

final class ProviderTypeOption {
  const ProviderTypeOption({
    required this.id,
    required this.label,
    required this.isLlm,
    required this.fields,
  });

  final String id;
  final String label;
  final bool isLlm;
  final List<ProviderFieldSpec> fields;
}

final class ProviderRecord {
  const ProviderRecord({
    required this.id,
    required this.typeId,
    required this.displayName,
    required this.publicFields,
    required this.storedSecretKeys,
  });

  final String id;
  final String typeId;
  final String displayName;
  final Map<String, String> publicFields;
  final Set<String> storedSecretKeys;

  bool get hasStoredSecret => storedSecretKeys.isNotEmpty;
}

final class ProviderDraft {
  const ProviderDraft({
    required this.id,
    required this.typeId,
    required this.fields,
  });

  final String id;
  final String typeId;
  final Map<String, String> fields;
}

final class ProviderTestResult {
  const ProviderTestResult({
    required this.status,
    this.message,
    this.errorCode,
  });

  final ProviderTestStatus status;
  final String? message;
  final String? errorCode;
}

final class ShortcutRecord {
  const ShortcutRecord({
    required this.actionId,
    required this.labelKey,
    required this.accelerator,
    required this.status,
    this.conflictReason,
  });

  final String actionId;
  final String labelKey;
  final String accelerator;
  final ShortcutStatus status;
  final String? conflictReason;
}

final class AboutInfo {
  const AboutInfo({
    required this.appName,
    required this.version,
    required this.buildNumber,
    required this.platformLabel,
    required this.license,
  });

  final String appName;
  final String version;
  final String buildNumber;
  final String platformLabel;
  final String license;
}
