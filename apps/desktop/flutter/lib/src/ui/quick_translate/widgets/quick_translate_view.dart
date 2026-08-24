import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../shared/status_message.dart';

import 'quick_translate_components.dart';
import 'quick_translate_input.dart';
import 'quick_translate_models.dart';
import 'quick_translate_result_panel.dart';

export 'quick_translate_models.dart';

class QuickTranslateView extends StatefulWidget {
  const QuickTranslateView({
    required this.labels,
    required this.languages,
    required this.services,
    required this.sourceText,
    required this.sourceLanguage,
    required this.targetLanguage,
    required this.selectedServiceId,
    required this.onSourceTextChanged,
    required this.onSourceLanguageChanged,
    required this.onTargetLanguageChanged,
    required this.onServiceSelected,
    required this.onSwapLanguages,
    required this.onTranslate,
    required this.onClear,
    required this.onCopy,
    required this.onTogglePin,
    required this.onCapture,
    required this.onClipboard,
    required this.onOpenSettings,
    required this.onConfigureServices,
    required this.onRecheckPermissions,
    super.key,
    this.results = const [],
    this.selectedResult,
    this.detectedLanguage,
    this.submitting = false,
    this.copied = false,
    this.pinned = false,
    this.notice = QuickTranslateNotice.none,
    this.toolbarKey,
    this.contentKey,
    this.onConfigureOcr,
    this.submitWithModifier = false,
    this.copyResultOnDoubleClick = false,
    this.glossaryMatches = const [],
    this.glossaryWarnings = const [],
    this.speechAvailable = false,
    this.dictionaryAvailable = false,
    this.speakingKind,
    this.savingVocabulary = false,
    this.vocabularySaved = false,
    this.favoriteAvailable = false,
    this.favorite = false,
    this.updatingFavorite = false,
    this.onSpeakSource,
    this.onSpeakResult,
    this.onStopSpeech,
    this.onLookup,
    this.onSaveVocabulary,
    this.onToggleFavorite,
  });

  final QuickTranslateLabels labels;
  final List<LanguageOption> languages;
  final List<TranslationServiceOption> services;
  final String sourceText;
  final String sourceLanguage;
  final String targetLanguage;
  final String? selectedServiceId;
  final List<ServiceTranslationResult> results;
  final ServiceTranslationResult? selectedResult;
  final String? detectedLanguage;
  final bool submitting;
  final bool copied;
  final bool pinned;
  final QuickTranslateNotice notice;
  final GlobalKey? toolbarKey;
  final GlobalKey? contentKey;
  final ValueChanged<String> onSourceTextChanged;
  final ValueChanged<String> onSourceLanguageChanged;
  final ValueChanged<String> onTargetLanguageChanged;
  final ValueChanged<String> onServiceSelected;
  final VoidCallback onSwapLanguages;
  final VoidCallback onTranslate;
  final VoidCallback onClear;
  final ValueChanged<String> onCopy;
  final VoidCallback onTogglePin;
  final VoidCallback onCapture;
  final VoidCallback onClipboard;
  final VoidCallback onOpenSettings;
  final VoidCallback onConfigureServices;
  final VoidCallback onRecheckPermissions;
  final VoidCallback? onConfigureOcr;
  final bool submitWithModifier;
  final bool copyResultOnDoubleClick;
  final List<GlossaryMatchHit> glossaryMatches;
  final List<GlossaryComplianceWarning> glossaryWarnings;
  final bool speechAvailable;
  final bool dictionaryAvailable;
  final SpeechUtteranceKind? speakingKind;
  final bool savingVocabulary;
  final bool vocabularySaved;
  final bool favoriteAvailable;
  final bool favorite;
  final bool updatingFavorite;
  final VoidCallback? onSpeakSource;
  final VoidCallback? onSpeakResult;
  final VoidCallback? onStopSpeech;
  final ValueChanged<String>? onLookup;
  final VoidCallback? onSaveVocabulary;
  final VoidCallback? onToggleFavorite;

  @override
  State<QuickTranslateView> createState() => _QuickTranslateViewState();
}

class _QuickTranslateViewState extends State<QuickTranslateView> {
  late final TextEditingController _controller = TextEditingController(
    text: widget.sourceText,
  );

  @override
  void didUpdateWidget(covariant QuickTranslateView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.sourceText != _controller.text) {
      _controller
        ..text = widget.sourceText
        ..selection = TextSelection.collapsed(offset: widget.sourceText.length);
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final canTranslate =
        widget.sourceText.trim().isNotEmpty &&
        widget.services.isNotEmpty &&
        !widget.submitting;

    final submitBindings = !widget.submitWithModifier
        ? <ShortcutActivator, VoidCallback>{
            const SingleActivator(LogicalKeyboardKey.enter): () {
              if (canTranslate) widget.onTranslate();
            },
          }
        : <ShortcutActivator, VoidCallback>{
            const SingleActivator(LogicalKeyboardKey.enter, meta: true): () {
              if (canTranslate) widget.onTranslate();
            },
            const SingleActivator(LogicalKeyboardKey.enter, control: true): () {
              if (canTranslate) widget.onTranslate();
            },
          };

    return CallbackShortcuts(
      bindings: submitBindings,
      child: Material(
        color: theme.colorScheme.surface,
        child: Padding(
          padding: const EdgeInsets.fromLTRB(12, 8, 12, 12),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              KeyedSubtree(
                key: widget.toolbarKey,
                child: QuickTranslateCommandHeader(
                  labels: widget.labels,
                  pinned: widget.pinned,
                  languages: widget.languages,
                  sourceLanguage: widget.sourceLanguage,
                  targetLanguage: widget.targetLanguage,
                  onTogglePin: widget.onTogglePin,
                  onCapture: widget.onCapture,
                  onClipboard: widget.onClipboard,
                  onOpenSettings: widget.onOpenSettings,
                  onSourceLanguageChanged: widget.onSourceLanguageChanged,
                  onTargetLanguageChanged: widget.onTargetLanguageChanged,
                  onSwapLanguages: widget.onSwapLanguages,
                ),
              ),
              KeyedSubtree(
                key: widget.contentKey,
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    const SizedBox(height: 8),
                    QuickTranslateInput(
                      labels: widget.labels,
                      controller: _controller,
                      sourceText: widget.sourceText,
                      submitting: widget.submitting,
                      speechAvailable: widget.speechAvailable,
                      submitWithModifier: widget.submitWithModifier,
                      canTranslate: canTranslate,
                      speakingKind: widget.speakingKind,
                      onSourceTextChanged: widget.onSourceTextChanged,
                      onClear: widget.onClear,
                      onTranslate: widget.onTranslate,
                      onSpeakSource: widget.onSpeakSource,
                      onStopSpeech: widget.onStopSpeech,
                    ),
                    if (widget.notice != QuickTranslateNotice.none)
                      Padding(
                        padding: const EdgeInsets.only(top: 8),
                        child: QuickTranslateNoticeMessage(
                          labels: widget.labels,
                          notice: widget.notice,
                          onRecheck: widget.onRecheckPermissions,
                          onConfigureOcr: widget.onConfigureOcr,
                          onConfigureServices: widget.onConfigureServices,
                          onRetryCapture: widget.onCapture,
                        ),
                      ),
                    if (widget.services.isEmpty)
                      Padding(
                        padding: const EdgeInsets.only(top: 8),
                        child: StatusMessage(
                          kind: StatusKind.warning,
                          title: widget.labels.noServices,
                          action: OutlinedButton(
                            onPressed: widget.onConfigureServices,
                            child: Text(widget.labels.configureServices),
                          ),
                        ),
                      ),
                    if (widget.submitting)
                      const Padding(
                        padding: EdgeInsets.only(top: 8),
                        child: LinearProgressIndicator(minHeight: 2),
                      ),
                    QuickTranslateResultPanel(
                      labels: widget.labels,
                      results: widget.results,
                      selectedResult: widget.selectedResult,
                      selectedServiceId: widget.selectedServiceId,
                      glossaryMatches: widget.glossaryMatches,
                      glossaryWarnings: widget.glossaryWarnings,
                      dictionaryAvailable: widget.dictionaryAvailable,
                      speechAvailable: widget.speechAvailable,
                      speakingKind: widget.speakingKind,
                      savingVocabulary: widget.savingVocabulary,
                      vocabularySaved: widget.vocabularySaved,
                      favoriteAvailable: widget.favoriteAvailable,
                      favorite: widget.favorite,
                      updatingFavorite: widget.updatingFavorite,
                      copied: widget.copied,
                      copyResultOnDoubleClick: widget.copyResultOnDoubleClick,
                      onTranslate: widget.onTranslate,
                      onCopy: widget.onCopy,
                      onServiceSelected: widget.onServiceSelected,
                      onLookup: widget.onLookup,
                      onSaveVocabulary: widget.onSaveVocabulary,
                      onToggleFavorite: widget.onToggleFavorite,
                      onSpeakResult: widget.onSpeakResult,
                      onStopSpeech: widget.onStopSpeech,
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
