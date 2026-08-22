const String autoLanguageCode = 'auto';
const String automaticTargetCode = 'auto-target';

final class LanguageOption {
  const LanguageOption({required this.code, required this.name});

  final String code;
  final String name;

  @override
  bool operator ==(Object other) =>
      other is LanguageOption && other.code == code && other.name == name;

  @override
  int get hashCode => Object.hash(code, name);
}

final class TranslationServiceOption {
  const TranslationServiceOption({
    required this.id,
    required this.name,
    required this.isStreaming,
  });

  final String id;
  final String name;
  final bool isStreaming;

  @override
  bool operator ==(Object other) =>
      other is TranslationServiceOption &&
      other.id == id &&
      other.name == name &&
      other.isStreaming == isStreaming;

  @override
  int get hashCode => Object.hash(id, name, isStreaming);
}

final class TranslationCatalog {
  const TranslationCatalog({
    required this.languages,
    required this.services,
    required this.defaultSourceLanguage,
    required this.defaultTargetLanguage,
  });

  final List<LanguageOption> languages;
  final List<TranslationServiceOption> services;
  final String defaultSourceLanguage;
  final String defaultTargetLanguage;
}

final class TranslationQuery {
  const TranslationQuery({
    required this.text,
    required this.sourceLanguage,
    required this.targetLanguage,
  });

  final String text;
  final String sourceLanguage;
  final String? targetLanguage;
}

enum TranslationResultStatus { waiting, translating, completed, failed }

final class ServiceTranslationResult {
  const ServiceTranslationResult({
    required this.service,
    this.text = '',
    this.status = TranslationResultStatus.waiting,
    this.errorCode,
  });

  final TranslationServiceOption service;
  final String text;
  final TranslationResultStatus status;
  final String? errorCode;

  bool get hasText => text.trim().isNotEmpty;

  ServiceTranslationResult copyWith({
    String? text,
    TranslationResultStatus? status,
    String? errorCode,
    bool clearError = false,
  }) {
    return ServiceTranslationResult(
      service: service,
      text: text ?? this.text,
      status: status ?? this.status,
      errorCode: clearError ? null : errorCode ?? this.errorCode,
    );
  }
}

final class TranslationRun {
  const TranslationRun({
    required this.sourceText,
    required this.sourceLanguage,
    required this.targetLanguage,
    required this.results,
    this.detectedLanguage,
    this.complete = false,
  });

  final String sourceText;
  final String sourceLanguage;
  final String targetLanguage;
  final String? detectedLanguage;
  final List<ServiceTranslationResult> results;
  final bool complete;
}

final class TranslationFailure implements Exception {
  const TranslationFailure(this.code);

  final String code;

  @override
  String toString() => 'TranslationFailure($code)';
}
