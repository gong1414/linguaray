import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';

import 'quick_translate_models.dart';

class QuickTranslateInput extends StatelessWidget {
  const QuickTranslateInput({
    required this.labels,
    required this.controller,
    required this.sourceText,
    required this.submitting,
    required this.speechAvailable,
    required this.submitWithModifier,
    required this.canTranslate,
    required this.onSourceTextChanged,
    required this.onClear,
    required this.onTranslate,
    super.key,
    this.speakingKind,
    this.onSpeakSource,
    this.onStopSpeech,
  });

  final QuickTranslateLabels labels;
  final TextEditingController controller;
  final String sourceText;
  final bool submitting;
  final bool speechAvailable;
  final bool submitWithModifier;
  final bool canTranslate;
  final SpeechUtteranceKind? speakingKind;
  final ValueChanged<String> onSourceTextChanged;
  final VoidCallback onClear;
  final VoidCallback onTranslate;
  final VoidCallback? onSpeakSource;
  final VoidCallback? onStopSpeech;

  @override
  Widget build(BuildContext context) {
    final speaking = speakingKind == SpeechUtteranceKind.source;
    final theme = Theme.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        SizedBox(
          height: 32,
          child: Row(
            children: [
              Text(labels.sourceLabel, style: theme.textTheme.labelMedium),
              const Spacer(),
              if (sourceText.isNotEmpty)
                IconButton(
                  tooltip: labels.clear,
                  onPressed: onClear,
                  icon: const Icon(Icons.close_rounded, size: 16),
                ),
              if (speechAvailable && onSpeakSource != null)
                IconButton(
                  tooltip: speaking ? labels.stopSpeaking : labels.speakSource,
                  onPressed: speaking ? onStopSpeech : onSpeakSource,
                  icon: Icon(
                    speaking
                        ? Icons.stop_circle_outlined
                        : Icons.volume_up_outlined,
                    size: 17,
                  ),
                ),
            ],
          ),
        ),
        TextField(
          key: const ValueKey('quick-source-input'),
          controller: controller,
          autofocus: true,
          minLines: 5,
          maxLines: 7,
          style: theme.textTheme.bodyLarge?.copyWith(
            fontSize: 16,
            height: 1.65,
          ),
          textInputAction: submitWithModifier
              ? TextInputAction.newline
              : TextInputAction.go,
          onChanged: onSourceTextChanged,
          decoration: InputDecoration(
            hintText: labels.inputHint,
            filled: false,
            border: InputBorder.none,
            enabledBorder: InputBorder.none,
            focusedBorder: InputBorder.none,
            contentPadding: const EdgeInsets.symmetric(vertical: 8),
            hintStyle: theme.textTheme.bodyLarge?.copyWith(
              fontSize: 16,
              height: 1.65,
              color: theme.colorScheme.onSurfaceVariant,
            ),
          ),
        ),
        const SizedBox(height: 4),
        Row(
          mainAxisAlignment: MainAxisAlignment.end,
          children: [
            Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                if (submitWithModifier) ...[
                  Text(
                    theme.platform == TargetPlatform.macOS ? '⌘' : 'Ctrl',
                    style: theme.textTheme.labelSmall,
                  ),
                  const SizedBox(width: 3),
                ],
                Icon(
                  Icons.keyboard_return_rounded,
                  size: 13,
                  color: theme.colorScheme.onSurfaceVariant,
                ),
              ],
            ),
            const SizedBox(width: 10),
            FilledButton(
              onPressed: canTranslate ? onTranslate : null,
              style: FilledButton.styleFrom(
                minimumSize: const Size(0, 30),
                padding: const EdgeInsets.symmetric(horizontal: 12),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(submitting ? labels.translating : labels.translate),
                  const SizedBox(width: 8),
                  if (submitting)
                    const SizedBox.square(
                      dimension: 13,
                      child: CircularProgressIndicator(strokeWidth: 1.5),
                    )
                  else
                    const Icon(Icons.arrow_forward_rounded, size: 15),
                ],
              ),
            ),
          ],
        ),
      ],
    );
  }
}
