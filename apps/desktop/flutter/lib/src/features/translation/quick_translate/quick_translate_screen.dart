import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:nativeapi/nativeapi.dart' as nativeapi;

import '../../../app/commands/trigger_controller.dart';
import '../../../app/dependencies.dart';
import '../../../app/settings/settings_store.dart';
import '../../../app/windows/app_windows.dart';
import '../../../i18n/i18n.dart';
import '../../../platform/permissions/permission_controller.dart';
import '../../../platform/platform_types.dart';
import '../../../platform/selection/selection_replacement_controller.dart';
import '../../../shared/i18n_labels.dart';
import '../view_models/translation_view_model.dart';
import '../widgets/dictionary_lookup_dialog.dart';
import 'quick_translate_window_coordinator.dart';
import 'widgets/quick_translate_view.dart';

class QuickTranslateScreen extends ConsumerStatefulWidget {
  const QuickTranslateScreen({super.key});

  @override
  ConsumerState<QuickTranslateScreen> createState() =>
      _QuickTranslateScreenState();
}

class _QuickTranslateScreenState extends ConsumerState<QuickTranslateScreen>
    with WidgetsBindingObserver {
  bool _pinned = false;
  SelectionTarget? _replacementTarget;
  bool _replacing = false;
  bool _copied = false;
  bool _speechAvailable = false;
  bool _dictionaryAvailable = false;
  bool _savingVocabulary = false;
  bool _vocabularySaved = false;
  SpeechUtteranceKind? _speakingKind;
  SpeechUtteranceKind? _requestedSpeechKind;
  QuickTranslateNotice _notice = QuickTranslateNotice.none;
  Timer? _copiedTimer;
  StreamSubscription<SpeechState>? _speechSubscription;
  late final SpeechService _speechService;
  late final LookUpWord _lookUpWord;
  late final VocabularyRepository _vocabularyRepository;
  late final QuickTranslateWindowCoordinator _windowCoordinator;

  nativeapi.Window get _window => _windowCoordinator.window;

  @override
  void initState() {
    super.initState();
    _speechService = ref.read(speechServiceProvider);
    _lookUpWord = ref.read(lookUpWordProvider);
    _vocabularyRepository = ref.read(vocabularyRepositoryProvider);
    _windowCoordinator = QuickTranslateWindowCoordinator(
      () => miniTranslatorWindowController.window,
      () => mounted,
      onDismiss: () => ref.read(translationViewModelProvider.notifier).cancel(),
    );
    _speechSubscription = _speechService.states.listen(_handleSpeechState);
    WidgetsBinding.instance.addObserver(this);
    triggerController.quickWindowRequest.addListener(_consumeWindowRequest);
    triggerController.lastError.addListener(_showTriggerError);
    permissionController.addListener(_onPermissionChanged);
    _windowCoordinator.registerEvents();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _consumeWindowRequest();
      _scheduleWindowResize();
      unawaited(_loadInteractionCapabilities());
    });
  }

  @override
  void dispose() {
    triggerController.quickWindowRequest.removeListener(_consumeWindowRequest);
    triggerController.lastError.removeListener(_showTriggerError);
    permissionController.removeListener(_onPermissionChanged);
    WidgetsBinding.instance.removeObserver(this);
    _windowCoordinator.dispose();
    _copiedTimer?.cancel();
    unawaited(_speechSubscription?.cancel());
    unawaited(_speechService.stop());
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      unawaited(permissionController.refresh());
    }
  }

  void _consumeWindowRequest() {
    final request = triggerController.quickWindowRequest.value;
    if (request == null) return;
    triggerController.quickWindowRequest.value = null;
    final viewModel = ref.read(translationViewModelProvider.notifier);
    if (request.clearExisting) {
      _replacementTarget = request.replacementTarget;
      viewModel.clearSourceText();
    }
    final text = request.text;
    if (text != null) viewModel.setSourceText(text);
    if (request.submit && text != null && text.trim().isNotEmpty) {
      unawaited(viewModel.submit());
    }
    _scheduleWindowResize();
  }

  void _showTriggerError() {
    final error = triggerController.lastError.value;
    if (error == null) return;
    setState(() {
      _notice = switch (error.code) {
        'cancelled' ||
        'capture_cancelled' => QuickTranslateNotice.captureCancelled,
        'permission_denied' ||
        'accessibility_denied' ||
        'accessibilityDenied' ||
        'screen_recording_denied' => QuickTranslateNotice.permissionDenied,
        'capture_failed' ||
        'captureFailed' => QuickTranslateNotice.captureFailed,
        'ocr_not_configured' ||
        'ocrNotConfigured' => QuickTranslateNotice.ocrNotConfigured,
        'ocr_empty' || 'ocrEmpty' => QuickTranslateNotice.ocrEmpty,
        'empty_selection' => QuickTranslateNotice.emptySelection,
        'clipboard_unavailable' => QuickTranslateNotice.clipboardUnavailable,
        'clipboard_restore_failed' =>
          QuickTranslateNotice.clipboardRestoreFailed,
        _ => QuickTranslateNotice.none,
      };
    });
    _scheduleWindowResize();
  }

  void _onPermissionChanged() {
    final snapshot = permissionController.snapshot;
    final denied =
        snapshot.accessibility == PermissionState.denied ||
        snapshot.screenRecording == PermissionState.denied;
    if (!denied && _notice == QuickTranslateNotice.permissionDenied) {
      setState(() => _notice = QuickTranslateNotice.none);
    }
  }

  void _scheduleWindowResize() => _windowCoordinator.scheduleResize();

  Future<void> _copy(String value) async {
    await Clipboard.setData(ClipboardData(text: value));
    if (!mounted) return;
    setState(() => _copied = true);
    _copiedTimer?.cancel();
    _copiedTimer = Timer(const Duration(milliseconds: 1400), () {
      if (mounted) setState(() => _copied = false);
    });
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

  Future<void> _speak({
    required SpeechUtteranceKind kind,
    required String text,
    required String? language,
  }) async {
    if (text.trim().isEmpty) return;
    _requestedSpeechKind = kind;
    setState(() => _speakingKind = kind);
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
    if (canResizeMiniTranslatorWindow) {
      _window.setSize(_window.size.width, 620, animate: true);
    }
    try {
      final sourceLanguage = run.targetLanguage;
      final targetLanguage = run.detectedLanguage ?? run.sourceLanguage;
      await showDialog<void>(
        context: context,
        builder: (context) => DictionaryLookupDialog(
          labels: DictionaryLookupDialogLabels(
            title: t.ui.dictionary.title,
            pronunciation: t.ui.dictionary.pronunciation,
            speak: t.ui.speech.speak_source,
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
          onSpeak: (entry) => _speak(
            kind: SpeechUtteranceKind.source,
            text: entry.word,
            language: sourceLanguage,
          ),
        ),
      );
    } finally {
      _scheduleWindowResize();
    }
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
    final languages = orderLanguagesByPreference(
      state.languages,
      settingsStore.general.commonLanguages,
    );
    ref.listen(translationViewModelProvider, (previous, next) {
      _scheduleWindowResize();
    });

    return QuickTranslateView(
      labels: quickTranslateLabels(),
      onLayoutChanged: _scheduleWindowResize,
      onStartDragging: _windowCoordinator.startDragging,
      onClose: () {
        ref.read(translationViewModelProvider.notifier).cancel();
        hideMiniTranslatorWindow();
      },
      onStop: ref.read(translationViewModelProvider.notifier).cancel,
      onToggleReading: () {
        _window.setContentSize(
          _window.contentSize.width >= 600 ? 460 : 760,
          _window.contentSize.height,
        );
        _scheduleWindowResize();
      },
      onReplace:
          !_replacing &&
              _replacementTarget != null &&
              state.run?.sourceText == _replacementTarget!.text.trim() &&
              state.sourceText.trim() == _replacementTarget!.text.trim() &&
              state.selectedResult?.status == TranslationResultStatus.completed
          ? () => unawaited(_replaceSelection(state.selectedResult!.text))
          : null,

      toolbarKey: _windowCoordinator.toolbarKey,
      contentKey: _windowCoordinator.contentKey,
      languages: languages,
      services: state.services,
      sourceText: state.sourceText,
      sourceLanguage: state.sourceLanguage,
      targetLanguage: state.targetLanguage,
      selectedServiceId: state.selectedServiceId,
      selectedResult: state.selectedResult,
      results: state.run?.results ?? const [],
      detectedLanguage: state.run?.detectedLanguage,
      submitting: state.submitting,
      copied: _copied,
      pinned: _pinned,
      notice: _notice,
      submitWithModifier: settingsStore.inputSubmitMode.name == 'commandEnter',
      copyResultOnDoubleClick: settingsStore.doubleClickCopyResult,
      glossaryMatches: state.glossaryMatches,
      glossaryWarnings: state.glossaryWarnings,
      speechAvailable: _speechAvailable,
      dictionaryAvailable: _dictionaryAvailable,
      speakingKind: _speakingKind,
      savingVocabulary: _savingVocabulary,
      vocabularySaved: _vocabularySaved,
      favoriteAvailable: state.selectedHistoryRecord != null,
      favorite: state.selectedHistoryRecord?.favorite ?? false,
      updatingFavorite: state.updatingFavorite,
      onSourceTextChanged: (value) {
        _markVocabularyUnsaved();
        ref.read(translationViewModelProvider.notifier).setSourceText(value);
        _scheduleWindowResize();
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
        _scheduleWindowResize();
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
      onToggleFavorite: () => unawaited(_toggleFavorite()),
      onTogglePin: () {
        setState(() => _pinned = !_pinned);
        _window.isAlwaysOnTop = _pinned;
      },
      onCapture: () => unawaited(
        triggerController.trigger(TriggerAction.captureAndTranslate),
      ),
      onClipboard: () =>
          unawaited(triggerController.trigger(TriggerAction.translateInput)),
      onOpenSettings: showSettingsWindow,
      onConfigureServices: showSettingsWindow,
      onRecheckPermissions: () => unawaited(permissionController.refresh()),
    );
  }

  Future<void> _replaceSelection(String text) async {
    final target = _replacementTarget;
    if (target == null || _replacing) return;
    setState(() => _replacing = true);
    await permissionController.refresh();
    final result = await selectionReplacementController.replace(target, text);
    if (!mounted) return;
    setState(() => _replacing = false);
    if (result == SelectionReplacementResult.replaced) {
      _replacementTarget = null;
      hideMiniTranslatorWindow();
    } else {
      await _showMessage(switch (result) {
        SelectionReplacementResult.changed => t.ui.quick.replace_changed,
        SelectionReplacementResult.denied => t.ui.quick.permission_denied,
        _ => t.ui.quick.replace_unsupported,
      });
    }
  }

  Future<void> _toggleFavorite() async {
    final updated = await ref
        .read(translationViewModelProvider.notifier)
        .toggleSelectedFavorite();
    if (!updated && mounted) {
      await _showMessage(t.workbench.translation.favorite_unavailable);
    }
  }
}

List<LanguageOption> orderLanguagesByPreference(
  List<LanguageOption> languages,
  List<String> preferredCodes,
) {
  if (languages.length < 2 || preferredCodes.isEmpty) return languages;
  final priority = <String, int>{
    for (var index = 0; index < preferredCodes.length; index++)
      preferredCodes[index]: index,
  };
  final indexed = languages.indexed.toList();
  indexed.sort((left, right) {
    final leftPriority = priority[left.$2.code];
    final rightPriority = priority[right.$2.code];
    if (leftPriority != null && rightPriority != null) {
      return leftPriority.compareTo(rightPriority);
    }
    if (leftPriority != null) return -1;
    if (rightPriority != null) return 1;
    return left.$1.compareTo(right.$1);
  });
  return [for (final entry in indexed) entry.$2];
}

String? _speechLanguage(String? language) {
  return switch (language) {
    null || '' || 'auto' || 'automatic' => null,
    'zh-Hans' => 'zh-CN',
    'zh-Hant' => 'zh-TW',
    _ => language,
  };
}
