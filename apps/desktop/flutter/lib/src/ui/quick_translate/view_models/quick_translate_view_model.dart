import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../config/dependencies.dart';
import '../../translation/view_models/translation_view_model.dart';

final quickTranslateViewModelProvider =
    NotifierProvider<QuickTranslateViewModel, TranslationViewState>(
      QuickTranslateViewModel.new,
    );

final class QuickTranslateViewModel extends Notifier<TranslationViewState> {
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
      state = state.copyWith(
        catalog: catalog,
        sourceLanguage: catalog.defaultSourceLanguage,
        targetLanguage: automaticTargetCode,
        selectedServiceId: catalog.services.isEmpty
            ? null
            : catalog.services.first.id,
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
        state = state.copyWith(
          run: run,
          selectedServiceId: _selectedServiceFor(run),
          submitting: !run.complete,
        );
      }
      if (requestId == _requestId) {
        state = state.copyWith(submitting: false);
      }
    } catch (_) {
      if (requestId != _requestId) return;
      state = state.copyWith(
        submitting: false,
        submissionErrorCode: 'translation_failed',
      );
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
}
