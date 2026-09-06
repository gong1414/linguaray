import 'dart:async';

import 'package:linguaray_application/src/errors/mapping.dart';
import 'package:linguaray_application/src/translation/models.dart';
import 'package:linguaray_application/src/translation/ports.dart';

final class TranslateText {
  const TranslateText(this._repository);

  final TranslationRepository _repository;

  Stream<TranslationRun> call({
    required TranslationQuery query,
    required TranslationCatalog catalog,
  }) {
    final operation = _TranslationOperation();
    final controller = StreamController<TranslationRun>(
      onCancel: operation.cancel,
    );
    unawaited(_run(query, catalog, controller, operation));
    return controller.stream;
  }

  Future<void> _run(
    TranslationQuery query,
    TranslationCatalog catalog,
    StreamController<TranslationRun> controller,
    _TranslationOperation operation,
  ) async {
    final sourceText = query.text.trim();
    if (sourceText.isEmpty) {
      await controller.close();
      return;
    }

    String? detectedLanguage;
    if (catalog.services.isNotEmpty &&
        (query.sourceLanguage == autoLanguageCode ||
            query.targetLanguage == null)) {
      try {
        detectedLanguage = query.sourceLanguage != autoLanguageCode
            ? query.sourceLanguage
            : await _repository
                  .detectLanguage(
                    serviceId: catalog.services.first.id,
                    text: sourceText,
                  )
                  .timeout(const Duration(seconds: 3));
      } catch (_) {
        // Detection is supplemental. Translation remains usable when a
        // provider cannot detect language independently.
      }
    }

    if (operation.cancelled) return;
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

    if (operation.cancelled) return;
    final orderedIds = [for (final service in catalog.services) service.id];
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
          requestedSourceLanguage: query.sourceLanguage,
          detectedLanguage: detectedLanguage,
          targetLanguage: targetLanguage,
          results: results,
          operation: operation,
          emit: () {
            if (!operation.cancelled) controller.add(snapshot(complete: false));
          },
        ),
    ]);

    if (!operation.cancelled) {
      controller.add(snapshot(complete: true));
      await controller.close();
    }
  }

  Future<void> _translateService({
    required TranslationServiceOption service,
    required String sourceText,
    required String requestedSourceLanguage,
    required String? detectedLanguage,
    required String targetLanguage,
    required Map<String, ServiceTranslationResult> results,
    required void Function() emit,
    required _TranslationOperation operation,
  }) async {
    final sourceLanguage = _sourceForService(
      service: service,
      requested: requestedSourceLanguage,
      detected: detectedLanguage,
    );
    final buffer = StringBuffer();
    try {
      final done = Completer<void>();
      final subscription = _repository
          .translate(
            service: service,
            text: sourceText,
            sourceLanguage: sourceLanguage,
            targetLanguage: targetLanguage,
          )
          .listen(
            (chunk) {
              if (operation.cancelled || chunk.isEmpty) return;
              buffer.write(chunk);
              results[service.id] = results[service.id]!.copyWith(
                text: buffer.toString(),
                status: TranslationResultStatus.translating,
                clearError: true,
              );
              emit();
            },
            onError: (Object error, StackTrace stack) {
              if (!done.isCompleted) done.completeError(error, stack);
            },
            onDone: () {
              if (!done.isCompleted) done.complete();
            },
            cancelOnError: true,
          );
      operation.track(subscription, done);
      await done.future;
      if (operation.cancelled) return;

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
        errorCode: mapErrorCode(error.code).wireName,
      );
    } catch (_) {
      results[service.id] = results[service.id]!.copyWith(
        status: TranslationResultStatus.failed,
        errorCode: 'translation_failed',
      );
    }
    emit();
  }

  String _sourceForService({
    required TranslationServiceOption service,
    required String requested,
    required String? detected,
  }) {
    if (requested != autoLanguageCode) return requested;
    if (detected != null && detected.isNotEmpty) return detected;
    if (service.omitsSourceLanguage) return autoLanguageCode;
    return autoLanguageCode;
  }
}

/// Owns the subscriptions so cancellation propagates even while a provider is silent.
final class _TranslationOperation {
  bool cancelled = false;
  final _subscriptions = <StreamSubscription<String>>[];
  final _completions = <Completer<void>>[];

  void track(StreamSubscription<String> subscription, Completer<void> done) {
    _subscriptions.add(subscription);
    _completions.add(done);
    if (cancelled) {
      unawaited(subscription.cancel());
      if (!done.isCompleted) done.complete();
    }
  }

  Future<void> cancel() async {
    cancelled = true;
    for (final done in _completions) {
      if (!done.isCompleted) done.complete();
    }
    await Future.wait([
      for (final subscription in _subscriptions) subscription.cancel(),
    ]);
  }
}
