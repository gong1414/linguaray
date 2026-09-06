import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../app/dependencies.dart';
import 'translation_session_id.dart';

final translationViewModelProvider =
    NotifierProvider<TranslationViewModel, TranslationViewState>(
      TranslationViewModel.new,
    );

final class TranslationViewModel extends Notifier<TranslationViewState> {
  int _requestId = 0;
  String? _activeHistorySessionId;
  StreamSubscription<TranslationRun>? _subscription;
  Completer<void>? _completion;
  String? _queryKey;

  String get _currentQueryKey =>
      '${state.sourceText.trim()}\u0000${state.sourceLanguage}\u0000${state.targetLanguage}';

  void cancel() {
    _requestId++;
    unawaited(_subscription?.cancel());
    _subscription = null;
    if (_completion?.isCompleted == false) _completion!.complete();
    final run = state.run;
    state = state.copyWith(
      submitting: false,
      run: run == null
          ? null
          : TranslationRun(
              sourceText: run.sourceText,
              sourceLanguage: run.sourceLanguage,
              targetLanguage: run.targetLanguage,
              detectedLanguage: run.detectedLanguage,
              complete: true,
              results: [
                for (final result in run.results)
                  result.status == TranslationResultStatus.translating
                      ? result.copyWith(
                          status: TranslationResultStatus.cancelled,
                        )
                      : result,
              ],
            ),
    );
  }

  @override
  TranslationViewState build() {
    ref.onDispose(() {
      _requestId++;
      unawaited(_subscription?.cancel());
      if (_completion?.isCompleted == false) _completion!.complete();
    });
    scheduleMicrotask(initialize);
    return const TranslationViewState();
  }

  Future<void> initialize() async {
    state = state.copyWith(loadingCatalog: true, clearCatalogError: true);
    try {
      final catalog = await ref.read(loadTranslationCatalogProvider)();
      final selectedServiceId = catalog.services.isEmpty
          ? null
          : catalog.services.first.id;
      state = state.copyWith(
        catalog: catalog,
        sourceLanguage: catalog.defaultSourceLanguage,
        targetLanguage: automaticTargetCode,
        selectedServiceId: selectedServiceId,
        loadingCatalog: false,
        clearCatalogError: true,
      );
    } catch (_) {
      state = state.copyWith(
        loadingCatalog: false,
        catalogErrorCode: 'catalog_unavailable',
      );
    }
  }

  void setSourceText(String value) {
    if (value != state.sourceText && state.submitting) cancel();
    state = state.copyWith(sourceText: value);
  }

  void clearSourceText() {
    cancel();
    _queryKey = null;
    _activeHistorySessionId = null;
    state = state.copyWith(
      sourceText: '',
      submitting: false,
      clearRun: true,
      clearSubmissionError: true,
      historyByService: const {},
    );
  }

  void setSourceLanguage(String value) {
    if (value != state.sourceLanguage && state.submitting) cancel();
    state = state.copyWith(sourceLanguage: value);
  }

  void setTargetLanguage(String value) {
    if (value != state.targetLanguage && state.submitting) cancel();
    state = state.copyWith(targetLanguage: value);
  }

  void selectService(String serviceId) {
    if (!state.services.any((service) => service.id == serviceId)) return;
    state = state.copyWith(selectedServiceId: serviceId);
    final cached = state.run?.results
        .where((result) => result.service.id == serviceId)
        .firstOrNull;
    if (_queryKey == _currentQueryKey &&
        cached?.status == TranslationResultStatus.completed) {
      unawaited(_refreshGlossary(state.run!));
    } else if (state.sourceText.trim().isNotEmpty) {
      unawaited(submit(reuseResults: true));
    }
  }

  void swapLanguages() {
    final catalog = state.catalog;
    if (catalog == null) return;

    final oldSource = state.sourceLanguage;
    final oldTarget = state.targetLanguage == automaticTargetCode
        ? state.run?.targetLanguage ?? catalog.defaultTargetLanguage
        : state.targetLanguage;
    final detected = state.run?.detectedLanguage;
    final nextTarget = oldSource == autoLanguageCode
        ? detected ?? catalog.defaultTargetLanguage
        : oldSource;
    state = state.copyWith(
      sourceLanguage: oldTarget,
      targetLanguage: nextTarget,
    );
  }

  Future<void> submit({bool reuseResults = false}) async {
    final catalog = state.catalog;
    final text = state.sourceText.trim();
    if (catalog == null ||
        text.isEmpty ||
        state.loadingCatalog ||
        catalog.services.isEmpty) {
      return;
    }
    final service =
        catalog.services
            .where((item) => item.id == state.selectedServiceId)
            .firstOrNull ??
        catalog.services.first;
    final keep = reuseResults && _queryKey == _currentQueryKey;
    cancel();
    final previous = keep
        ? state.run?.results ?? const <ServiceTranslationResult>[]
        : const <ServiceTranslationResult>[];
    final requestId = ++_requestId;
    final sessionId = keep
        ? _activeHistorySessionId ?? newTranslationSessionId()
        : newTranslationSessionId();
    final comparisonTarget = keep ? state.run?.targetLanguage : null;
    _activeHistorySessionId = sessionId;
    _queryKey = _currentQueryKey;
    final completion = Completer<void>();
    _completion = completion;
    state = state.copyWith(
      submitting: true,
      clearRun: !keep,
      clearSubmissionError: true,
      selectedServiceId: service.id,
      historyByService: keep ? state.historyByService : const {},
    );
    _subscription = ref
        .read(translateTextProvider)(
          query: TranslationQuery(
            text: text,
            sourceLanguage: state.sourceLanguage,
            targetLanguage: state.targetLanguage == automaticTargetCode
                ? comparisonTarget
                : state.targetLanguage,
          ),
          catalog: TranslationCatalog(
            languages: catalog.languages,
            services: [service],
            defaultSourceLanguage: catalog.defaultSourceLanguage,
            defaultTargetLanguage: catalog.defaultTargetLanguage,
          ),
        )
        .listen(
          (run) {
            if (requestId != _requestId) return;
            final merged = TranslationRun(
              sourceText: run.sourceText,
              sourceLanguage: run.sourceLanguage,
              targetLanguage: run.targetLanguage,
              detectedLanguage: run.detectedLanguage,
              complete: run.complete,
              results: [
                ...previous.where((result) => result.service.id != service.id),
                ...run.results,
              ],
            );
            state = state.copyWith(run: merged, submitting: !run.complete);
            if (run.complete) {
              unawaited(_recordHistory(sessionId, run));
              unawaited(_refreshGlossary(merged));
            }
          },
          onError: (Object error) {
            if (requestId == _requestId) {
              state = state.copyWith(
                submitting: false,
                submissionErrorCode: 'translation_failed',
              );
            }
            if (!completion.isCompleted) completion.complete();
          },
          onDone: () {
            if (requestId == _requestId) {
              state = state.copyWith(submitting: false);
            }
            if (!completion.isCompleted) completion.complete();
          },
          cancelOnError: true,
        );
    await completion.future;
  }

  Future<void> _recordHistory(String sessionId, TranslationRun run) async {
    try {
      final saved = await ref.read(recordCompletedTranslationProvider)(
        sessionId: sessionId,
        run: run,
      );
      if (_activeHistorySessionId != sessionId) return;
      state = state.copyWith(
        historyByService: {
          ...state.historyByService,
          for (final entry in saved) entry.serviceId: entry,
        },
      );
    } catch (_) {
      // History persistence must never turn a successful translation into an
      // unhandled asynchronous error.
    }
  }

  Future<bool> toggleSelectedFavorite() async {
    final current = state.selectedHistoryRecord;
    if (current == null || state.updatingFavorite) return false;
    state = state.copyWith(updatingFavorite: true);
    try {
      final updated = await ref
          .read(historyRepositoryProvider)
          .setFavorite(entryId: current.id, favorite: !current.favorite);
      if (updated == null) return false;
      state = state.copyWith(
        historyByService: {
          ...state.historyByService,
          updated.serviceId: updated,
        },
      );
      return true;
    } catch (_) {
      return false;
    } finally {
      state = state.copyWith(updatingFavorite: false);
    }
  }

  Future<void> _refreshGlossary(TranslationRun run) async {
    final request = _requestId;
    final selectedId = state.selectedServiceId;
    try {
      final matches = await ref
          .read(glossaryRepositoryProvider)
          .matchText(
            text: run.sourceText,
            sourceLanguage: run.detectedLanguage ?? run.sourceLanguage,
            targetLanguage: run.targetLanguage,
          );
      if (request != _requestId || selectedId != state.selectedServiceId) {
        return;
      }
      final selected = state.selectedResult?.text ?? '';
      final warnings = selected.trim().isEmpty
          ? const <GlossaryComplianceWarning>[]
          : await ref
                .read(glossaryRepositoryProvider)
                .checkCompliance(
                  source: run.sourceText,
                  translated: selected,
                  sourceLanguage: run.detectedLanguage ?? run.sourceLanguage,
                  targetLanguage: run.targetLanguage,
                );
      if (request != _requestId || selectedId != state.selectedServiceId) {
        return;
      }
      state = state.copyWith(
        glossaryMatches: matches,
        glossaryWarnings: warnings,
      );
    } catch (_) {
      state = state.copyWith(
        glossaryMatches: const [],
        glossaryWarnings: const [],
      );
    }
  }
}

const Object _unset = Object();

final class TranslationViewState {
  const TranslationViewState({
    this.catalog,
    this.sourceText = '',
    this.sourceLanguage = autoLanguageCode,
    this.targetLanguage = automaticTargetCode,
    this.selectedServiceId,
    this.run,
    this.loadingCatalog = true,
    this.submitting = false,
    this.catalogErrorCode,
    this.submissionErrorCode,
    this.glossaryMatches = const [],
    this.glossaryWarnings = const [],
    this.historyByService = const {},
    this.updatingFavorite = false,
  });

  final TranslationCatalog? catalog;
  final String sourceText;
  final String sourceLanguage;
  final String targetLanguage;
  final String? selectedServiceId;
  final TranslationRun? run;
  final bool loadingCatalog;
  final bool submitting;
  final String? catalogErrorCode;
  final String? submissionErrorCode;
  final List<GlossaryMatchHit> glossaryMatches;
  final List<GlossaryComplianceWarning> glossaryWarnings;
  final Map<String, HistoryRecord> historyByService;
  final bool updatingFavorite;

  List<LanguageOption> get languages => catalog?.languages ?? const [];
  List<TranslationServiceOption> get services => catalog?.services ?? const [];

  ServiceTranslationResult? get selectedResult {
    final results = run?.results ?? const [];
    if (results.isEmpty) return null;
    for (final result in results) {
      if (result.service.id == selectedServiceId) return result;
    }
    return selectedServiceId == null ? results.first : null;
  }

  HistoryRecord? get selectedHistoryRecord {
    final serviceId = selectedResult?.service.id;
    return serviceId == null ? null : historyByService[serviceId];
  }

  TranslationViewState copyWith({
    Object? catalog = _unset,
    String? sourceText,
    String? sourceLanguage,
    String? targetLanguage,
    Object? selectedServiceId = _unset,
    Object? run = _unset,
    bool? loadingCatalog,
    bool? submitting,
    Object? catalogErrorCode = _unset,
    Object? submissionErrorCode = _unset,
    List<GlossaryMatchHit>? glossaryMatches,
    List<GlossaryComplianceWarning>? glossaryWarnings,
    Map<String, HistoryRecord>? historyByService,
    bool? updatingFavorite,
    bool clearRun = false,
    bool clearCatalogError = false,
    bool clearSubmissionError = false,
  }) {
    return TranslationViewState(
      catalog: identical(catalog, _unset)
          ? this.catalog
          : catalog as TranslationCatalog?,
      sourceText: sourceText ?? this.sourceText,
      sourceLanguage: sourceLanguage ?? this.sourceLanguage,
      targetLanguage: targetLanguage ?? this.targetLanguage,
      selectedServiceId: identical(selectedServiceId, _unset)
          ? this.selectedServiceId
          : selectedServiceId as String?,
      run: clearRun
          ? null
          : identical(run, _unset)
          ? this.run
          : run as TranslationRun?,
      loadingCatalog: loadingCatalog ?? this.loadingCatalog,
      submitting: submitting ?? this.submitting,
      catalogErrorCode: clearCatalogError
          ? null
          : identical(catalogErrorCode, _unset)
          ? this.catalogErrorCode
          : catalogErrorCode as String?,
      submissionErrorCode: clearSubmissionError
          ? null
          : identical(submissionErrorCode, _unset)
          ? this.submissionErrorCode
          : submissionErrorCode as String?,
      glossaryMatches: glossaryMatches ?? this.glossaryMatches,
      glossaryWarnings: glossaryWarnings ?? this.glossaryWarnings,
      historyByService: historyByService ?? this.historyByService,
      updatingFavorite: updatingFavorite ?? this.updatingFavorite,
    );
  }
}
