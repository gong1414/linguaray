import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_ui/linguaray_ui.dart' show LinguaRayThemeContext;

import '../../shared/status_message.dart';

final class TranslationWorkspaceLabels {
  const TranslationWorkspaceLabels({
    required this.title,
    required this.subtitle,
    required this.source,
    required this.target,
    required this.autoDetect,
    required this.autoMatch,
    required this.inputHint,
    required this.translate,
    required this.clear,
    required this.swapLanguages,
    required this.loadingServices,
    required this.noServices,
    required this.translating,
    required this.failed,
    required this.empty,
    required this.services,
    required this.copy,
    required this.copied,
    required this.configureServices,
    required this.retry,
    required this.characterCount,
    required this.failureMessage,
    required this.partialFailure,
    required this.streaming,
  });

  final String title;
  final String subtitle;
  final String source;
  final String target;
  final String autoDetect;
  final String autoMatch;
  final String inputHint;
  final String translate;
  final String clear;
  final String swapLanguages;
  final String loadingServices;
  final String noServices;
  final String translating;
  final String failed;
  final String empty;
  final String services;
  final String copy;
  final String copied;
  final String configureServices;
  final String retry;
  final String Function(int count) characterCount;
  final String Function(String? code) failureMessage;
  final String Function(int failedCount) partialFailure;
  final String streaming;
}

class TranslationWorkspaceView extends StatefulWidget {
  const TranslationWorkspaceView({
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
    required this.onConfigureServices,
    super.key,
    this.selectedResult,
    this.results = const [],
    this.detectedLanguage,
    this.resolvedTargetLanguage,
    this.loadingCatalog = false,
    this.submitting = false,
    this.catalogFailed = false,
    this.submissionFailed = false,
    this.copied = false,
    this.showHeader = true,
  });

  final TranslationWorkspaceLabels labels;
  final List<LanguageOption> languages;
  final List<TranslationServiceOption> services;
  final String sourceText;
  final String sourceLanguage;
  final String targetLanguage;
  final String? selectedServiceId;
  final ServiceTranslationResult? selectedResult;
  final List<ServiceTranslationResult> results;
  final String? detectedLanguage;
  final String? resolvedTargetLanguage;
  final bool loadingCatalog;
  final bool submitting;
  final bool catalogFailed;
  final bool submissionFailed;
  final bool copied;
  final bool showHeader;
  final ValueChanged<String> onSourceTextChanged;
  final ValueChanged<String> onSourceLanguageChanged;
  final ValueChanged<String> onTargetLanguageChanged;
  final ValueChanged<String> onServiceSelected;
  final VoidCallback onSwapLanguages;
  final VoidCallback onTranslate;
  final VoidCallback onClear;
  final ValueChanged<String> onCopy;
  final VoidCallback onConfigureServices;

  @override
  State<TranslationWorkspaceView> createState() =>
      _TranslationWorkspaceViewState();
}

class _TranslationWorkspaceViewState extends State<TranslationWorkspaceView> {
  late final TextEditingController _textController = TextEditingController(
    text: widget.sourceText,
  );

  @override
  void didUpdateWidget(covariant TranslationWorkspaceView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.sourceText != _textController.text) {
      _textController
        ..text = widget.sourceText
        ..selection = TextSelection.collapsed(offset: widget.sourceText.length);
    }
  }

  @override
  void dispose() {
    _textController.dispose();
    super.dispose();
  }

  bool get _canTranslate =>
      widget.sourceText.trim().isNotEmpty &&
      widget.services.isNotEmpty &&
      !widget.loadingCatalog &&
      !widget.submitting;

  bool get _canSwap => !widget.loadingCatalog && widget.languages.isNotEmpty;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return CallbackShortcuts(
      bindings: {
        const SingleActivator(LogicalKeyboardKey.enter, meta: true): () {
          if (_canTranslate) widget.onTranslate();
        },
        const SingleActivator(LogicalKeyboardKey.enter, control: true): () {
          if (_canTranslate) widget.onTranslate();
        },
      },
      child: Material(
        color: theme.scaffoldBackgroundColor,
        child: Padding(
          padding: const EdgeInsets.fromLTRB(20, 16, 20, 16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              if (widget.showHeader) ...[
                Text(
                  widget.labels.title,
                  style: theme.textTheme.headlineMedium,
                ),
                const SizedBox(height: 4),
                Text(
                  widget.labels.subtitle,
                  style: theme.textTheme.bodyMedium?.copyWith(
                    color: theme.colorScheme.onSurfaceVariant,
                  ),
                ),
                const SizedBox(height: 16),
              ],
              _ControlStrip(
                labels: widget.labels,
                languages: widget.languages,
                sourceLanguage: widget.sourceLanguage,
                targetLanguage: widget.targetLanguage,
                detectedLanguage: widget.detectedLanguage,
                resolvedTargetLanguage: widget.resolvedTargetLanguage,
                canSwap: _canSwap,
                canTranslate: _canTranslate,
                submitting: widget.submitting,
                onSourceLanguageChanged: widget.onSourceLanguageChanged,
                onTargetLanguageChanged: widget.onTargetLanguageChanged,
                onSwapLanguages: widget.onSwapLanguages,
                onTranslate: widget.onTranslate,
              ),
              const SizedBox(height: 12),
              Expanded(
                child: LayoutBuilder(
                  builder: (context, constraints) {
                    final source = _SourcePane(
                      labels: widget.labels,
                      controller: _textController,
                      sourceText: widget.sourceText,
                      onChanged: widget.onSourceTextChanged,
                      onClear: widget.onClear,
                    );
                    final result = _ResultPane(
                      labels: widget.labels,
                      services: widget.services,
                      selectedServiceId: widget.selectedServiceId,
                      selectedResult: widget.selectedResult,
                      results: widget.results,
                      loadingCatalog: widget.loadingCatalog,
                      catalogFailed: widget.catalogFailed,
                      submissionFailed: widget.submissionFailed,
                      submitting: widget.submitting,
                      copied: widget.copied,
                      onServiceSelected: widget.onServiceSelected,
                      onCopy: widget.onCopy,
                      onRetry: widget.onTranslate,
                      onConfigureServices: widget.onConfigureServices,
                    );

                    if (constraints.maxWidth >= 640) {
                      return DecoratedBox(
                        decoration: BoxDecoration(
                          color: theme.colorScheme.surfaceContainerLowest,
                          borderRadius: BorderRadius.circular(10),
                          border: Border.all(
                            color: theme.colorScheme.outlineVariant,
                          ),
                        ),
                        child: Row(
                          crossAxisAlignment: CrossAxisAlignment.stretch,
                          children: [
                            Expanded(child: source),
                            VerticalDivider(
                              width: 1,
                              color: theme.colorScheme.outlineVariant,
                            ),
                            Expanded(child: result),
                          ],
                        ),
                      );
                    }

                    return ListView(
                      children: [
                        DecoratedBox(
                          decoration: BoxDecoration(
                            color: theme.colorScheme.surfaceContainerLowest,
                            borderRadius: BorderRadius.circular(10),
                            border: Border.all(
                              color: theme.colorScheme.outlineVariant,
                            ),
                          ),
                          child: SizedBox(height: 240, child: source),
                        ),
                        const SizedBox(height: 10),
                        DecoratedBox(
                          decoration: BoxDecoration(
                            color: context.brandColors.resultSurface,
                            borderRadius: BorderRadius.circular(10),
                            border: Border.all(
                              color: theme.colorScheme.outlineVariant,
                            ),
                          ),
                          child: SizedBox(height: 260, child: result),
                        ),
                      ],
                    );
                  },
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _ControlStrip extends StatelessWidget {
  const _ControlStrip({
    required this.labels,
    required this.languages,
    required this.sourceLanguage,
    required this.targetLanguage,
    required this.detectedLanguage,
    required this.resolvedTargetLanguage,
    required this.canSwap,
    required this.canTranslate,
    required this.submitting,
    required this.onSourceLanguageChanged,
    required this.onTargetLanguageChanged,
    required this.onSwapLanguages,
    required this.onTranslate,
  });

  final TranslationWorkspaceLabels labels;
  final List<LanguageOption> languages;
  final String sourceLanguage;
  final String targetLanguage;
  final String? detectedLanguage;
  final String? resolvedTargetLanguage;
  final bool canSwap;
  final bool canTranslate;
  final bool submitting;
  final ValueChanged<String> onSourceLanguageChanged;
  final ValueChanged<String> onTargetLanguageChanged;
  final VoidCallback onSwapLanguages;
  final VoidCallback onTranslate;

  @override
  Widget build(BuildContext context) {
    final languageByCode = <String, String>{
      for (final language in languages) language.code: language.name,
    };
    final sourceItems = <LanguageOption>[
      LanguageOption(code: autoLanguageCode, name: labels.autoDetect),
      ...languages.where((language) => language.code != autoLanguageCode),
    ];
    final targetItems = <LanguageOption>[
      LanguageOption(code: automaticTargetCode, name: labels.autoMatch),
      ...languages.where((language) => language.code != autoLanguageCode),
    ];
    if (!targetItems.any((language) => language.code == targetLanguage)) {
      targetItems.add(
        LanguageOption(
          code: targetLanguage,
          name: languageByCode[targetLanguage] ?? targetLanguage,
        ),
      );
    }
    final detectedName = detectedLanguage == null
        ? null
        : languageByCode[detectedLanguage] ?? detectedLanguage;
    final resolvedTargetName = resolvedTargetLanguage == null
        ? null
        : languageByCode[resolvedTargetLanguage] ?? resolvedTargetLanguage;

    return Row(
      children: [
        Expanded(
          child: _LanguageMenu(
            key: const ValueKey('source-language'),
            semanticsLabel: labels.source,
            value: sourceItems.any((item) => item.code == sourceLanguage)
                ? sourceLanguage
                : autoLanguageCode,
            items: sourceItems,
            enabled: canSwap,
            suffix: sourceLanguage == autoLanguageCode && detectedName != null
                ? detectedName
                : null,
            onChanged: onSourceLanguageChanged,
          ),
        ),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 4),
          child: IconButton(
            key: const ValueKey('swap-languages'),
            tooltip: labels.swapLanguages,
            onPressed: canSwap ? onSwapLanguages : null,
            icon: const Icon(Icons.swap_horiz_rounded),
          ),
        ),
        Expanded(
          child: _LanguageMenu(
            key: const ValueKey('target-language'),
            semanticsLabel: labels.target,
            value: targetLanguage,
            items: targetItems,
            enabled: canSwap,
            suffix: targetLanguage == automaticTargetCode
                ? resolvedTargetName
                : null,
            onChanged: onTargetLanguageChanged,
          ),
        ),
        const SizedBox(width: 12),
        FilledButton.icon(
          key: const ValueKey('translate-button'),
          onPressed: canTranslate ? onTranslate : null,
          icon: submitting
              ? const SizedBox.square(
                  dimension: 16,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : const Icon(Icons.translate_rounded, size: 18),
          label: Text(submitting ? labels.translating : labels.translate),
        ),
      ],
    );
  }
}

class _LanguageMenu extends StatelessWidget {
  const _LanguageMenu({
    required this.semanticsLabel,
    required this.value,
    required this.items,
    required this.enabled,
    required this.onChanged,
    super.key,
    this.suffix,
  });

  final String semanticsLabel;
  final String value;
  final List<LanguageOption> items;
  final bool enabled;
  final String? suffix;
  final ValueChanged<String> onChanged;

  @override
  Widget build(BuildContext context) {
    final selected = items.firstWhere(
      (item) => item.code == value,
      orElse: () => items.first,
    );
    final label = suffix == null ? selected.name : '${selected.name} · $suffix';

    return Semantics(
      label: semanticsLabel,
      button: true,
      enabled: enabled,
      child: DropdownMenu<String>(
        key: ValueKey('$semanticsLabel-$value'),
        initialSelection: value,
        enabled: enabled,
        expandedInsets: EdgeInsets.zero,
        requestFocusOnTap: false,
        inputDecorationTheme: InputDecorationTheme(
          isDense: true,
          filled: true,
          fillColor: Theme.of(context).colorScheme.surfaceContainerLowest,
          contentPadding: const EdgeInsets.symmetric(
            horizontal: 12,
            vertical: 10,
          ),
          border: OutlineInputBorder(
            borderRadius: BorderRadius.circular(10),
            borderSide: BorderSide(
              color: Theme.of(context).colorScheme.outlineVariant,
            ),
          ),
        ),
        dropdownMenuEntries: [
          for (final item in items)
            DropdownMenuEntry(
              value: item.code,
              label: item.code == value && suffix != null
                  ? '${item.name} · $suffix'
                  : item.name,
            ),
        ],
        onSelected: enabled
            ? (value) {
                if (value != null) onChanged(value);
              }
            : null,
        hintText: label,
      ),
    );
  }
}

class _SourcePane extends StatelessWidget {
  const _SourcePane({
    required this.labels,
    required this.controller,
    required this.sourceText,
    required this.onChanged,
    required this.onClear,
  });

  final TranslationWorkspaceLabels labels;
  final TextEditingController controller;
  final String sourceText;
  final ValueChanged<String> onChanged;
  final VoidCallback onClear;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(
                  labels.source,
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ),
              Text(
                labels.characterCount(sourceText.characters.length),
                style: Theme.of(context).textTheme.labelMedium?.copyWith(
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
              ),
              IconButton(
                key: const ValueKey('clear-source'),
                tooltip: labels.clear,
                onPressed: sourceText.isEmpty ? null : onClear,
                icon: const Icon(Icons.close_rounded),
              ),
            ],
          ),
          const SizedBox(height: 4),
          Expanded(
            child: TextField(
              key: const ValueKey('translation-source-input'),
              controller: controller,
              autofocus: true,
              expands: true,
              minLines: null,
              maxLines: null,
              textAlignVertical: TextAlignVertical.top,
              decoration: InputDecoration(
                hintText: labels.inputHint,
                border: InputBorder.none,
                enabledBorder: InputBorder.none,
                focusedBorder: InputBorder.none,
                filled: false,
                contentPadding: EdgeInsets.zero,
              ),
              onChanged: onChanged,
            ),
          ),
        ],
      ),
    );
  }
}

class _ResultPane extends StatelessWidget {
  const _ResultPane({
    required this.labels,
    required this.services,
    required this.selectedServiceId,
    required this.selectedResult,
    required this.results,
    required this.loadingCatalog,
    required this.catalogFailed,
    required this.submissionFailed,
    required this.submitting,
    required this.copied,
    required this.onServiceSelected,
    required this.onCopy,
    required this.onRetry,
    required this.onConfigureServices,
  });

  final TranslationWorkspaceLabels labels;
  final List<TranslationServiceOption> services;
  final String? selectedServiceId;
  final ServiceTranslationResult? selectedResult;
  final List<ServiceTranslationResult> results;
  final bool loadingCatalog;
  final bool catalogFailed;
  final bool submissionFailed;
  final bool submitting;
  final bool copied;
  final ValueChanged<String> onServiceSelected;
  final ValueChanged<String> onCopy;
  final VoidCallback onRetry;
  final VoidCallback onConfigureServices;

  @override
  Widget build(BuildContext context) {
    final result = selectedResult;
    final text = result?.text ?? '';
    final failedCount = results
        .where((item) => item.status == TranslationResultStatus.failed)
        .length;
    final completedCount = results
        .where((item) => item.status == TranslationResultStatus.completed)
        .length;
    final isStreaming =
        submitting ||
        result?.status == TranslationResultStatus.translating ||
        result?.status == TranslationResultStatus.waiting;

    return ColoredBox(
      color: context.brandColors.resultSurface,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 8, 8, 4),
            child: Row(
              children: [
                Expanded(
                  child: Text(
                    labels.target,
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                ),
                IconButton(
                  key: const ValueKey('copy-result'),
                  tooltip: copied ? labels.copied : labels.copy,
                  onPressed: text.trim().isEmpty ? null : () => onCopy(text),
                  icon: Icon(copied ? Icons.check_rounded : Icons.copy_rounded),
                ),
              ],
            ),
          ),
          if (services.length > 1)
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 0, 16, 8),
              child: _ServiceSwitcher(
                labels: labels,
                services: services,
                results: results,
                selectedServiceId: selectedServiceId,
                onServiceSelected: onServiceSelected,
              ),
            ),
          if (isStreaming) const LinearProgressIndicator(minHeight: 2),
          if (failedCount > 0 && completedCount > 0)
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 8, 16, 0),
              child: Text(
                labels.partialFailure(failedCount),
                style: Theme.of(context).textTheme.labelMedium
                    ?.copyWith(color: Theme.of(context).colorScheme.error),
              ),
            ),
          Expanded(
            child: Padding(
              padding: const EdgeInsets.fromLTRB(16, 12, 16, 16),
              child: _ResultBody(
                labels: labels,
                text: text,
                loadingCatalog: loadingCatalog,
                noServices: services.isEmpty,
                failed:
                    catalogFailed ||
                    submissionFailed ||
                    result?.status == TranslationResultStatus.failed,
                failureCode: result?.errorCode,
                isTranslating: isStreaming,
                onRetry: onRetry,
                onConfigureServices: onConfigureServices,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _ServiceSwitcher extends StatelessWidget {
  const _ServiceSwitcher({
    required this.labels,
    required this.services,
    required this.results,
    required this.selectedServiceId,
    required this.onServiceSelected,
  });

  final TranslationWorkspaceLabels labels;
  final List<TranslationServiceOption> services;
  final List<ServiceTranslationResult> results;
  final String? selectedServiceId;
  final ValueChanged<String> onServiceSelected;

  @override
  Widget build(BuildContext context) {
    if (services.length <= 4) {
      return SegmentedButton<String>(
        segments: [
          for (final service in services)
            ButtonSegment(
              value: service.id,
              label: Text(service.name),
              icon: _statusIcon(service.id),
              tooltip: service.name,
            ),
        ],
        emptySelectionAllowed: true,
        selected: {if (selectedServiceId != null) selectedServiceId!},
        onSelectionChanged: (selection) {
          if (selection.isNotEmpty) onServiceSelected(selection.first);
        },
      );
    }

    return DropdownMenu<String>(
      initialSelection: selectedServiceId,
      label: Text(labels.services),
      dropdownMenuEntries: [
        for (final service in services)
          DropdownMenuEntry(value: service.id, label: service.name),
      ],
      onSelected: (value) {
        if (value != null) onServiceSelected(value);
      },
    );
  }

  Widget? _statusIcon(String serviceId) {
    final match = results.where((item) => item.service.id == serviceId);
    if (match.isEmpty) return null;
    return switch (match.first.status) {
      TranslationResultStatus.completed => const Icon(
        Icons.check_rounded,
        size: 14,
      ),
      TranslationResultStatus.failed => const Icon(
        Icons.error_outline_rounded,
        size: 14,
      ),
      TranslationResultStatus.translating ||
      TranslationResultStatus.waiting => const SizedBox.square(
        dimension: 12,
        child: CircularProgressIndicator(strokeWidth: 1.6),
      ),
    };
  }
}

class _ResultBody extends StatelessWidget {
  const _ResultBody({
    required this.labels,
    required this.text,
    required this.loadingCatalog,
    required this.noServices,
    required this.failed,
    required this.failureCode,
    required this.isTranslating,
    required this.onRetry,
    required this.onConfigureServices,
  });

  final TranslationWorkspaceLabels labels;
  final String text;
  final bool loadingCatalog;
  final bool noServices;
  final bool failed;
  final String? failureCode;
  final bool isTranslating;
  final VoidCallback onRetry;
  final VoidCallback onConfigureServices;

  @override
  Widget build(BuildContext context) {
    if (text.trim().isNotEmpty) {
      return SingleChildScrollView(
        child: SelectableText(
          text,
          key: const ValueKey('translation-result'),
          style: Theme.of(context).textTheme.bodyLarge,
        ),
      );
    }
    if (loadingCatalog) {
      return StatusMessage(
        kind: StatusKind.progress,
        title: labels.loadingServices,
      );
    }
    if (noServices) {
      return StatusMessage(
        kind: StatusKind.warning,
        title: labels.noServices,
        action: OutlinedButton(
          onPressed: onConfigureServices,
          child: Text(labels.configureServices),
        ),
      );
    }
    if (failed) {
      return StatusMessage(
        kind: StatusKind.error,
        title: labels.failureMessage(failureCode),
        action: OutlinedButton.icon(
          onPressed: onRetry,
          icon: const Icon(Icons.refresh_rounded),
          label: Text(labels.retry),
        ),
      );
    }
    return StatusMessage(
      kind: isTranslating ? StatusKind.progress : StatusKind.info,
      title: isTranslating ? labels.streaming : labels.empty,
    );
  }
}
