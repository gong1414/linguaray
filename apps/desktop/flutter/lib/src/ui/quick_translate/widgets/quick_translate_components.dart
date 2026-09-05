import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_ui/linguaray_ui.dart' show BrandLogo;

import '../../shared/status_message.dart';
import 'quick_translate_models.dart';

class QuickTranslateGlossaryMatches extends StatelessWidget {
  const QuickTranslateGlossaryMatches({
    required this.label,
    required this.matches,
    super.key,
  });

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

class QuickTranslateCommandHeader extends StatelessWidget {
  const QuickTranslateCommandHeader({
    required this.labels,
    required this.pinned,
    required this.languages,
    required this.sourceLanguage,
    required this.targetLanguage,
    this.onClose,
    this.onStartDragging,
    this.menuItems = const [],
    this.onMenuSelected,
    required this.onTogglePin,
    required this.onCapture,
    required this.onClipboard,
    required this.onOpenSettings,
    required this.onSourceLanguageChanged,
    required this.onTargetLanguageChanged,
    required this.onSwapLanguages,
    super.key,
  });

  final QuickTranslateLabels labels;
  final bool pinned;
  final List<LanguageOption> languages;
  final String sourceLanguage;
  final String targetLanguage;
  final VoidCallback? onClose;
  final VoidCallback? onStartDragging;
  final List<PopupMenuEntry<String>> menuItems;
  final ValueChanged<String>? onMenuSelected;
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
            const BrandLogo(size: 18),
            const SizedBox(width: 8),
            Expanded(
              child: GestureDetector(
                behavior: HitTestBehavior.opaque,
                onPanStart: (_) => onStartDragging?.call(),
                child: Padding(
                  padding: const EdgeInsets.symmetric(vertical: 8),
                  child: Text(
                    labels.title,
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                ),
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
            if (onClose != null)
              IconButton(
                tooltip: labels.close,
                onPressed: onClose,
                icon: const Icon(Icons.close_rounded, size: 18),
              ),
            PopupMenuButton<String>(
              tooltip: labels.openSettings,
              onSelected: (value) {
                switch (value) {
                  case 'settings':
                    onOpenSettings();
                  default:
                    onMenuSelected?.call(value);
                }
              },
              itemBuilder: (context) => [
                ...menuItems,
                PopupMenuItem(
                  value: 'settings',
                  child: Text(labels.openSettings),
                ),
              ],
            ),
          ],
        ),
        const SizedBox(height: 8),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 2),
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surfaceContainerLow,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Row(
            children: [
              Expanded(
                child: DropdownButtonHideUnderline(
                  child: DropdownButton<String>(
                    value:
                        sourceItems.any((item) => item.code == sourceLanguage)
                        ? sourceLanguage
                        : autoLanguageCode,
                    isExpanded: true,
                    isDense: true,
                    style: Theme.of(context).textTheme.bodyMedium,
                    icon: const Icon(
                      Icons.keyboard_arrow_down_rounded,
                      size: 16,
                    ),
                    items: [
                      for (final item in sourceItems)
                        DropdownMenuItem(
                          value: item.code,
                          child: Text(
                            item.name,
                            overflow: TextOverflow.ellipsis,
                          ),
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
                    value:
                        targetItems.any((item) => item.code == targetLanguage)
                        ? targetLanguage
                        : automaticTargetCode,
                    isExpanded: true,
                    isDense: true,
                    style: Theme.of(context).textTheme.bodyMedium,
                    icon: const Icon(
                      Icons.keyboard_arrow_down_rounded,
                      size: 16,
                    ),
                    items: [
                      for (final item in targetItems)
                        DropdownMenuItem(
                          value: item.code,
                          child: Text(
                            item.name,
                            overflow: TextOverflow.ellipsis,
                          ),
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
        ),
      ],
    );
  }
}

class QuickTranslateNoticeMessage extends StatelessWidget {
  const QuickTranslateNoticeMessage({
    required this.labels,
    required this.notice,
    required this.onRecheck,
    required this.onConfigureOcr,
    required this.onConfigureServices,
    required this.onRetryCapture,
    super.key,
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
