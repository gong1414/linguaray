import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_ui/linguaray_ui.dart' show LinguaRayThemeContext;

import '../../../features.dart';
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
    this.speakSource = '',
    this.speakResult = '',
    this.stopSpeech = '',
    this.lookup = '',
    this.saveWord = '',
    this.savedWord = '',
    this.glossaryHits = '',
    this.glossaryEmpty = '',
    this.glossaryWarning = '',
    this.recoveryRecheck = '',
    this.recoveryPermissions = '',
    this.recoveryConfigureOcr = '',
    this.recoveryConfigureProvider = '',
    this.recoveryEditInput = '',
    this.recoveryChooseLanguage = '',
    this.recoverySwitchToGoogleWeb = '',
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
  final String speakSource;
  final String speakResult;
  final String stopSpeech;
  final String lookup;
  final String saveWord;
  final String savedWord;
  final String glossaryHits;
  final String glossaryEmpty;
  final String glossaryWarning;
  final String recoveryRecheck;
  final String recoveryPermissions;
  final String recoveryConfigureOcr;
  final String recoveryConfigureProvider;
  final String recoveryEditInput;
  final String recoveryChooseLanguage;
  final String recoverySwitchToGoogleWeb;
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
    this.glossaryMatches = const [],
    this.glossaryWarnings = const [],
    this.dictionaryAvailable = false,
    this.speechAvailable = false,
    this.speakingKind,
    this.onSpeakSource,
    this.onSpeakResult,
    this.onStopSpeech,
    this.onLookup,
    this.onSaveVocabulary,
    this.vocabularySaved = false,
    this.savingVocabulary = false,
    this.onRetry,
    this.onRecovery,
    this.inputSubmitMode = InputSubmitMode.commandEnter,
    this.doubleClickCopy = false,
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
  final List<GlossaryMatchHit> glossaryMatches;
  final List<GlossaryComplianceWarning> glossaryWarnings;
  final bool dictionaryAvailable;
  final bool speechAvailable;
  final SpeechUtteranceKind? speakingKind;
  final VoidCallback? onSpeakSource;
  final VoidCallback? onSpeakResult;
  final VoidCallback? onStopSpeech;
  final ValueChanged<String>? onLookup;
  final VoidCallback? onSaveVocabulary;
  final bool vocabularySaved;
  final bool savingVocabulary;
  final VoidCallback? onRetry;
  final ValueChanged<RecoveryAction>? onRecovery;
  final InputSubmitMode inputSubmitMode;
  final bool doubleClickCopy;
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
    final enterBinding = widget.inputSubmitMode == InputSubmitMode.enter
        ? const SingleActivator(LogicalKeyboardKey.enter)
        : const SingleActivator(LogicalKeyboardKey.enter, meta: true);

    return CallbackShortcuts(
      bindings: {
        enterBinding: () {
          if (_canTranslate) widget.onTranslate();
        },
        const SingleActivator(LogicalKeyboardKey.enter, control: true): () {
          if (_canTranslate) widget.onTranslate();
        },
      },
      child: Material(
        color: theme.scaffoldBackgroundColor,
        child: LayoutBuilder(
          builder: (context, viewport) {
            // The native runner starts with a tiny bootstrap surface before
            // AppWindowController applies the 840×560 workbench constraints.
            // Rendering the two flexible panes during that transient frame
            // would overflow even though the visible window is never that
            // size.
            if (viewport.maxWidth < 360 || viewport.maxHeight < 240) {
              return const SizedBox.expand();
            }
            return Padding(
              padding: const EdgeInsets.fromLTRB(20, 12, 20, 16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  if (widget.showHeader) ...[
                    Text(
                      widget.labels.title,
                      style: theme.textTheme.headlineMedium,
                    ),
                    const SizedBox(height: 2),
                    Text(
                      widget.labels.subtitle,
                      style: theme.textTheme.bodyMedium?.copyWith(
                        color: theme.colorScheme.onSurfaceVariant,
                      ),
                    ),
                    const SizedBox(height: 12),
                  ],
                  _ControlStrip(
                    labels: widget.labels,
                    languages: widget.languages,
                    services: widget.services,
                    sourceLanguage: widget.sourceLanguage,
                    targetLanguage: widget.targetLanguage,
                    detectedLanguage: widget.detectedLanguage,
                    resolvedTargetLanguage: widget.resolvedTargetLanguage,
                    selectedServiceId: widget.selectedServiceId,
                    results: widget.results,
                    canSwap: _canSwap,
                    canTranslate: _canTranslate,
                    submitting: widget.submitting,
                    onSourceLanguageChanged: widget.onSourceLanguageChanged,
                    onTargetLanguageChanged: widget.onTargetLanguageChanged,
                    onServiceSelected: widget.onServiceSelected,
                    onSwapLanguages: widget.onSwapLanguages,
                    onTranslate: widget.onTranslate,
                  ),
                  const SizedBox(height: 12),
                  Expanded(
                    child: LayoutBuilder(
                      builder: (context, constraints) {
                        final showAside =
                            kTranslationAsideEnabled &&
                            constraints.maxWidth >= 880 &&
                            (widget.glossaryMatches.isNotEmpty ||
                                widget.glossaryWarnings.isNotEmpty);
                        final composer = _TaskComposer(
                          labels: widget.labels,
                          controller: _textController,
                          sourceText: widget.sourceText,
                          selectedResult: widget.selectedResult,
                          results: widget.results,
                          services: widget.services,
                          selectedServiceId: widget.selectedServiceId,
                          loadingCatalog: widget.loadingCatalog,
                          catalogFailed: widget.catalogFailed,
                          submissionFailed: widget.submissionFailed,
                          submitting: widget.submitting,
                          copied: widget.copied,
                          speechAvailable: widget.speechAvailable,
                          speakingKind: widget.speakingKind,
                          dictionaryAvailable: widget.dictionaryAvailable,
                          doubleClickCopy: widget.doubleClickCopy,
                          onChanged: widget.onSourceTextChanged,
                          onClear: widget.onClear,
                          onCopy: widget.onCopy,
                          onServiceSelected: widget.onServiceSelected,
                          onRetry: widget.onRetry ?? widget.onTranslate,
                          onConfigureServices: widget.onConfigureServices,
                          onSpeakSource: widget.onSpeakSource,
                          onSpeakResult: widget.onSpeakResult,
                          onStopSpeech: widget.onStopSpeech,
                          onLookup: widget.onLookup,
                          onSaveVocabulary: widget.onSaveVocabulary,
                          vocabularySaved: widget.vocabularySaved,
                          savingVocabulary: widget.savingVocabulary,
                          onRecovery: widget.onRecovery,
                        );
                        if (!showAside) return composer;
                        return Row(
                          crossAxisAlignment: CrossAxisAlignment.stretch,
                          children: [
                            Expanded(child: composer),
                            const SizedBox(width: 12),
                            SizedBox(
                              width: 260,
                              child: _AsidePanel(
                                labels: widget.labels,
                                matches: widget.glossaryMatches,
                                warnings: widget.glossaryWarnings,
                              ),
                            ),
                          ],
                        );
                      },
                    ),
                  ),
                ],
              ),
            );
          },
        ),
      ),
    );
  }
}

class _ControlStrip extends StatelessWidget {
  const _ControlStrip({
    required this.labels,
    required this.languages,
    required this.services,
    required this.sourceLanguage,
    required this.targetLanguage,
    required this.detectedLanguage,
    required this.resolvedTargetLanguage,
    required this.selectedServiceId,
    required this.results,
    required this.canSwap,
    required this.canTranslate,
    required this.submitting,
    required this.onSourceLanguageChanged,
    required this.onTargetLanguageChanged,
    required this.onServiceSelected,
    required this.onSwapLanguages,
    required this.onTranslate,
  });

  final TranslationWorkspaceLabels labels;
  final List<LanguageOption> languages;
  final List<TranslationServiceOption> services;
  final String sourceLanguage;
  final String targetLanguage;
  final String? detectedLanguage;
  final String? resolvedTargetLanguage;
  final String? selectedServiceId;
  final List<ServiceTranslationResult> results;
  final bool canSwap;
  final bool canTranslate;
  final bool submitting;
  final ValueChanged<String> onSourceLanguageChanged;
  final ValueChanged<String> onTargetLanguageChanged;
  final ValueChanged<String> onServiceSelected;
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

    return Wrap(
      spacing: 8,
      runSpacing: 8,
      crossAxisAlignment: WrapCrossAlignment.center,
      children: [
        SizedBox(
          width: 180,
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
        IconButton(
          key: const ValueKey('swap-languages'),
          tooltip: labels.swapLanguages,
          onPressed: canSwap ? onSwapLanguages : null,
          icon: const Icon(Icons.swap_horiz_rounded),
        ),
        SizedBox(
          width: 180,
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
        if (services.length > 1)
          ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 360),
            child: _ServiceSwitcher(
              labels: labels,
              services: services,
              results: results,
              selectedServiceId: selectedServiceId,
              onServiceSelected: onServiceSelected,
            ),
          ),
        FilledButton.icon(
          key: const ValueKey('translate-button'),
          onPressed: canTranslate ? onTranslate : null,
          icon: submitting
              ? const SizedBox.square(
                  dimension: 14,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : const Icon(Icons.arrow_forward_rounded, size: 16),
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
        inputDecorationTheme: const InputDecorationTheme(
          isDense: true,
          contentPadding: EdgeInsets.symmetric(horizontal: 10, vertical: 8),
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

class _TaskComposer extends StatelessWidget {
  const _TaskComposer({
    required this.labels,
    required this.controller,
    required this.sourceText,
    required this.selectedResult,
    required this.results,
    required this.services,
    required this.selectedServiceId,
    required this.loadingCatalog,
    required this.catalogFailed,
    required this.submissionFailed,
    required this.submitting,
    required this.copied,
    required this.speechAvailable,
    required this.speakingKind,
    required this.dictionaryAvailable,
    required this.doubleClickCopy,
    required this.onChanged,
    required this.onClear,
    required this.onCopy,
    required this.onServiceSelected,
    required this.onRetry,
    required this.onConfigureServices,
    required this.onSpeakSource,
    required this.onSpeakResult,
    required this.onStopSpeech,
    required this.onLookup,
    required this.onSaveVocabulary,
    required this.vocabularySaved,
    required this.savingVocabulary,
    required this.onRecovery,
  });

  final TranslationWorkspaceLabels labels;
  final TextEditingController controller;
  final String sourceText;
  final ServiceTranslationResult? selectedResult;
  final List<ServiceTranslationResult> results;
  final List<TranslationServiceOption> services;
  final String? selectedServiceId;
  final bool loadingCatalog;
  final bool catalogFailed;
  final bool submissionFailed;
  final bool submitting;
  final bool copied;
  final bool speechAvailable;
  final SpeechUtteranceKind? speakingKind;
  final bool dictionaryAvailable;
  final bool doubleClickCopy;
  final ValueChanged<String> onChanged;
  final VoidCallback onClear;
  final ValueChanged<String> onCopy;
  final ValueChanged<String> onServiceSelected;
  final VoidCallback onRetry;
  final VoidCallback onConfigureServices;
  final VoidCallback? onSpeakSource;
  final VoidCallback? onSpeakResult;
  final VoidCallback? onStopSpeech;
  final ValueChanged<String>? onLookup;
  final VoidCallback? onSaveVocabulary;
  final bool vocabularySaved;
  final bool savingVocabulary;
  final ValueChanged<RecoveryAction>? onRecovery;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
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

    return DecoratedBox(
      decoration: BoxDecoration(
        color: theme.colorScheme.surfaceContainerLowest,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: theme.colorScheme.outlineVariant),
      ),
      child: Column(
        children: [
          Expanded(
            flex: 5,
            child: Padding(
              padding: const EdgeInsets.fromLTRB(16, 10, 8, 8),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Row(
                    children: [
                      Text(labels.source, style: theme.textTheme.titleMedium),
                      const Spacer(),
                      Text(
                        labels.characterCount(sourceText.characters.length),
                        style: theme.textTheme.labelMedium?.copyWith(
                          color: theme.colorScheme.onSurfaceVariant,
                        ),
                      ),
                      if (speechAvailable)
                        IconButton(
                          tooltip: speakingKind == SpeechUtteranceKind.source
                              ? labels.stopSpeech
                              : labels.speakSource,
                          onPressed: sourceText.trim().isEmpty
                              ? null
                              : speakingKind == SpeechUtteranceKind.source
                              ? onStopSpeech
                              : onSpeakSource,
                          icon: Icon(
                            speakingKind == SpeechUtteranceKind.source
                                ? Icons.stop_rounded
                                : Icons.volume_up_outlined,
                            size: 18,
                          ),
                        ),
                      IconButton(
                        key: const ValueKey('clear-source'),
                        tooltip: labels.clear,
                        onPressed: sourceText.isEmpty ? null : onClear,
                        icon: const Icon(Icons.close_rounded, size: 18),
                      ),
                    ],
                  ),
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
            ),
          ),
          if (isStreaming) const LinearProgressIndicator(minHeight: 2),
          Container(height: 3, color: context.brandColors.resultRule),
          Expanded(
            flex: 6,
            child: Padding(
              padding: const EdgeInsets.fromLTRB(16, 8, 8, 12),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Row(
                    children: [
                      Container(
                        width: 3,
                        height: 14,
                        color: context.brandColors.resultRule,
                      ),
                      const SizedBox(width: 8),
                      Text(labels.target, style: theme.textTheme.titleMedium),
                      const Spacer(),
                      if (dictionaryAvailable && text.trim().isNotEmpty)
                        IconButton(
                          tooltip: labels.lookup,
                          onPressed: () => onLookup?.call(text),
                          icon: const Icon(Icons.menu_book_outlined, size: 18),
                        ),
                      if (onSaveVocabulary != null && text.trim().isNotEmpty)
                        IconButton(
                          tooltip: vocabularySaved
                              ? labels.savedWord
                              : labels.saveWord,
                          onPressed: savingVocabulary || vocabularySaved
                              ? null
                              : onSaveVocabulary,
                          icon: savingVocabulary
                              ? const SizedBox.square(
                                  dimension: 16,
                                  child: CircularProgressIndicator(
                                    strokeWidth: 2,
                                  ),
                                )
                              : Icon(
                                  vocabularySaved
                                      ? Icons.check_rounded
                                      : Icons.bookmark_add_outlined,
                                  size: 18,
                                ),
                        ),
                      if (speechAvailable)
                        IconButton(
                          tooltip:
                              speakingKind == SpeechUtteranceKind.translation
                              ? labels.stopSpeech
                              : labels.speakResult,
                          onPressed: text.trim().isEmpty
                              ? null
                              : speakingKind == SpeechUtteranceKind.translation
                              ? onStopSpeech
                              : onSpeakResult,
                          icon: Icon(
                            speakingKind == SpeechUtteranceKind.translation
                                ? Icons.stop_rounded
                                : Icons.volume_up_outlined,
                            size: 18,
                          ),
                        ),
                      IconButton(
                        key: const ValueKey('copy-result'),
                        tooltip: copied ? labels.copied : labels.copy,
                        onPressed: text.trim().isEmpty
                            ? null
                            : () => onCopy(text),
                        icon: Icon(
                          copied ? Icons.check_rounded : Icons.copy_rounded,
                          size: 18,
                        ),
                      ),
                    ],
                  ),
                  if (failedCount > 0 && completedCount > 0)
                    Padding(
                      padding: const EdgeInsets.only(top: 4, bottom: 4),
                      child: Text(
                        labels.partialFailure(failedCount),
                        style: theme.textTheme.labelMedium?.copyWith(
                          color: theme.colorScheme.error,
                        ),
                      ),
                    ),
                  Expanded(
                    child: GestureDetector(
                      onDoubleTap: doubleClickCopy && text.trim().isNotEmpty
                          ? () => onCopy(text)
                          : null,
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
                        onRecovery: onRecovery,
                      ),
                    ),
                  ),
                ],
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
    required this.onRecovery,
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
  final ValueChanged<RecoveryAction>? onRecovery;

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
      final failure = mapFailure(failureCode);
      return StatusMessage(
        kind: StatusKind.error,
        title: labels.failureMessage(failure.wireName),
        action: Wrap(
          spacing: 8,
          children: [
            OutlinedButton.icon(
              onPressed: onRetry,
              icon: const Icon(Icons.refresh_rounded, size: 16),
              label: Text(labels.retry),
            ),
            if (failure.recovery != RecoveryAction.none &&
                failure.recovery != RecoveryAction.retry)
              TextButton(
                onPressed: () => onRecovery?.call(failure.recovery),
                child: Text(_recoveryLabel(failure.recovery)),
              ),
          ],
        ),
      );
    }
    return StatusMessage(
      kind: isTranslating ? StatusKind.progress : StatusKind.info,
      title: isTranslating ? labels.streaming : labels.empty,
    );
  }

  String _recoveryLabel(RecoveryAction action) {
    return switch (action) {
      RecoveryAction.openPermissionSettings => labels.recoveryPermissions,
      RecoveryAction.recheckPermission => labels.recoveryRecheck,
      RecoveryAction.configureOcr => labels.recoveryConfigureOcr,
      RecoveryAction.configureTranslationProvider =>
        labels.recoveryConfigureProvider,
      RecoveryAction.editInput => labels.recoveryEditInput,
      RecoveryAction.chooseLanguage => labels.recoveryChooseLanguage,
      RecoveryAction.switchToGoogleWeb => labels.recoverySwitchToGoogleWeb,
      _ => labels.retry,
    };
  }
}

class _AsidePanel extends StatelessWidget {
  const _AsidePanel({
    required this.labels,
    required this.matches,
    required this.warnings,
  });

  final TranslationWorkspaceLabels labels;
  final List<GlossaryMatchHit> matches;
  final List<GlossaryComplianceWarning> warnings;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return DecoratedBox(
      decoration: BoxDecoration(
        color: theme.colorScheme.surfaceContainerLow,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: theme.colorScheme.outlineVariant),
      ),
      child: ListView(
        padding: const EdgeInsets.fromLTRB(14, 12, 14, 16),
        children: [
          Text(labels.glossaryHits, style: theme.textTheme.titleMedium),
          const SizedBox(height: 8),
          if (matches.isEmpty)
            Text(
              labels.glossaryEmpty,
              style: theme.textTheme.bodyMedium?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            )
          else
            for (final match in matches)
              Padding(
                padding: const EdgeInsets.only(bottom: 8),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(match.term, style: theme.textTheme.titleMedium),
                    Text(match.translation, style: theme.textTheme.bodyMedium),
                  ],
                ),
              ),
          if (warnings.isNotEmpty) ...[
            const SizedBox(height: 12),
            Text(labels.glossaryWarning, style: theme.textTheme.titleMedium),
            const SizedBox(height: 8),
            for (final warning in warnings)
              Padding(
                padding: const EdgeInsets.only(bottom: 8),
                child: Text(
                  warning.kind == GlossaryIssueKind.forbiddenUsed
                      ? '${warning.term} → ${warning.found}'
                      : '${warning.term} → ${warning.expected}',
                  style: theme.textTheme.bodyMedium?.copyWith(
                    color: theme.colorScheme.error,
                  ),
                ),
              ),
          ],
        ],
      ),
    );
  }
}
