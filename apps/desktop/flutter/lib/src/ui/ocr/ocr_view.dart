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
    return Scaffold(
      body: SafeArea(
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(12, 10, 8, 8),
              child: Row(
                children: [
                  Text(
                    labels.title,
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                  if (state.results.isNotEmpty) ...[
                    const SizedBox(width: 8),
                    Text(
                      labels.resultCount(state.results.length),
                      style: Theme.of(context).textTheme.labelMedium,
                    ),
                  ],
                  const Spacer(),
                  IconButton(
                    tooltip: labels.close,
                    onPressed: widget.onClose,
                    icon: const Icon(Icons.close_rounded),
                  ),
                ],
              ),
            ),
            const Divider(height: 1),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              child: Wrap(
                spacing: 6,
                runSpacing: 6,
                crossAxisAlignment: WrapCrossAlignment.center,
                children: [
                  FilledButton.tonalIcon(
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
                  IconButton(
                    tooltip: labels.copy,
                    onPressed: state.text.trim().isEmpty ? null : widget.onCopy,
                    icon: const Icon(Icons.copy_rounded),
                  ),
                  IconButton(
                    tooltip: labels.clear,
                    onPressed: state.text.isEmpty ? null : widget.onClear,
                    icon: const Icon(Icons.delete_outline_rounded),
                  ),
                ],
              ),
            ),
            if (state.busy) const LinearProgressIndicator(minHeight: 2),
            if (state.errorCode != null)
              Padding(
                padding: const EdgeInsets.fromLTRB(12, 0, 12, 8),
                child: StatusMessage(
                  kind: StatusKind.error,
                  title: labels.errorMessage(state.errorCode),
                ),
              ),
            Expanded(
              child: state.text.isEmpty && !state.busy
                  ? Center(
                      child: StatusMessage(
                        kind: StatusKind.info,
                        title: labels.emptyTitle,
                        body: labels.emptyDescription,
                      ),
                    )
                  : Padding(
                      padding: const EdgeInsets.fromLTRB(12, 0, 12, 12),
                      child: TextField(
                        controller: _controller,
                        expands: true,
                        minLines: null,
                        maxLines: null,
                        textAlignVertical: TextAlignVertical.top,
                        onChanged: widget.onTextChanged,
                        decoration: InputDecoration(
                          hintText: labels.emptyTitle,
                          border: const OutlineInputBorder(),
                        ),
                      ),
                    ),
            ),
          ],
        ),
      ),
    );
  }
}
