import 'dart:async';

import 'package:linguaray_application/src/translation/models.dart';
import 'package:linguaray_application/src/translation/ports.dart';

final class TranslateText {
  const TranslateText(this._repository);

  final TranslationRepository _repository;

  Stream<TranslationRun> call({
    required TranslationQuery query,
    required TranslationCatalog catalog,
  }) {
    final controller = StreamController<TranslationRun>();
    unawaited(_run(query, catalog, controller));
    return controller.stream;
  }

  Future<void> _run(
    TranslationQuery query,
    TranslationCatalog catalog,
    StreamController<TranslationRun> controller,
  ) async {
    final sourceText = query.text.trim();
    if (sourceText.isEmpty) {
      await controller.close();
      return;
    }

    String? detectedLanguage;
    if (catalog.services.isNotEmpty) {
      try {
        detectedLanguage = await _repository.detectLanguage(
          serviceId: catalog.services.first.id,
          text: sourceText,
        );
      } catch (_) {
        // Detection is supplemental. Translation remains usable when a
        // provider cannot detect language independently.
      }
    }

    var targetLanguage = query.targetLanguage ?? catalog.defaultTargetLanguage;
    try {
      targetLanguage = await _repository.resolveTarget(
        selectedTarget: query.targetLanguage,
        fallbackTarget: catalog.defaultTargetLanguage,
        detectedLanguage: detectedLanguage,
      );
    } catch (_) {
      // The concrete fallback is always valid for a translation request.
    }

    final orderedIds = [for (final service in catalog.services) service.id];
    final resolvedSourceLanguage = query.sourceLanguage == autoLanguageCode
        ? detectedLanguage ?? autoLanguageCode
        : query.sourceLanguage;
    final results = <String, ServiceTranslationResult>{
      for (final service in catalog.services)
        service.id: ServiceTranslationResult(
          service: service,
          status: TranslationResultStatus.translating,
        ),
    };

    TranslationRun snapshot({required bool complete}) => TranslationRun(
      sourceText: sourceText,
      sourceLanguage: query.sourceLanguage,
      targetLanguage: targetLanguage,
      detectedLanguage: detectedLanguage,
      results: [for (final id in orderedIds) results[id]!],
      complete: complete,
    );

    if (catalog.services.isEmpty) {
      controller.add(snapshot(complete: true));
      await controller.close();
      return;
    }

    controller.add(snapshot(complete: false));

    await Future.wait([
      for (final service in catalog.services)
        _translateService(
          service: service,
          sourceText: sourceText,
          sourceLanguage: resolvedSourceLanguage,
          targetLanguage: targetLanguage,
          results: results,
          emit: () => controller.add(snapshot(complete: false)),
        ),
    ]);

    controller.add(snapshot(complete: true));
    await controller.close();
  }

  Future<void> _translateService({
    required TranslationServiceOption service,
    required String sourceText,
    required String sourceLanguage,
    required String targetLanguage,
    required Map<String, ServiceTranslationResult> results,
    required void Function() emit,
  }) async {
    final buffer = StringBuffer();
    try {
      await for (final chunk in _repository.translate(
        service: service,
        text: sourceText,
        sourceLanguage: sourceLanguage,
        targetLanguage: targetLanguage,
      )) {
        if (chunk.isEmpty) continue;
        buffer.write(chunk);
        results[service.id] = results[service.id]!.copyWith(
          text: buffer.toString(),
          status: TranslationResultStatus.translating,
          clearError: true,
        );
        emit();
      }

      results[service.id] = results[service.id]!.copyWith(
        text: buffer.toString(),
        status: buffer.isEmpty
            ? TranslationResultStatus.failed
            : TranslationResultStatus.completed,
        errorCode: buffer.isEmpty ? 'empty_result' : null,
        clearError: buffer.isNotEmpty,
      );
    } on TranslationFailure catch (error) {
      results[service.id] = results[service.id]!.copyWith(
        status: TranslationResultStatus.failed,
        errorCode: error.code,
      );
    } catch (_) {
      results[service.id] = results[service.id]!.copyWith(
        status: TranslationResultStatus.failed,
        errorCode: 'translation_failed',
      );
    }
    emit();
  }
}
