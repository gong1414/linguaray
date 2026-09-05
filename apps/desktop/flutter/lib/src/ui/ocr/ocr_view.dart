import 'package:flutter/material.dart';

import '../../platform/ocr_controller.dart';
import '../shared/status_message.dart';

final class OcrViewLabels {
  const OcrViewLabels({
    required this.title,
    required this.emptyTitle,
    required this.emptyDescription,
    required this.capture,
    required this.file,
    required this.clipboard,
    required this.continuous,
    required this.copy,
    required this.clear,
    required this.close,
    required this.resultCount,
    required this.errorMessage,
  });

  final String title;
  final String emptyTitle;
  final String emptyDescription;
  final String capture;
  final String file;
  final String clipboard;
  final String continuous;
  final String copy;
  final String clear;
  final String close;
  final String Function(int count) resultCount;
  final String Function(String? code) errorMessage;
}

class OcrView extends StatefulWidget {
  const OcrView({
    required this.labels,
    required this.state,
    required this.onTextChanged,
    required this.onCapture,
    required this.onFile,
    required this.onClipboard,
    required this.onContinuousChanged,
    required this.onCopy,
    required this.onClear,
    required this.onClose,
    super.key,
  });

  final OcrViewLabels labels;
  final OcrViewState state;
  final ValueChanged<String> onTextChanged;
  final VoidCallback onCapture;
  final VoidCallback onFile;
  final VoidCallback onClipboard;
  final ValueChanged<bool> onContinuousChanged;
  final VoidCallback onCopy;
  final VoidCallback onClear;
  final VoidCallback onClose;

  @override
  State<OcrView> createState() => _OcrViewState();
}

class _OcrViewState extends State<OcrView> {
  late final TextEditingController _controller = TextEditingController(
    text: widget.state.text,
  );

  @override
  void didUpdateWidget(covariant OcrView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (_controller.text != widget.state.text) {
      _controller
        ..text = widget.state.text
        ..selection = TextSelection.collapsed(offset: widget.state.text.length);
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final state = widget.state;
    final labels = widget.labels;
    final theme = Theme.of(context);
    final sources = <Widget>[
      FilledButton.icon(
        onPressed: state.busy ? null : widget.onCapture,
        icon: const Icon(Icons.crop_free_rounded, size: 18),
        label: Text(labels.capture),
      ),
      OutlinedButton.icon(
        onPressed: state.busy ? null : widget.onFile,
        icon: const Icon(Icons.image_outlined, size: 18),
        label: Text(labels.file),
      ),
      OutlinedButton.icon(
        onPressed: state.busy ? null : widget.onClipboard,
        icon: const Icon(Icons.content_paste_rounded, size: 18),
        label: Text(labels.clipboard),
      ),
      FilterChip(
        selected: state.continuous,
        avatar: const Icon(Icons.playlist_add_rounded, size: 17),
        label: Text(labels.continuous),
        onSelected: state.busy ? null : widget.onContinuousChanged,
      ),
    ];
    final editor = Padding(
      padding: const EdgeInsets.all(24),
      child: state.text.isEmpty && !state.busy
          ? Center(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(
                    Icons.text_snippet_outlined,
                    size: 40,
                    color: theme.colorScheme.outline,
                  ),
                  const SizedBox(height: 16),
                  Text(labels.emptyTitle, style: theme.textTheme.titleMedium),
                  const SizedBox(height: 8),
                  Text(
                    labels.emptyDescription,
                    textAlign: TextAlign.center,
                    style: theme.textTheme.bodySmall,
                  ),
                ],
              ),
            )
          : TextField(
              controller: _controller,
              expands: true,
              minLines: null,
              maxLines: null,
              textAlignVertical: TextAlignVertical.top,
              onChanged: widget.onTextChanged,
              style: theme.textTheme.bodyLarge?.copyWith(
                fontSize: 17,
                height: 1.7,
              ),
              decoration: InputDecoration(
                hintText: labels.emptyTitle,
                filled: false,
                contentPadding: EdgeInsets.zero,
                border: InputBorder.none,
                enabledBorder: InputBorder.none,
                focusedBorder: InputBorder.none,
              ),
            ),
    );
    return Scaffold(
      backgroundColor: theme.colorScheme.surfaceContainerLowest,
      body: SafeArea(
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(24, 18, 12, 18),
              child: Row(
                children: [
                  Text(labels.title, style: theme.textTheme.titleLarge),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Text(
                      state.results.isNotEmpty
                          ? labels.resultCount(state.results.length)
                          : '',
                      style: theme.textTheme.labelMedium,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                  IconButton(
                    tooltip: labels.copy,
                    onPressed: state.text.trim().isEmpty ? null : widget.onCopy,
                    icon: const Icon(Icons.copy_rounded, size: 18),
                  ),
                  IconButton(
                    tooltip: labels.clear,
                    onPressed: state.text.isEmpty ? null : widget.onClear,
                    icon: const Icon(Icons.delete_outline_rounded, size: 18),
                  ),
                  IconButton(
                    tooltip: labels.close,
                    onPressed: widget.onClose,
                    icon: const Icon(Icons.close_rounded, size: 18),
                  ),
                ],
              ),
            ),
            const Divider(),
            if (state.busy) const LinearProgressIndicator(minHeight: 2),
            if (state.errorCode != null)
              Padding(
                padding: const EdgeInsets.all(12),
                child: StatusMessage(
                  kind: StatusKind.error,
                  title: labels.errorMessage(state.errorCode),
                ),
              ),
            Expanded(
              child: LayoutBuilder(
                builder: (context, constraints) {
                  if (constraints.maxWidth < 560) {
                    return Column(
                      children: [
                        Padding(
                          padding: const EdgeInsets.all(12),
                          child: Wrap(
                            spacing: 8,
                            runSpacing: 6,
                            children: sources,
                          ),
                        ),
                        Expanded(child: editor),
                      ],
                    );
                  }
                  return Row(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      Container(
                        width: 166,
                        color: theme.colorScheme.surface,
                        child: ListView(
                          padding: const EdgeInsets.all(16),
                          children: [
                            for (final source in sources)
                              Padding(
                                padding: const EdgeInsets.only(bottom: 12),
                                child: source,
                              ),
                          ],
                        ),
                      ),
                      Expanded(child: editor),
                    ],
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }
}
