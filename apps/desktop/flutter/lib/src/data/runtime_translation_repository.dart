import 'dart:io';

import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_runtime/linguaray_runtime.dart'
    show ErrorExceptionRuntimeException;

import '../services/llm_stream.dart';
import '../services/runtime.dart';
import '../services/settings_store.dart';

final class RuntimeTranslationRepository implements TranslationRepository {
  final Map<String, ProviderType> _providerTypesById = {};

  @override
  Future<TranslationCatalog> loadCatalog() async {
    final settings = runtime.settings();
    final providers = await settings.listProviders();
    final services = await settings.listServices();
    _providerTypesById
      ..clear()
      ..addEntries(
        providers.map((provider) => MapEntry(provider.id, provider.type)),
      );

    final languages = runtime
        .listLanguages()
        .map(
          (language) =>
              LanguageOption(code: language.code, name: language.localName),
        )
        .toList(growable: false);

    final translationServices = services
        .where((service) {
          if (service.type != ServiceType.translation) return false;
          if (service.fields[_serviceEnabledField] == 'false') return false;
          final providerType = _providerTypesById[service.providerId];
          if (providerType == ProviderType.system &&
              !const PlatformCapabilities.windows().systemTranslation &&
              _isWindows) {
            return false;
          }
          return true;
        })
        .map(
          (service) => TranslationServiceOption(
            id: service.id,
            name: service.name.trim().isEmpty ? service.id : service.name,
            isStreaming: _isStreamingProvider(
              _providerTypesById[service.providerId],
            ),
            omitsSourceLanguage: _isStreamingProvider(
              _providerTypesById[service.providerId],
            ),
          ),
        )
        .toList(growable: false);

    final configuredTargets = settingsStore.general.translationTargets
        .where((target) => target.enabled)
        .toList(growable: false);
    final defaultTarget = configuredTargets.isNotEmpty
        ? configuredTargets.first.target
        : _fallbackTarget(languages);
    final defaultSource = configuredTargets.isNotEmpty
        ? configuredTargets.first.source
        : autoLanguageCode;

    return TranslationCatalog(
      languages: languages,
      services: translationServices,
      defaultSourceLanguage: defaultSource.trim().isEmpty
          ? autoLanguageCode
          : defaultSource,
      defaultTargetLanguage: defaultTarget,
    );
  }

  @override
  Future<String?> detectLanguage({
    required String serviceId,
    required String text,
  }) async {
    final response = await runtime
        .translation(providerId: serviceId)
        .detectLanguage(request: DetectLanguageRequest(texts: [text]));
    final detections = response.detections;
    if (detections == null || detections.isEmpty) return null;
    return detections.first.detectedLanguage;
  }

  @override
  Future<String> resolveTarget({
    required String? selectedTarget,
    required String fallbackTarget,
    required String? detectedLanguage,
  }) async {
    if (selectedTarget != null && selectedTarget.trim().isNotEmpty) {
      return selectedTarget;
    }

    final configuredTargets = settingsStore.general.translationTargets
        .where((target) => target.enabled)
        .toList(growable: false);
    if (configuredTargets.isEmpty) return fallbackTarget;

    final active = await runtime.settings().getActiveTranslationTargets(
      targets: configuredTargets,
      detectedLanguage: detectedLanguage,
    );
    return active.isEmpty ? fallbackTarget : active.first.target;
  }

  @override
  Stream<String> translate({
    required TranslationServiceOption service,
    required String text,
    required String sourceLanguage,
    required String targetLanguage,
  }) async* {
    if (service.isStreaming) {
      try {
        await for (final chunk in LlmStream.translate(
          providerId: service.id,
          sourceLang: sourceLanguage,
          targetLang: targetLanguage,
          text: text,
        )) {
          if (chunk.content.isNotEmpty) yield chunk.content;
        }
      } catch (error) {
        throw _translationFailure(error);
      }
      return;
    }

    try {
      final response = await runtime
          .translation(providerId: service.id)
          .translate(
            request: TranslateRequest(
              sourceLanguage: sourceLanguage == autoLanguageCode
                  ? null
                  : sourceLanguage,
              targetLanguage: targetLanguage,
              text: text,
            ),
          );
      if (response.translations.isEmpty) {
        throw const TranslationFailure('empty_result');
      }
      final translated = response.translations.first.text;
      if (translated.trim().isEmpty) {
        throw const TranslationFailure('empty_result');
      }
      yield translated;
    } on TranslationFailure {
      rethrow;
    } catch (error) {
      throw _translationFailure(error);
    }
  }

  String _fallbackTarget(List<LanguageOption> languages) {
    const preferred = ['zh-Hans', 'en'];
    for (final code in preferred) {
      if (languages.any((language) => language.code == code)) return code;
    }
    return languages.isEmpty ? 'en' : languages.first.code;
  }

  bool _isStreamingProvider(ProviderType? type) {
    return switch (type) {
      ProviderType.anthropic ||
      ProviderType.deepSeek ||
      ProviderType.doubao ||
      ProviderType.gemini ||
      ProviderType.groq ||
      ProviderType.moonshot ||
      ProviderType.openAi ||
      ProviderType.openAiCompatible ||
      ProviderType.ollama ||
      ProviderType.qwen ||
      ProviderType.xAi ||
      ProviderType.zhipu => true,
      _ => false,
    };
  }

  TranslationFailure _translationFailure(Object error) {
    final message = switch (error) {
      ErrorExceptionRuntimeException(:final msg) => msg,
      _ => error.toString(),
    }.toLowerCase();
    if (message.contains('not installed')) {
      return const TranslationFailure('language_pair_not_installed');
    }
    if (message.contains('unsupported language') ||
        message.contains('unsupported pair')) {
      return const TranslationFailure('unsupported_language_pair');
    }
    if (message.contains('unable to detect source language') ||
        message.contains('language detection failed')) {
      return const TranslationFailure('source_language_detection_failed');
    }
    if (message.contains('network') || message.contains('connection')) {
      return const TranslationFailure('network_failure');
    }
    if (message.contains('401') ||
        message.contains('unauthorized') ||
        message.contains('auth')) {
      return const TranslationFailure('provider_auth_failed');
    }
    return TranslationFailure(mapErrorCode(message).wireName);
  }
}

bool get _isWindows => Platform.isWindows;

const String _serviceEnabledField = 'enabled';
