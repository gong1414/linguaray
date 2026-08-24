import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_ui/linguaray_ui.dart' show LinguaRayThemeContext;

import '../../shared/status_message.dart';

final class QuickTranslateLabels {
  const QuickTranslateLabels({
    required this.title,
    required this.inputHint,
    required this.translate,
    required this.clear,
    required this.copy,
    required this.copied,
    required this.pin,
    required this.unpin,
    required this.capture,
    required this.clipboard,
    required this.openSettings,
    required this.autoDetect,
    required this.autoMatch,
    required this.swapLanguages,
    required this.translating,
    required this.empty,
    required this.retry,
    required this.configureServices,
    required this.permissionDenied,
    required this.permissionNext,
    required this.captureCancelled,
    required this.serviceError,
    required this.noServices,
    required this.failureMessage,
    this.captureFailed = '',
    this.ocrNotConfigured = '',
    this.ocrEmpty = '',
    this.emptySelection = '',
    this.clipboardUnavailable = '',
    this.clipboardRestoreFailed = '',
    this.recheck = '',
    this.speakSource = '',
    this.speakResult = '',
    this.stopSpeaking = '',
    this.lookup = '',
    this.saveVocabulary = '',
    this.vocabularySaved = '',
    this.favorite = '',
    this.unfavorite = '',
    this.glossaryMatches = '',
    this.glossaryWarnings = '',
  });

  final String title;
  final String inputHint;
  final String translate;
  final String clear;
  final String copy;
  final String copied;
  final String pin;
  final String unpin;
  final String capture;
  final String clipboard;
  final String openSettings;
  final String autoDetect;
  final String autoMatch;
  final String swapLanguages;
  final String translating;
  final String empty;
  final String retry;
  final String configureServices;
  final String permissionDenied;
  final String permissionNext;
  final String captureCancelled;
  final String serviceError;
  final String noServices;
  final String Function(String? code) failureMessage;
  final String captureFailed;
  final String ocrNotConfigured;
  final String ocrEmpty;
  final String emptySelection;
  final String clipboardUnavailable;
  final String clipboardRestoreFailed;
  final String recheck;
  final String speakSource;
  final String speakResult;
  final String stopSpeaking;
  final String lookup;
  final String saveVocabulary;
  final String vocabularySaved;
  final String favorite;
  final String unfavorite;
  final String glossaryMatches;
  final String glossaryWarnings;
}

enum QuickTranslateNotice {
  none,
  permissionDenied,
  captureCancelled,
  captureFailed,
  ocrNotConfigured,
  ocrEmpty,
  emptySelection,
  clipboardUnavailable,
  clipboardRestoreFailed,
}

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
  String _selectedResultText = '';

  @override
  void didUpdateWidget(covariant QuickTranslateView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.sourceText != _controller.text) {
      _controller
        ..text = widget.sourceText
        ..selection = TextSelection.collapsed(offset: widget.sourceText.length);
    }
    if (widget.selectedResult?.text != oldWidget.selectedResult?.text) {
      _selectedResultText = '';
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
    final resultText = widget.selectedResult?.text ?? '';
    final failed =
        widget.selectedResult?.status == TranslationResultStatus.failed;
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
                child: _CommandHeader(
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
                    TextField(
                      key: const ValueKey('quick-source-input'),
                      controller: _controller,
                      autofocus: true,
                      minLines: 1,
                      maxLines: 4,
                      textInputAction: widget.submitWithModifier
                          ? TextInputAction.newline
                          : TextInputAction.go,
                      onChanged: widget.onSourceTextChanged,
                      decoration: InputDecoration(
                        hintText: widget.labels.inputHint,
                        suffixIcon: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            if (widget.sourceText.isNotEmpty)
                              IconButton(
                                tooltip: widget.labels.clear,
                                onPressed: widget.onClear,
                                icon: const Icon(Icons.close_rounded, size: 18),
                              ),
                            if (widget.speechAvailable &&
                                widget.onSpeakSource != null)
                              IconButton(
                                tooltip:
                                    widget.speakingKind ==
                                        SpeechUtteranceKind.source
                                    ? widget.labels.stopSpeaking
                                    : widget.labels.speakSource,
                                onPressed:
                                    widget.speakingKind ==
                                        SpeechUtteranceKind.source
                                    ? widget.onStopSpeech
                                    : widget.onSpeakSource,
                                icon: Icon(
                                  widget.speakingKind ==
                                          SpeechUtteranceKind.source
                                      ? Icons.stop_circle_outlined
                                      : Icons.volume_up_outlined,
                                  size: 18,
                                ),
                              ),
                            IconButton(
                              tooltip: widget.submitting
                                  ? widget.labels.translating
                                  : widget.labels.translate,
                              onPressed: canTranslate
                                  ? widget.onTranslate
                                  : null,
                              icon: widget.submitting
                                  ? const SizedBox.square(
                                      dimension: 14,
                                      child: CircularProgressIndicator(
                                        strokeWidth: 2,
                                      ),
                                    )
                                  : const Icon(Icons.arrow_forward_rounded),
                            ),
                          ],
                        ),
                      ),
                    ),
                    if (widget.notice != QuickTranslateNotice.none)
                      Padding(
                        padding: const EdgeInsets.only(top: 8),
                        child: _Notice(
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
                    if (resultText.trim().isNotEmpty) ...[
                      const SizedBox(height: 10),
                      Container(
                        height: 2,
                        color: context.brandColors.resultRule,
                      ),
                      const SizedBox(height: 8),
                      ConstrainedBox(
                        constraints: const BoxConstraints(maxHeight: 240),
                        child: SingleChildScrollView(
                          child: GestureDetector(
                            onDoubleTap: widget.copyResultOnDoubleClick
                                ? () => widget.onCopy(resultText)
                                : null,
                            child: SelectableText(
                              resultText,
                              key: const ValueKey('quick-result'),
                              style: theme.textTheme.bodyLarge,
                              onSelectionChanged: (selection, _) {
                                if (selection.isCollapsed) {
                                  _selectedResultText = '';
                                  return;
                                }
                                final start = selection.start.clamp(
                                  0,
                                  resultText.length,
                                );
                                final end = selection.end.clamp(
                                  0,
                                  resultText.length,
                                );
                                _selectedResultText = resultText.substring(
                                  start,
                                  end,
                                );
                              },
                            ),
                          ),
                        ),
                      ),
                      if (widget.glossaryMatches.isNotEmpty)
                        Padding(
                          padding: const EdgeInsets.only(top: 8),
                          child: _GlossaryMatches(
                            label: widget.labels.glossaryMatches,
                            matches: widget.glossaryMatches,
                          ),
                        ),
                      if (widget.glossaryWarnings.isNotEmpty)
                        Padding(
                          padding: const EdgeInsets.only(top: 8),
                          child: StatusMessage(
                            kind: StatusKind.warning,
                            title: widget.labels.glossaryWarnings,
                            body: widget.glossaryWarnings
                                .map(
                                  (warning) =>
                                      '${warning.term} → ${warning.expected}',
                                )
                                .join('\n'),
                          ),
                        ),
                      Align(
                        alignment: Alignment.centerRight,
                        child: Wrap(
                          spacing: 2,
                          children: [
                            if (widget.dictionaryAvailable &&
                                widget.onLookup != null)
                              IconButton(
                                tooltip: widget.labels.lookup,
                                onPressed: () => widget.onLookup!(
                                  _selectedResultText.trim().isEmpty
                                      ? resultText
                                      : _selectedResultText,
                                ),
                                icon: const Icon(
                                  Icons.menu_book_outlined,
                                  size: 18,
                                ),
                              ),
                            if (widget.onSaveVocabulary != null)
                              IconButton(
                                tooltip: widget.vocabularySaved
                                    ? widget.labels.vocabularySaved
                                    : widget.labels.saveVocabulary,
                                onPressed:
                                    widget.savingVocabulary ||
                                        widget.vocabularySaved
                                    ? null
                                    : widget.onSaveVocabulary,
                                icon: widget.savingVocabulary
                                    ? const SizedBox.square(
                                        dimension: 16,
                                        child: CircularProgressIndicator(
                                          strokeWidth: 2,
                                        ),
                                      )
                                    : Icon(
                                        widget.vocabularySaved
                                            ? Icons.bookmark_added_rounded
                                            : Icons.bookmark_add_outlined,
                                        size: 18,
                                      ),
                              ),
                            if (widget.onToggleFavorite != null)
                              IconButton(
                                tooltip: widget.favorite
                                    ? widget.labels.unfavorite
                                    : widget.labels.favorite,
                                onPressed:
                                    widget.favoriteAvailable &&
                                        !widget.updatingFavorite
                                    ? widget.onToggleFavorite
                                    : null,
                                icon: widget.updatingFavorite
                                    ? const SizedBox.square(
                                        dimension: 16,
                                        child: CircularProgressIndicator(
                                          strokeWidth: 2,
                                        ),
                                      )
                                    : Icon(
                                        widget.favorite
                                            ? Icons.star_rounded
                                            : Icons.star_outline_rounded,
                                        size: 18,
                                      ),
                              ),
                            if (widget.speechAvailable &&
                                widget.onSpeakResult != null)
                              IconButton(
                                tooltip:
                                    widget.speakingKind ==
                                        SpeechUtteranceKind.translation
                                    ? widget.labels.stopSpeaking
                                    : widget.labels.speakResult,
                                onPressed:
                                    widget.speakingKind ==
                                        SpeechUtteranceKind.translation
                                    ? widget.onStopSpeech
                                    : widget.onSpeakResult,
                                icon: Icon(
                                  widget.speakingKind ==
                                          SpeechUtteranceKind.translation
                                      ? Icons.stop_circle_outlined
                                      : Icons.volume_up_outlined,
                                  size: 18,
                                ),
                              ),
                            TextButton.icon(
                              onPressed: () => widget.onCopy(resultText),
                              icon: Icon(
                                widget.copied
                                    ? Icons.check_rounded
                                    : Icons.copy_rounded,
                                size: 16,
                              ),
                              label: Text(
                                widget.copied
                                    ? widget.labels.copied
                                    : widget.labels.copy,
                              ),
                            ),
                          ],
                        ),
                      ),
                    ] else if (failed)
                      Padding(
                        padding: const EdgeInsets.only(top: 8),
                        child: StatusMessage(
                          kind: StatusKind.error,
                          title: widget.labels.failureMessage(
                            widget.selectedResult?.errorCode,
                          ),
                          action: OutlinedButton(
                            onPressed: widget.onTranslate,
                            child: Text(widget.labels.retry),
                          ),
                        ),
                      ),
                    if (widget.results.length > 1)
                      Padding(
                        padding: const EdgeInsets.only(top: 4),
                        child: Wrap(
                          spacing: 6,
                          children: [
                            for (final result in widget.results)
                              ChoiceChip(
                                label: Text(result.service.name),
                                selected:
                                    result.service.id ==
                                    widget.selectedServiceId,
                                onSelected: (_) =>
                                    widget.onServiceSelected(result.service.id),
                              ),
                          ],
                        ),
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

class _GlossaryMatches extends StatelessWidget {
  const _GlossaryMatches({required this.label, required this.matches});

  final String label;
  final List<GlossaryMatchHit> matches;

  @override
  Widget build(BuildContext context) {
    return Wrap(
      crossAxisAlignment: WrapCrossAlignment.center,
      spacing: 6,
      runSpacing: 4,
      children: [
        if (label.isNotEmpty)
          Text(label, style: Theme.of(context).textTheme.labelMedium),
        for (final match in matches)
          Chip(
            visualDensity: VisualDensity.compact,
            label: Text('${match.term} → ${match.translation}'),
          ),
      ],
    );
  }
}

class _CommandHeader extends StatelessWidget {
  const _CommandHeader({
    required this.labels,
    required this.pinned,
    required this.languages,
    required this.sourceLanguage,
    required this.targetLanguage,
    required this.onTogglePin,
    required this.onCapture,
    required this.onClipboard,
    required this.onOpenSettings,
    required this.onSourceLanguageChanged,
    required this.onTargetLanguageChanged,
    required this.onSwapLanguages,
  });

  final QuickTranslateLabels labels;
  final bool pinned;
  final List<LanguageOption> languages;
  final String sourceLanguage;
  final String targetLanguage;
  final VoidCallback onTogglePin;
  final VoidCallback onCapture;
  final VoidCallback onClipboard;
  final VoidCallback onOpenSettings;
  final ValueChanged<String> onSourceLanguageChanged;
  final ValueChanged<String> onTargetLanguageChanged;
  final VoidCallback onSwapLanguages;

  @override
  Widget build(BuildContext context) {
    final sourceItems = [
      LanguageOption(code: autoLanguageCode, name: labels.autoDetect),
      ...languages,
    ];
    final targetItems = [
      LanguageOption(code: automaticTargetCode, name: labels.autoMatch),
      ...languages,
    ];
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          children: [
            Expanded(
              child: Text(
                labels.title,
                style: Theme.of(context).textTheme.titleMedium,
              ),
            ),
            IconButton(
              tooltip: labels.capture,
              onPressed: onCapture,
              icon: const Icon(Icons.crop_free_rounded, size: 18),
            ),
            IconButton(
              tooltip: labels.clipboard,
              onPressed: onClipboard,
              icon: const Icon(Icons.content_paste_rounded, size: 18),
            ),
            IconButton(
              tooltip: pinned ? labels.unpin : labels.pin,
              onPressed: onTogglePin,
              icon: Icon(
                pinned ? Icons.push_pin_rounded : Icons.push_pin_outlined,
                size: 18,
              ),
            ),
            PopupMenuButton<String>(
              tooltip: labels.openSettings,
              onSelected: (value) {
                switch (value) {
                  case 'settings':
                    onOpenSettings();
                }
              },
              itemBuilder: (context) => [
                PopupMenuItem(
                  value: 'settings',
                  child: Text(labels.openSettings),
                ),
              ],
            ),
          ],
        ),
        Row(
          children: [
            Expanded(
              child: DropdownButtonHideUnderline(
                child: DropdownButton<String>(
                  value: sourceItems.any((item) => item.code == sourceLanguage)
                      ? sourceLanguage
                      : autoLanguageCode,
                  isExpanded: true,
                  isDense: true,
                  items: [
                    for (final item in sourceItems)
                      DropdownMenuItem(
                        value: item.code,
                        child: Text(item.name, overflow: TextOverflow.ellipsis),
                      ),
                  ],
                  onChanged: (value) {
                    if (value != null) onSourceLanguageChanged(value);
                  },
                ),
              ),
            ),
            IconButton(
              tooltip: labels.swapLanguages,
              onPressed: onSwapLanguages,
              icon: const Icon(Icons.swap_horiz_rounded, size: 18),
            ),
            Expanded(
              child: DropdownButtonHideUnderline(
                child: DropdownButton<String>(
                  value: targetItems.any((item) => item.code == targetLanguage)
                      ? targetLanguage
                      : automaticTargetCode,
                  isExpanded: true,
                  isDense: true,
                  items: [
                    for (final item in targetItems)
                      DropdownMenuItem(
                        value: item.code,
                        child: Text(item.name, overflow: TextOverflow.ellipsis),
                      ),
                  ],
                  onChanged: (value) {
                    if (value != null) onTargetLanguageChanged(value);
                  },
                ),
              ),
            ),
          ],
        ),
      ],
    );
  }
}

class _Notice extends StatelessWidget {
  const _Notice({
    required this.labels,
    required this.notice,
    required this.onRecheck,
    required this.onConfigureOcr,
    required this.onConfigureServices,
    required this.onRetryCapture,
  });

  final QuickTranslateLabels labels;
  final QuickTranslateNotice notice;
  final VoidCallback onRecheck;
  final VoidCallback? onConfigureOcr;
  final VoidCallback onConfigureServices;
  final VoidCallback onRetryCapture;

  @override
  Widget build(BuildContext context) {
    return switch (notice) {
      QuickTranslateNotice.none => const SizedBox.shrink(),
      QuickTranslateNotice.permissionDenied => StatusMessage(
        kind: StatusKind.warning,
        title: labels.permissionDenied,
        body: labels.permissionNext,
        action: OutlinedButton(
          onPressed: onRecheck,
          child: Text(
            labels.recheck.isEmpty ? labels.openSettings : labels.recheck,
          ),
        ),
      ),
      QuickTranslateNotice.captureCancelled => StatusMessage(
        kind: StatusKind.info,
        title: labels.captureCancelled,
      ),
      QuickTranslateNotice.captureFailed => StatusMessage(
        kind: StatusKind.error,
        title: labels.captureFailed.isEmpty
            ? labels.failureMessage(AppErrorCode.captureFailed.wireName)
            : labels.captureFailed,
        action: OutlinedButton(
          onPressed: onRetryCapture,
          child: Text(labels.retry),
        ),
      ),
      QuickTranslateNotice.ocrNotConfigured => StatusMessage(
        kind: StatusKind.warning,
        title: labels.ocrNotConfigured.isEmpty
            ? labels.failureMessage(AppErrorCode.ocrNotConfigured.wireName)
            : labels.ocrNotConfigured,
        action: OutlinedButton(
          onPressed: onConfigureOcr ?? onConfigureServices,
          child: Text(labels.configureServices),
        ),
      ),
      QuickTranslateNotice.ocrEmpty => StatusMessage(
        kind: StatusKind.warning,
        title: labels.ocrEmpty.isEmpty
            ? labels.failureMessage(AppErrorCode.ocrEmpty.wireName)
            : labels.ocrEmpty,
        action: OutlinedButton(
          onPressed: onRetryCapture,
          child: Text(labels.retry),
        ),
      ),
      QuickTranslateNotice.emptySelection => StatusMessage(
        kind: StatusKind.info,
        title: labels.emptySelection.isEmpty
            ? labels.failureMessage(AppErrorCode.emptySelection.wireName)
            : labels.emptySelection,
      ),
      QuickTranslateNotice.clipboardUnavailable => StatusMessage(
        kind: StatusKind.warning,
        title: labels.clipboardUnavailable.isEmpty
            ? labels.failureMessage(AppErrorCode.clipboardUnavailable.wireName)
            : labels.clipboardUnavailable,
      ),
      QuickTranslateNotice.clipboardRestoreFailed => StatusMessage(
        kind: StatusKind.warning,
        title: labels.clipboardRestoreFailed.isEmpty
            ? labels.failureMessage(
                AppErrorCode.clipboardRestoreFailed.wireName,
              )
            : labels.clipboardRestoreFailed,
      ),
    };
  }
}
