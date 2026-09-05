import 'dart:async';

import 'package:flutter/material.dart';

import '../../app/commands/trigger_controller.dart';
import '../../i18n/i18n.dart';
import '../../platform/platform_types.dart';
import '../../shared/i18n_labels.dart';
import 'ocr_controller.dart';
import 'ocr_view.dart';
import 'ocr_window_coordinator.dart';

class OcrScreen extends StatefulWidget {
  const OcrScreen({super.key});

  @override
  State<OcrScreen> createState() => _OcrScreenState();
}

class _OcrScreenState extends State<OcrScreen> {
  late final OcrWindowCoordinator _window;
  bool _pinned = false;

  @override
  void initState() {
    super.initState();
    ocrController.addListener(_onStateChanged);
    _window = OcrWindowCoordinator(
      keepVisible: () => ocrController.state.busy || _pinned,
    );
    _window.registerEvents();
  }

  @override
  void dispose() {
    ocrController.removeListener(_onStateChanged);
    _window.dispose();
    super.dispose();
  }

  void _onStateChanged() {
    if (mounted) setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    final labels = t.ui.ocr;
    return OcrView(
      pinned: _pinned,
      onTogglePin: () {
        setState(() => _pinned = !_pinned);
        _window.setPinned(_pinned);
      },
      labels: OcrViewLabels(
        pin: t.ui.quick.pin,
        unpin: t.ui.quick.unpin,
        title: labels.title,
        emptyTitle: labels.empty_title,
        emptyDescription: labels.empty_description,
        capture: labels.capture,
        file: labels.file,
        clipboard: labels.clipboard,
        continuous: labels.continuous,
        copy: t.mini_translator.button.copy,
        clear: t.mini_translator.button.clear,
        close: t.ui.shell.close,
        resultCount: (count) => labels.result_count(count: count),
        errorMessage: appErrorMessage,
      ),
      state: ocrController.state,
      onTextChanged: ocrController.setText,
      onCapture: () =>
          unawaited(triggerController.trigger(TriggerAction.captureOcr)),
      onFile: () => unawaited(triggerController.trigger(TriggerAction.fileOcr)),
      onClipboard: () =>
          unawaited(triggerController.trigger(TriggerAction.clipboardOcr)),
      onContinuousChanged: ocrController.setContinuous,
      onCopy: () => unawaited(ocrController.copy()),
      onClear: ocrController.clear,
      onClose: _window.close,
    );
  }
}
