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

enum InputSubmitMode { enter, commandEnter }

final class GeneralPreferences {
  const GeneralPreferences({
    required this.launchAtLogin,
    required this.showInMenuBar,
    required this.language,
    required this.themeMode,
    this.commonLanguages = const [],
    this.translationTargets = const [],
    this.inputSubmitMode = InputSubmitMode.enter,
    this.autoCopyDetectedText = true,
    this.doubleClickCopyResult = true,
    this.defaultTranslationService,
    this.defaultOcrService,
    this.defaultDictionaryService,
  });

  final bool launchAtLogin;
  final bool showInMenuBar;
  final String language;
  final ThemePreference themeMode;
  final List<String> commonLanguages;
  final List<TranslationTargetRule> translationTargets;
  final InputSubmitMode inputSubmitMode;
  final bool autoCopyDetectedText;
  final bool doubleClickCopyResult;
  final String? defaultTranslationService;
  final String? defaultOcrService;
  final String? defaultDictionaryService;

  GeneralPreferences copyWith({
    bool? launchAtLogin,
    bool? showInMenuBar,
    String? language,
    ThemePreference? themeMode,
    List<String>? commonLanguages,
    List<TranslationTargetRule>? translationTargets,
    InputSubmitMode? inputSubmitMode,
    bool? autoCopyDetectedText,
    bool? doubleClickCopyResult,
    Object? defaultTranslationService = _unset,
    Object? defaultOcrService = _unset,
    Object? defaultDictionaryService = _unset,
  }) {
    return GeneralPreferences(
      launchAtLogin: launchAtLogin ?? this.launchAtLogin,
      showInMenuBar: showInMenuBar ?? this.showInMenuBar,
      language: language ?? this.language,
      themeMode: themeMode ?? this.themeMode,
      commonLanguages: commonLanguages ?? this.commonLanguages,
      translationTargets: translationTargets ?? this.translationTargets,
      inputSubmitMode: inputSubmitMode ?? this.inputSubmitMode,
      autoCopyDetectedText: autoCopyDetectedText ?? this.autoCopyDetectedText,
      doubleClickCopyResult:
          doubleClickCopyResult ?? this.doubleClickCopyResult,
      defaultTranslationService: identical(defaultTranslationService, _unset)
          ? this.defaultTranslationService
          : defaultTranslationService as String?,
      defaultOcrService: identical(defaultOcrService, _unset)
          ? this.defaultOcrService
          : defaultOcrService as String?,
      defaultDictionaryService: identical(defaultDictionaryService, _unset)
          ? this.defaultDictionaryService
          : defaultDictionaryService as String?,
    );
  }
}

const Object _unset = Object();

final class TranslationTargetRule {
  const TranslationTargetRule({
    required this.source,
    required this.target,
    this.enabled = true,
  });

  final String source;
  final String target;
  final bool enabled;

  @override
  bool operator ==(Object other) =>
      other is TranslationTargetRule &&
      other.source == source &&
      other.target == target &&
      other.enabled == enabled;

  @override
  int get hashCode => Object.hash(source, target, enabled);
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
    this.synthesized = true,
    this.usable = true,
  });

  final String id;
  final String name;
  final String providerId;
  final String providerName;
  final String kind;
  final bool enabled;
  final bool isDefault;

  /// True when this service is the default synthesized `{provider}+{kind}`
  /// entry rather than a user-created extra configuration.
  final bool synthesized;

  /// False when the platform cannot actually run this service, e.g. Windows
  /// system translation.
  final bool usable;
}

final class ServiceDraft {
  const ServiceDraft({
    this.id,
    required this.providerId,
    required this.kind,
    required this.name,
    this.fields = const {},
  });

  final String? id;
  final String providerId;
  final String kind;
  final String name;
  final Map<String, String> fields;
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
    this.supportsTranslation = true,
    this.supportsOcr = false,
    this.supportsDictionary = false,
  });

  final String id;
  final String label;
  final bool isLlm;
  final List<ProviderFieldSpec> fields;
  final bool supportsTranslation;
  final bool supportsOcr;
  final bool supportsDictionary;
}

final class ProviderRecord {
  const ProviderRecord({
    required this.id,
    required this.typeId,
    required this.displayName,
    required this.publicFields,
    required this.storedSecretKeys,
    this.usableForTranslation = true,
  });

  final String id;
  final String typeId;
  final String displayName;
  final Map<String, String> publicFields;
  final Set<String> storedSecretKeys;
  final bool usableForTranslation;

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

final class ApiServerStatus {
  const ApiServerStatus({
    required this.enabled,
    required this.host,
    required this.port,
    this.baseUrl,
    this.bindErrorCode,
  });

  final bool enabled;
  final String host;
  final int port;
  final String? baseUrl;
  final String? bindErrorCode;

  bool get running => baseUrl != null && baseUrl!.isNotEmpty;
}

final class PlatformCapabilities {
  const PlatformCapabilities({
    required this.systemTranslation,
    required this.systemLanguageDetection,
    required this.systemOcr,
    required this.systemDictionary,
    required this.accessibilityRequired,
    required this.screenRecordingRequired,
  });

  const PlatformCapabilities.macos()
    : systemTranslation = true,
      systemLanguageDetection = true,
      systemOcr = true,
      systemDictionary = true,
      accessibilityRequired = true,
      screenRecordingRequired = true;

  const PlatformCapabilities.windows()
    : systemTranslation = false,
      systemLanguageDetection = false,
      systemOcr = true,
      systemDictionary = false,
      accessibilityRequired = false,
      screenRecordingRequired = false;

  final bool systemTranslation;
  final bool systemLanguageDetection;
  final bool systemOcr;
  final bool systemDictionary;
  final bool accessibilityRequired;
  final bool screenRecordingRequired;
}
