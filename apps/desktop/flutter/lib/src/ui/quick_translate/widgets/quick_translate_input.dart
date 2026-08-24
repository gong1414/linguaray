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
    return TextField(
      key: const ValueKey('quick-source-input'),
      controller: controller,
      autofocus: true,
      minLines: 1,
      maxLines: 4,
      textInputAction: submitWithModifier
          ? TextInputAction.newline
          : TextInputAction.go,
      onChanged: onSourceTextChanged,
      decoration: InputDecoration(
        hintText: labels.inputHint,
        suffixIcon: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            if (sourceText.isNotEmpty)
              IconButton(
                tooltip: labels.clear,
                onPressed: onClear,
                icon: const Icon(Icons.close_rounded, size: 18),
              ),
            if (speechAvailable && onSpeakSource != null)
              IconButton(
                tooltip: speaking ? labels.stopSpeaking : labels.speakSource,
                onPressed: speaking ? onStopSpeech : onSpeakSource,
                icon: Icon(
                  speaking
                      ? Icons.stop_circle_outlined
                      : Icons.volume_up_outlined,
                  size: 18,
                ),
              ),
            IconButton(
              tooltip: submitting ? labels.translating : labels.translate,
              onPressed: canTranslate ? onTranslate : null,
              icon: submitting
                  ? const SizedBox.square(
                      dimension: 14,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Icon(Icons.arrow_forward_rounded),
            ),
          ],
        ),
      ),
    );
  }
}
