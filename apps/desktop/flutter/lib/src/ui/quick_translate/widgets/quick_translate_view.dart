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
    required this.openWorkbench,
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
  final String openWorkbench;
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
    required this.onOpenWorkbench,
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
  final VoidCallback onOpenWorkbench;
  final VoidCallback onOpenSettings;
  final VoidCallback onConfigureServices;
  final VoidCallback onRecheckPermissions;
  final VoidCallback? onConfigureOcr;

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
    final resultText = widget.selectedResult?.text ?? '';
    final failed =
        widget.selectedResult?.status == TranslationResultStatus.failed;
    final canTranslate =
        widget.sourceText.trim().isNotEmpty &&
        widget.services.isNotEmpty &&
        !widget.submitting;

    return CallbackShortcuts(
      bindings: {
        const SingleActivator(LogicalKeyboardKey.enter, meta: true): () {
          if (canTranslate) widget.onTranslate();
        },
        const SingleActivator(LogicalKeyboardKey.enter): () {
          if (canTranslate) widget.onTranslate();
        },
      },
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
                  onOpenWorkbench: widget.onOpenWorkbench,
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
                      textInputAction: TextInputAction.go,
                      onSubmitted: (_) {
                        if (canTranslate) widget.onTranslate();
                      },
                      onChanged: widget.onSourceTextChanged,
                      decoration: InputDecoration(
                        hintText: widget.labels.inputHint,
                        suffixIcon: IconButton(
                          tooltip: widget.submitting
                              ? widget.labels.translating
                              : widget.labels.translate,
                          onPressed: canTranslate ? widget.onTranslate : null,
                          icon: widget.submitting
                              ? const SizedBox.square(
                                  dimension: 14,
                                  child: CircularProgressIndicator(
                                    strokeWidth: 2,
                                  ),
                                )
                              : const Icon(Icons.arrow_forward_rounded),
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
                          child: SelectableText(
                            resultText,
                            key: const ValueKey('quick-result'),
                            style: theme.textTheme.bodyLarge,
                          ),
                        ),
                      ),
                      Align(
                        alignment: Alignment.centerRight,
                        child: TextButton.icon(
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
    required this.onOpenWorkbench,
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
  final VoidCallback onOpenWorkbench;
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
                  case 'workbench':
                    onOpenWorkbench();
                  case 'settings':
                    onOpenSettings();
                }
              },
              itemBuilder: (context) => [
                PopupMenuItem(
                  value: 'workbench',
                  child: Text(labels.openWorkbench),
                ),
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
