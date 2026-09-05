import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../../shared/status_message.dart';
import 'quick_translate_components.dart';
import 'quick_translate_models.dart';

class QuickTranslateResultPanel extends StatefulWidget {
  const QuickTranslateResultPanel({
    required this.labels,
    required this.results,
    required this.selectedServiceId,
    required this.glossaryMatches,
    required this.glossaryWarnings,
    required this.dictionaryAvailable,
    required this.speechAvailable,
    required this.savingVocabulary,
    required this.vocabularySaved,
    required this.favoriteAvailable,
    required this.favorite,
    required this.updatingFavorite,
    required this.copied,
    required this.copyResultOnDoubleClick,
    required this.onTranslate,
    required this.onCopy,
    required this.onServiceSelected,
    super.key,
    this.services = const [],
    this.onReplace,
    this.submitting = false,
    this.selectedResult,
    this.speakingKind,
    this.onLookup,
    this.onSaveVocabulary,
    this.onToggleFavorite,
    this.onSpeakResult,
    this.onStopSpeech,
  });

  final List<TranslationServiceOption> services;
  final VoidCallback? onReplace;
  final bool submitting;
  final QuickTranslateLabels labels;
  final List<ServiceTranslationResult> results;
  final ServiceTranslationResult? selectedResult;
  final String? selectedServiceId;
  final List<GlossaryMatchHit> glossaryMatches;
  final List<GlossaryComplianceWarning> glossaryWarnings;
  final bool dictionaryAvailable;
  final bool speechAvailable;
  final SpeechUtteranceKind? speakingKind;
  final bool savingVocabulary;
  final bool vocabularySaved;
  final bool favoriteAvailable;
  final bool favorite;
  final bool updatingFavorite;
  final bool copied;
  final bool copyResultOnDoubleClick;
  final VoidCallback onTranslate;
  final ValueChanged<String> onCopy;
  final ValueChanged<String> onServiceSelected;
  final ValueChanged<String>? onLookup;
  final VoidCallback? onSaveVocabulary;
  final VoidCallback? onToggleFavorite;
  final VoidCallback? onSpeakResult;
  final VoidCallback? onStopSpeech;

  @override
  State<QuickTranslateResultPanel> createState() =>
      _QuickTranslateResultPanelState();
}

class _QuickTranslateResultPanelState extends State<QuickTranslateResultPanel> {
  String _selectedText = '';

  @override
  void didUpdateWidget(covariant QuickTranslateResultPanel oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.selectedResult?.text != oldWidget.selectedResult?.text) {
      _selectedText = '';
    }
  }

  @override
  Widget build(BuildContext context) {
    final resultText = widget.selectedResult?.text ?? '';
    final failed =
        widget.selectedResult?.status == TranslationResultStatus.failed;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        ...[
          SizedBox(
            height: 32,
            child: Row(
              children: [
                Container(
                  width: 3,
                  height: 12,
                  decoration: BoxDecoration(
                    color: Theme.of(context).colorScheme.primary,
                    borderRadius: BorderRadius.circular(2),
                  ),
                ),
                const SizedBox(width: 7),
                Text(
                  widget.labels.resultLabel,
                  style: Theme.of(context).textTheme.labelMedium,
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: Text(
                    widget.selectedResult?.service.name ?? '',
                    textAlign: TextAlign.right,
                    style: Theme.of(context).textTheme.labelSmall,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 10),
          if (resultText.trim().isEmpty && !failed)
            Padding(
              padding: const EdgeInsets.symmetric(vertical: 18),
              child: Text(
                widget.selectedResult?.status ==
                        TranslationResultStatus.translating
                    ? widget.labels.translating
                    : widget.labels.resultPlaceholder,
                style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
                textAlign: TextAlign.center,
              ),
            ),
          if (widget.selectedResult?.status ==
              TranslationResultStatus.translating)
            const LinearProgressIndicator(minHeight: 2),
          if (resultText.trim().isNotEmpty)
            ConstrainedBox(
              constraints: const BoxConstraints(minHeight: 28, maxHeight: 300),
              child: SingleChildScrollView(
                child: GestureDetector(
                  onDoubleTap: widget.copyResultOnDoubleClick
                      ? () => widget.onCopy(resultText)
                      : null,
                  child: SelectableText(
                    resultText,
                    key: const ValueKey('quick-result'),
                    style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                      fontSize: 17,
                      height: 1.65,
                      letterSpacing: 0,
                    ),
                    onSelectionChanged: (selection, _) {
                      if (selection.isCollapsed) {
                        _selectedText = '';
                        return;
                      }
                      final start = selection.start.clamp(0, resultText.length);
                      final end = selection.end.clamp(0, resultText.length);
                      _selectedText = resultText.substring(start, end);
                    },
                  ),
                ),
              ),
            ),
          if (widget.glossaryMatches.isNotEmpty)
            Padding(
              padding: const EdgeInsets.only(top: 8),
              child: QuickTranslateGlossaryMatches(
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
                    .map((warning) => '${warning.term} → ${warning.expected}')
                    .join('\n'),
              ),
            ),
          if (resultText.trim().isNotEmpty)
            Align(
              alignment: Alignment.centerRight,
              child: Wrap(
                spacing: 2,
                children: [
                  if (widget.dictionaryAvailable && widget.onLookup != null)
                    IconButton(
                      tooltip: widget.labels.lookup,
                      onPressed: () => widget.onLookup!(
                        _selectedText.trim().isEmpty
                            ? resultText
                            : _selectedText,
                      ),
                      icon: const Icon(Icons.menu_book_outlined, size: 18),
                    ),
                  if (widget.onSaveVocabulary != null)
                    IconButton(
                      tooltip: widget.vocabularySaved
                          ? widget.labels.vocabularySaved
                          : widget.labels.saveVocabulary,
                      onPressed:
                          widget.savingVocabulary || widget.vocabularySaved
                          ? null
                          : widget.onSaveVocabulary,
                      icon: widget.savingVocabulary
                          ? const SizedBox.square(
                              dimension: 16,
                              child: CircularProgressIndicator(strokeWidth: 2),
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
                          widget.favoriteAvailable && !widget.updatingFavorite
                          ? widget.onToggleFavorite
                          : null,
                      icon: widget.updatingFavorite
                          ? const SizedBox.square(
                              dimension: 16,
                              child: CircularProgressIndicator(strokeWidth: 2),
                            )
                          : Icon(
                              widget.favorite
                                  ? Icons.star_rounded
                                  : Icons.star_outline_rounded,
                              size: 18,
                            ),
                    ),
                  if (widget.speechAvailable && widget.onSpeakResult != null)
                    IconButton(
                      tooltip:
                          widget.speakingKind == SpeechUtteranceKind.translation
                          ? widget.labels.stopSpeaking
                          : widget.labels.speakResult,
                      onPressed:
                          widget.speakingKind == SpeechUtteranceKind.translation
                          ? widget.onStopSpeech
                          : widget.onSpeakResult,
                      icon: Icon(
                        widget.speakingKind == SpeechUtteranceKind.translation
                            ? Icons.stop_circle_outlined
                            : Icons.volume_up_outlined,
                        size: 18,
                      ),
                    ),
                  TextButton.icon(
                    onPressed: () => widget.onCopy(resultText),
                    icon: Icon(
                      widget.copied ? Icons.check_rounded : Icons.copy_rounded,
                      size: 16,
                    ),
                    label: Text(
                      widget.copied ? widget.labels.copied : widget.labels.copy,
                    ),
                  ),
                ],
              ),
            ),
        ],
        if (failed)
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
        if (widget.services.length > 1)
          Padding(
            padding: const EdgeInsets.only(top: 8),
            child: Tooltip(
              message: widget.labels.serviceHint,
              child: Wrap(
                spacing: 6,
                runSpacing: 4,
                children: [
                  for (final service in widget.services)
                    ChoiceChip(
                      label: Text(service.name),
                      selected: service.id == widget.selectedServiceId,
                      onSelected: (_) => widget.onServiceSelected(service.id),
                    ),
                ],
              ),
            ),
          ),
        if (widget.selectedResult?.status == TranslationResultStatus.cancelled)
          Text(
            widget.labels.stopped,
            style: Theme.of(context).textTheme.bodySmall,
          ),
        if (widget.selectedResult?.status == TranslationResultStatus.completed)
          Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: [
              if (widget.onReplace != null)
                TextButton.icon(
                  onPressed: widget.onReplace,
                  icon: const Icon(Icons.find_replace_rounded, size: 16),
                  label: Text(widget.labels.replace),
                ),
              IconButton(
                tooltip: widget.labels.retry,
                onPressed: widget.submitting ? null : widget.onTranslate,
                icon: const Icon(Icons.refresh_rounded, size: 18),
              ),
            ],
          ),
      ],
    );
  }
}
