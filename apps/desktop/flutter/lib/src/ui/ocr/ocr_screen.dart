import 'dart:async';

import 'package:flutter/material.dart';
import 'package:nativeapi/nativeapi.dart' as nativeapi;

import '../../i18n/i18n.dart';
import '../../platform/ocr_controller.dart';
import '../../platform/platform_types.dart';
import '../../platform/trigger_controller.dart';
import '../../services/app_windows.dart';
import '../i18n_labels.dart';
import 'ocr_view.dart';

class OcrScreen extends StatefulWidget {
  const OcrScreen({super.key});

  @override
  State<OcrScreen> createState() => _OcrScreenState();
}

class _OcrScreenState extends State<OcrScreen> {
  int? _windowBlurredListenerId;

  @override
  void initState() {
    super.initState();
    ocrController.addListener(_onStateChanged);
    _windowBlurredListenerId = nativeapi.WindowManager.instance
        .on<nativeapi.WindowBlurredEvent>((event) {
          if (event.windowId == ocrWindowController.window.id &&
              !ocrController.state.busy) {
            hideOcrWindow();
          }
        });
  }

  @override
  void dispose() {
    ocrController.removeListener(_onStateChanged);
    if (_windowBlurredListenerId != null) {
      nativeapi.WindowManager.instance.off(_windowBlurredListenerId!);
    }
    super.dispose();
  }

  void _onStateChanged() {
    if (mounted) setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    final labels = t.ui.ocr;
    return OcrView(
      labels: OcrViewLabels(
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
      onClose: hideOcrWindow,
    );
  }
}
