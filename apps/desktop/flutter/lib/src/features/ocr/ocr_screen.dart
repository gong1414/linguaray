import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../app/commands/trigger_controller.dart';
import '../../app/dependencies.dart';
import '../../i18n/i18n.dart';
import '../../platform/platform_types.dart';
import '../../shared/i18n_labels.dart';
import 'ocr_controller.dart';
import 'ocr_view.dart';
import 'ocr_window_coordinator.dart';

class OcrScreen extends ConsumerStatefulWidget {
  const OcrScreen({super.key});

  @override
  ConsumerState<OcrScreen> createState() => _OcrScreenState();
}

class _OcrScreenState extends ConsumerState<OcrScreen> {
  late final OcrController _ocr;
  late final TriggerController _triggers;
  late final OcrWindowCoordinator _window;
  bool _pinned = false;

  @override
  void initState() {
    super.initState();
    _ocr = ref.read(ocrControllerProvider);
    _triggers = ref.read(triggerControllerProvider);
    _window = OcrWindowCoordinator(
      keepVisible: () => _ocr.state.busy || _pinned,
    );
    _window.registerEvents();
  }

  @override
  void dispose() {
    _window.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final labels = t.ui.ocr;
    return ListenableBuilder(
      listenable: _ocr,
      builder: (context, _) => OcrView(
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
        state: _ocr.state,
        onTextChanged: _ocr.setText,
        onCapture: () => unawaited(_triggers.trigger(TriggerAction.captureOcr)),
        onFile: () => unawaited(_triggers.trigger(TriggerAction.fileOcr)),
        onClipboard: () =>
            unawaited(_triggers.trigger(TriggerAction.clipboardOcr)),
        onContinuousChanged: _ocr.setContinuous,
        onCopy: () => unawaited(_ocr.copy()),
        onClear: _ocr.clear,
        onClose: _window.close,
      ),
    );
  }
}
