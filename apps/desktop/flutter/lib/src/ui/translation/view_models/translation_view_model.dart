import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../config/dependencies.dart';
import 'translation_session_id.dart';

final translationViewModelProvider =
    NotifierProvider<TranslationViewModel, TranslationViewState>(
      TranslationViewModel.new,
    );

final class TranslationViewModel extends Notifier<TranslationViewState> {
  int _requestId = 0;

  @override
  TranslationViewState build() {
    ref.onDispose(() => _requestId++);
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
    state = state.copyWith(sourceText: value);
  }

  void clearSourceText() {
    _requestId++;
    state = state.copyWith(
      sourceText: '',
      submitting: false,
      clearRun: true,
      clearSubmissionError: true,
    );
  }

  void setSourceLanguage(String value) {
    state = state.copyWith(sourceLanguage: value);
  }

  void setTargetLanguage(String value) {
    state = state.copyWith(targetLanguage: value);
  }

  void selectService(String serviceId) {
    state = state.copyWith(selectedServiceId: serviceId);
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

  Future<void> submit() async {
    final catalog = state.catalog;
    final text = state.sourceText.trim();
    if (catalog == null || text.isEmpty || state.loadingCatalog) return;

    final requestId = ++_requestId;
    final sessionId = newTranslationSessionId();
    state = state.copyWith(
      submitting: true,
      clearRun: true,
      clearSubmissionError: true,
    );

    try {
      await for (final run in ref.read(translateTextProvider)(
        query: TranslationQuery(
          text: text,
          sourceLanguage: state.sourceLanguage,
          targetLanguage: state.targetLanguage == automaticTargetCode
              ? null
              : state.targetLanguage,
        ),
        catalog: catalog,
      )) {
        if (requestId != _requestId) return;
        final selectedId = _selectedServiceFor(run);
        state = state.copyWith(
          run: run,
          selectedServiceId: selectedId,
          submitting: !run.complete,
        );
      }
      if (requestId == _requestId) {
        final run = state.run;
        state = state.copyWith(submitting: false);
        if (run != null && run.complete) {
          unawaited(_recordHistory(sessionId, run));
          unawaited(_refreshGlossary(run));
        }
      }
    } catch (_) {
      if (requestId != _requestId) return;
      state = state.copyWith(
        submitting: false,
        submissionErrorCode: 'translation_failed',
      );
    }
  }

  Future<void> _recordHistory(String sessionId, TranslationRun run) async {
    try {
      await ref.read(recordCompletedTranslationProvider)(
        sessionId: sessionId,
        run: run,
      );
    } catch (_) {
      // History persistence must never turn a successful translation into an
      // unhandled asynchronous error.
    }
  }

  String? _selectedServiceFor(TranslationRun run) {
    final standing = state.selectedServiceId;
    if (standing != null &&
        run.results.any((result) => result.service.id == standing)) {
      return standing;
    }
    for (final result in run.results) {
      if (result.hasText) return result.service.id;
    }
    return run.results.isEmpty ? null : run.results.first.service.id;
  }

  Future<void> _refreshGlossary(TranslationRun run) async {
    try {
      final matches = await ref
          .read(glossaryRepositoryProvider)
          .matchText(
            text: run.sourceText,
            sourceLanguage: run.detectedLanguage ?? run.sourceLanguage,
            targetLanguage: run.targetLanguage,
          );
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

  List<LanguageOption> get languages => catalog?.languages ?? const [];
  List<TranslationServiceOption> get services => catalog?.services ?? const [];

  ServiceTranslationResult? get selectedResult {
    final results = run?.results ?? const [];
    if (results.isEmpty) return null;
    for (final result in results) {
      if (result.service.id == selectedServiceId) return result;
    }
    return results.first;
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
    );
  }
}
