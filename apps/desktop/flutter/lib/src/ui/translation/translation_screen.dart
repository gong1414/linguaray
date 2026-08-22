import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import 'package:linguaray_application/linguaray_application.dart';

import '../../config/dependencies.dart';
import '../../i18n/i18n.dart';
import '../../services/app_windows.dart' show workbenchTextHandoff;
import '../i18n_labels.dart';
import 'view_models/translation_view_model.dart';
import 'widgets/dictionary_lookup_dialog.dart';
import 'widgets/translation_workspace_view.dart';

class TranslationScreen extends ConsumerStatefulWidget {
  const TranslationScreen({super.key});

  @override
  ConsumerState<TranslationScreen> createState() => _TranslationScreenState();
}

class _TranslationScreenState extends ConsumerState<TranslationScreen> {
  late final SpeechService _speechService;
  late final LookUpWord _lookUpWord;
  late final VocabularyRepository _vocabularyRepository;
  Timer? _copiedTimer;
  StreamSubscription<SpeechState>? _speechSubscription;
  String? _pendingHandoff;
  bool _copied = false;
  bool _speechAvailable = false;
  bool _dictionaryAvailable = false;
  bool _savingVocabulary = false;
  bool _vocabularySaved = false;
  SpeechUtteranceKind? _speakingKind;
  SpeechUtteranceKind? _requestedSpeechKind;

  @override
  void initState() {
    super.initState();
    workbenchTextHandoff.addListener(_handleHandoff);
    _speechService = ref.read(speechServiceProvider);
    _lookUpWord = ref.read(lookUpWordProvider);
    _vocabularyRepository = ref.read(vocabularyRepositoryProvider);
    _speechSubscription = _speechService.states.listen(_handleSpeechState);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _handleHandoff();
      unawaited(_loadInteractionCapabilities());
    });
  }

  @override
  void dispose() {
    workbenchTextHandoff.removeListener(_handleHandoff);
    _copiedTimer?.cancel();
    unawaited(_speechSubscription?.cancel());
    unawaited(_speechService.stop());
    super.dispose();
  }

  Future<void> _loadInteractionCapabilities() async {
    final speech = await _speechService.isAvailable();
    bool dictionary;
    try {
      dictionary = await _lookUpWord.isAvailable;
    } catch (_) {
      dictionary = false;
    }
    if (!mounted) return;
    setState(() {
      _speechAvailable = speech;
      _dictionaryAvailable = dictionary;
    });
  }

  void _handleSpeechState(SpeechState state) {
    if (!mounted) return;
    setState(() {
      _speakingKind = state.isSpeaking ? _requestedSpeechKind : null;
      if (!state.isSpeaking) _requestedSpeechKind = null;
    });
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

  Future<void> _speak({
    required SpeechUtteranceKind kind,
    required String text,
    required String? language,
  }) async {
    if (text.trim().isEmpty) return;
    _requestedSpeechKind = kind;
    if (mounted) setState(() => _speakingKind = kind);
    final result = await _speechService.speak(
      text: text,
      kind: kind,
      language: _speechLanguage(language),
    );
    if (!mounted || result.isSpeaking) return;
    setState(() {
      _speakingKind = null;
      _requestedSpeechKind = null;
      if (result.status == SpeechStatus.unavailable) {
        _speechAvailable = false;
      }
    });
    if (result.status == SpeechStatus.failed ||
        result.status == SpeechStatus.unavailable) {
      await _showMessage(appErrorMessage(result.errorCode));
    }
  }

  Future<void> _stopSpeech() async {
    _requestedSpeechKind = null;
    await _speechService.stop();
    if (mounted) setState(() => _speakingKind = null);
  }

  Future<void> _lookup(String word) async {
    final state = ref.read(translationViewModelProvider);
    final run = state.run;
    if (run == null || word.trim().isEmpty) return;
    final sourceLanguage = run.targetLanguage;
    final targetLanguage = run.detectedLanguage ?? run.sourceLanguage;
    await showDialog<void>(
      context: context,
      builder: (context) => DictionaryLookupDialog(
        labels: DictionaryLookupDialogLabels(
          title: t.ui.dictionary.title,
          pronunciation: t.ui.dictionary.pronunciation,
          definitions: t.ui.dictionary.definitions,
          save: t.ui.dictionary.save,
          saved: t.ui.vocabulary.saved,
          close: t.ui.shell.close,
          empty: t.ui.dictionary.empty,
          lookupFailed: appErrorMessage(
            AppErrorCode.dictionaryUnavailable.wireName,
          ),
          saveFailed: appErrorMessage(
            AppErrorCode.vocabularyUnavailable.wireName,
          ),
        ),
        lookup: _lookUpWord(
          DictionaryLookupQuery(
            word: word,
            sourceLanguage: sourceLanguage,
            targetLanguage: targetLanguage,
          ),
        ),
        onSave: (entry) => _vocabularyRepository.upsert(
          VocabularyDraft(
            word: entry.word,
            translation: dictionaryVocabularyTranslation(entry),
            sourceLanguage: sourceLanguage,
            targetLanguage: targetLanguage,
            source: 'dictionary',
          ),
        ),
      ),
    );
  }

  Future<void> _saveTranslationVocabulary() async {
    final state = ref.read(translationViewModelProvider);
    final run = state.run;
    final result = state.selectedResult;
    if (run == null || result == null || !result.hasText) return;
    setState(() => _savingVocabulary = true);
    try {
      await _vocabularyRepository.upsert(
        VocabularyDraft(
          word: run.sourceText,
          translation: result.text,
          sourceLanguage: run.detectedLanguage ?? run.sourceLanguage,
          targetLanguage: run.targetLanguage,
          source: 'translation',
        ),
      );
      if (!mounted) return;
      setState(() {
        _savingVocabulary = false;
        _vocabularySaved = true;
      });
    } catch (_) {
      if (!mounted) return;
      setState(() => _savingVocabulary = false);
      await _showMessage(
        appErrorMessage(AppErrorCode.vocabularyUnavailable.wireName),
      );
    }
  }

  Future<void> _showMessage(String message) {
    if (!mounted) return Future.value();
    return showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        content: Text(message),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(t.common.ui.button.ok),
          ),
        ],
      ),
    );
  }

  void _markVocabularyUnsaved() {
    if (_vocabularySaved) setState(() => _vocabularySaved = false);
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
      glossaryMatches: state.glossaryMatches,
      glossaryWarnings: state.glossaryWarnings,
      dictionaryAvailable: _dictionaryAvailable,
      speechAvailable: _speechAvailable,
      speakingKind: _speakingKind,
      vocabularySaved: _vocabularySaved,
      savingVocabulary: _savingVocabulary,
      onSourceTextChanged: (value) {
        _markVocabularyUnsaved();
        ref.read(translationViewModelProvider.notifier).setSourceText(value);
      },
      onSourceLanguageChanged: (value) {
        _markVocabularyUnsaved();
        ref
            .read(translationViewModelProvider.notifier)
            .setSourceLanguage(value);
      },
      onTargetLanguageChanged: (value) {
        _markVocabularyUnsaved();
        ref
            .read(translationViewModelProvider.notifier)
            .setTargetLanguage(value);
      },
      onServiceSelected: (value) {
        _markVocabularyUnsaved();
        ref.read(translationViewModelProvider.notifier).selectService(value);
      },
      onSwapLanguages: ref
          .read(translationViewModelProvider.notifier)
          .swapLanguages,
      onTranslate: () {
        _markVocabularyUnsaved();
        unawaited(ref.read(translationViewModelProvider.notifier).submit());
      },
      onClear: () {
        _markVocabularyUnsaved();
        ref.read(translationViewModelProvider.notifier).clearSourceText();
      },
      onCopy: (value) => unawaited(_copy(value)),
      onSpeakSource: () => unawaited(
        _speak(
          kind: SpeechUtteranceKind.source,
          text: state.sourceText,
          language: state.run?.detectedLanguage ?? state.sourceLanguage,
        ),
      ),
      onSpeakResult: () => unawaited(
        _speak(
          kind: SpeechUtteranceKind.translation,
          text: state.selectedResult?.text ?? '',
          language: state.run?.targetLanguage ?? state.targetLanguage,
        ),
      ),
      onStopSpeech: () => unawaited(_stopSpeech()),
      onLookup: (word) => unawaited(_lookup(word)),
      onSaveVocabulary: () => unawaited(_saveTranslationVocabulary()),
      onConfigureServices: () => context.go('/settings/services'),
      onRecovery: (action) {
        switch (action) {
          case RecoveryAction.configureTranslationProvider:
            context.go('/settings/providers');
          case RecoveryAction.configureOcr:
            context.go('/settings/services');
          case RecoveryAction.openPermissionSettings:
          case RecoveryAction.recheckPermission:
            context.go('/settings/permissions');
          case RecoveryAction.switchToGoogleWeb:
            unawaited(_enableGoogleWeb());
          default:
            break;
        }
      },
    );
  }

  Future<void> _enableGoogleWeb() async {
    final repository = ref.read(workspaceSettingsRepositoryProvider);
    final providers = await repository.listProviders();
    final hasGoogleWeb = providers.any(
      (provider) =>
          provider.id == 'google-web' || provider.typeId == 'google_web',
    );
    if (!hasGoogleWeb) {
      await repository.saveProvider(
        const ProviderDraft(
          id: 'google-web',
          typeId: 'google_web',
          presetId: 'google-web',
          fields: {'baseUrl': 'https://translate.google.com'},
        ),
      );
    }
    await repository.setServiceEnabled(
      serviceId: 'google-web+translation',
      enabled: true,
    );
    final viewModel = ref.read(translationViewModelProvider.notifier);
    await viewModel.initialize();
    viewModel.selectService('google-web+translation');
  }
}

String? _speechLanguage(String? language) {
  return switch (language) {
    null || '' || 'auto' || 'automatic' => null,
    'zh-Hans' => 'zh-CN',
    'zh-Hant' => 'zh-TW',
    _ => language,
  };
}
