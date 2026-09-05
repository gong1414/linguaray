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
    this.onClose,
    this.onStartDragging,
    this.onStop,
    this.onReplace,
    this.onLayoutChanged,
    this.onToggleReading,
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

  final VoidCallback? onClose;
  final VoidCallback? onStartDragging;
  final VoidCallback? onStop;
  final VoidCallback? onReplace;
  final VoidCallback? onLayoutChanged;
  final VoidCallback? onToggleReading;
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
  bool _sourceVisible = true;
  double _fontScale = 1;

  void _toggleSource() {
    setState(() => _sourceVisible = !_sourceVisible);
    widget.onLayoutChanged?.call();
  }

  void _scaleFont(double value) {
    setState(() => _fontScale = value.clamp(.85, 1.5));
    widget.onLayoutChanged?.call();
  }

  late final TextEditingController _controller = TextEditingController(
    text: widget.sourceText,
  );

  @override
  void didUpdateWidget(covariant QuickTranslateView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.sourceText.isEmpty && oldWidget.sourceText.isNotEmpty) {
      _sourceVisible = true;
    }
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

    final mac = theme.platform == TargetPlatform.macOS;
    SingleActivator command(LogicalKeyboardKey key, {bool shift = false}) =>
        SingleActivator(
          key,
          meta: mac,
          control: !mac,
          shift: shift,
          includeRepeats: false,
        );
    submitBindings.addAll({
      const SingleActivator(LogicalKeyboardKey.escape): () =>
          widget.onClose?.call(),
      command(LogicalKeyboardKey.keyW): () => widget.onClose?.call(),
      command(LogicalKeyboardKey.keyP): widget.onTogglePin,
      command(LogicalKeyboardKey.keyR): () {
        if (canTranslate) widget.onTranslate();
      },
      command(LogicalKeyboardKey.keyS): () {
        if (widget.favoriteAvailable && !widget.updatingFavorite) {
          widget.onToggleFavorite?.call();
        }
      },
      command(LogicalKeyboardKey.equal): () => _scaleFont(_fontScale + .1),
      command(LogicalKeyboardKey.equal, shift: true): () =>
          _scaleFont(_fontScale + .1),
      command(LogicalKeyboardKey.minus): () => _scaleFont(_fontScale - .1),
      command(LogicalKeyboardKey.digit0): () => _scaleFont(1),
    });
    final input = QuickTranslateInput(
      labels: widget.labels,
      controller: _controller,
      sourceText: widget.sourceText,
      submitting: widget.submitting,
      speechAvailable: widget.speechAvailable,
      submitWithModifier: widget.submitWithModifier,
      canTranslate: canTranslate,
      speakingKind: widget.speakingKind,
      onSourceTextChanged: widget.onSourceTextChanged,
      onStop: widget.onStop,
      onClear: widget.onClear,
      onTranslate: widget.onTranslate,
      onSpeakSource: widget.onSpeakSource,
      onStopSpeech: widget.onStopSpeech,
    );
    final result = QuickTranslateResultPanel(
      labels: widget.labels,
      services: widget.services,
      onReplace: widget.onReplace,
      submitting: widget.submitting,
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
    );
    return MediaQuery(
      data: MediaQuery.of(context).copyWith(
        textScaler: TextScaler.linear(
          MediaQuery.textScalerOf(context).scale(1) * _fontScale,
        ),
      ),
      child: CallbackShortcuts(
        bindings: submitBindings,
        child: Material(
          color: theme.colorScheme.surfaceContainerLowest,
          child: SingleChildScrollView(
            child: Padding(
              padding: const EdgeInsets.fromLTRB(16, 12, 16, 12),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  KeyedSubtree(
                    key: widget.toolbarKey,
                    child: QuickTranslateCommandHeader(
                      labels: widget.labels,
                      onClose: widget.onClose,
                      onStartDragging: widget.onStartDragging,
                      menuItems: [
                        PopupMenuItem(
                          value: 'source',
                          child: Text(
                            _sourceVisible
                                ? widget.labels.collapseSource
                                : widget.labels.showSource,
                          ),
                        ),
                        if (widget.onToggleReading != null)
                          PopupMenuItem(
                            value: 'reading',
                            child: Text(
                              MediaQuery.sizeOf(context).width >= 600
                                  ? widget.labels.compactReading
                                  : widget.labels.expandReading,
                            ),
                          ),
                        PopupMenuItem(
                          value: 'larger',
                          child: Text(widget.labels.fontLarger),
                        ),
                        PopupMenuItem(
                          value: 'smaller',
                          child: Text(widget.labels.fontSmaller),
                        ),
                        PopupMenuItem(
                          value: 'reset',
                          child: Text(widget.labels.fontReset),
                        ),
                        const PopupMenuDivider(),
                      ],
                      onMenuSelected: (value) {
                        switch (value) {
                          case 'source':
                            _toggleSource();
                          case 'reading':
                            widget.onToggleReading?.call();
                          case 'larger':
                            _scaleFont(_fontScale + .1);
                          case 'smaller':
                            _scaleFont(_fontScale - .1);
                          case 'reset':
                            _scaleFont(1);
                        }
                      },
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
                        const SizedBox(height: 12),
                        LayoutBuilder(
                          builder: (context, constraints) {
                            final wide =
                                constraints.maxWidth >= 568 && _sourceVisible;
                            final sourcePane = Container(
                              key: const ValueKey('quick-source-pane'),
                              constraints: const BoxConstraints(minHeight: 0),
                              padding: const EdgeInsets.all(12),
                              child: _sourceVisible
                                  ? input
                                  : InkWell(
                                      onTap: _toggleSource,
                                      child: Row(
                                        children: [
                                          Icon(
                                            Icons.unfold_more_rounded,
                                            size: 16,
                                            semanticLabel:
                                                widget.labels.showSource,
                                          ),
                                          const SizedBox(width: 8),
                                          Expanded(
                                            child: Text(
                                              widget.sourceText.isEmpty
                                                  ? widget.labels.inputHint
                                                  : widget.sourceText,
                                              maxLines: 1,
                                              overflow: TextOverflow.ellipsis,
                                              style: theme.textTheme.bodySmall,
                                            ),
                                          ),
                                        ],
                                      ),
                                    ),
                            );
                            final resultPane = Container(
                              key: const ValueKey('quick-result-pane'),
                              constraints: const BoxConstraints(minHeight: 0),
                              padding: const EdgeInsets.all(12),
                              color: theme.colorScheme.surface,
                              child: result,
                            );
                            return DecoratedBox(
                              decoration: BoxDecoration(
                                border: Border.all(
                                  color: theme.colorScheme.outlineVariant,
                                ),
                                borderRadius: BorderRadius.circular(16),
                              ),
                              child: ClipRRect(
                                borderRadius: BorderRadius.circular(16),
                                child: wide
                                    ? IntrinsicHeight(
                                        child: Row(
                                          crossAxisAlignment:
                                              CrossAxisAlignment.stretch,
                                          children: [
                                            Expanded(child: sourcePane),
                                            VerticalDivider(
                                              width: 1,
                                              color: theme
                                                  .colorScheme
                                                  .outlineVariant,
                                            ),
                                            Expanded(child: resultPane),
                                          ],
                                        ),
                                      )
                                    : Column(
                                        mainAxisSize: MainAxisSize.min,
                                        crossAxisAlignment:
                                            CrossAxisAlignment.stretch,
                                        children: [
                                          sourcePane,
                                          const Divider(),
                                          resultPane,
                                        ],
                                      ),
                              ),
                            );
                          },
                        ),
                        if (widget.notice != QuickTranslateNotice.none)
                          Padding(
                            padding: const EdgeInsets.only(top: 12),
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
                            padding: const EdgeInsets.only(top: 12),
                            child: StatusMessage(
                              kind: StatusKind.warning,
                              title: widget.labels.noServices,
                              action: OutlinedButton(
                                onPressed: widget.onConfigureServices,
                                child: Text(widget.labels.configureServices),
                              ),
                            ),
                          ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
