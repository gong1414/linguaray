import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../services/app_windows.dart' show workbenchTextHandoff;
import '../i18n_labels.dart';
import 'view_models/translation_view_model.dart';
import 'widgets/translation_workspace_view.dart';

class TranslationScreen extends ConsumerStatefulWidget {
  const TranslationScreen({super.key});

  @override
  ConsumerState<TranslationScreen> createState() => _TranslationScreenState();
}

class _TranslationScreenState extends ConsumerState<TranslationScreen> {
  Timer? _copiedTimer;
  String? _pendingHandoff;
  bool _copied = false;

  @override
  void initState() {
    super.initState();
    workbenchTextHandoff.addListener(_handleHandoff);
    WidgetsBinding.instance.addPostFrameCallback((_) => _handleHandoff());
  }

  @override
  void dispose() {
    workbenchTextHandoff.removeListener(_handleHandoff);
    _copiedTimer?.cancel();
    super.dispose();
  }

  void _handleHandoff() {
    final value = workbenchTextHandoff.value;
    if (value == null || value.trim().isEmpty) return;
    workbenchTextHandoff.value = null;
    ref.read(translationViewModelProvider.notifier).setSourceText(value);
    if (ref.read(translationViewModelProvider).catalog == null) {
      _pendingHandoff = value;
      return;
    }
    unawaited(ref.read(translationViewModelProvider.notifier).submit());
  }

  Future<void> _copy(String value) async {
    if (value.trim().isEmpty) return;
    await Clipboard.setData(ClipboardData(text: value));
    if (!mounted) return;
    setState(() => _copied = true);
    _copiedTimer?.cancel();
    _copiedTimer = Timer(const Duration(milliseconds: 1400), () {
      if (mounted) setState(() => _copied = false);
    });
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(translationViewModelProvider);
    ref.listen(translationViewModelProvider, (previous, next) {
      if (_pendingHandoff == null || next.catalog == null) return;
      _pendingHandoff = null;
      unawaited(ref.read(translationViewModelProvider.notifier).submit());
    });

    return TranslationWorkspaceView(
      labels: translationWorkspaceLabels(),
      languages: state.languages,
      services: state.services,
      sourceText: state.sourceText,
      sourceLanguage: state.sourceLanguage,
      targetLanguage: state.targetLanguage,
      selectedServiceId: state.selectedServiceId,
      selectedResult: state.selectedResult,
      results: state.run?.results ?? const [],
      detectedLanguage: state.run?.detectedLanguage,
      resolvedTargetLanguage: state.run?.targetLanguage,
      loadingCatalog: state.loadingCatalog,
      submitting: state.submitting,
      catalogFailed: state.catalogErrorCode != null,
      submissionFailed: state.submissionErrorCode != null,
      copied: _copied,
      showHeader: false,
      onSourceTextChanged: ref
          .read(translationViewModelProvider.notifier)
          .setSourceText,
      onSourceLanguageChanged: ref
          .read(translationViewModelProvider.notifier)
          .setSourceLanguage,
      onTargetLanguageChanged: ref
          .read(translationViewModelProvider.notifier)
          .setTargetLanguage,
      onServiceSelected: ref
          .read(translationViewModelProvider.notifier)
          .selectService,
      onSwapLanguages: ref
          .read(translationViewModelProvider.notifier)
          .swapLanguages,
      onTranslate: () =>
          unawaited(ref.read(translationViewModelProvider.notifier).submit()),
      onClear: ref.read(translationViewModelProvider.notifier).clearSourceText,
      onCopy: (value) => unawaited(_copy(value)),
      onConfigureServices: () => context.go('/settings/services'),
    );
  }
}
