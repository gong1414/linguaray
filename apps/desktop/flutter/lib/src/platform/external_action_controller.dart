import 'dart:async';

import 'package:linguaray_application/linguaray_application.dart';

import '../services/app_windows.dart';
import '../services/runtime.dart';
import 'platform_types.dart';
import 'trigger_controller.dart';

/// Dispatches local API and deep-link commands through the same typed
/// controller boundary as tray items and global shortcuts.
class ExternalActionController {
  ExternalActionController({TriggerController? triggers})
    : _triggers = triggers ?? triggerController;

  final TriggerController _triggers;
  bool _started = false;

  void start() {
    if (_started) return;
    _started = true;
    unawaited(_listen());
  }

  Future<void> dispatchProtocol(ProtocolCommand command) async {
    switch (command.action) {
      case ProtocolAction.translate:
        final text = command.text?.trim();
        if (text == null || text.isEmpty) return;
        await _translateText(text);
      case ProtocolAction.translateSelection:
        await _triggers.trigger(TriggerAction.translateSelection);
      case ProtocolAction.translateInput:
        await _triggers.openInputWindow();
      case ProtocolAction.translateClipboard:
        await _triggers.trigger(TriggerAction.translateInput);
      case ProtocolAction.captureTranslate:
        await _triggers.trigger(TriggerAction.captureAndTranslate);
      case ProtocolAction.captureOcr:
        await _triggers.trigger(TriggerAction.captureOcr);
      case ProtocolAction.clipboardOcr:
        await _triggers.trigger(TriggerAction.clipboardOcr);
      case ProtocolAction.showTranslationWindow:
        await _triggers.showTranslationWindow();
      case ProtocolAction.showOcrWindow:
        await showOcrWindow(position: ocrWindowPositionNearCursor());
      case ProtocolAction.settings:
        showSettingsWindow();
      case ProtocolAction.ignored:
        break;
    }
  }

  Future<void> _listen() async {
    final subscription = runtime.subscribeActions();
    while (_started) {
      final request = await subscription.next();
      if (request == null) return;
      try {
        await _dispatchRuntime(request);
      } catch (_) {
        // One failed platform action must not detach the process-wide API
        // listener. TriggerController exposes expected failures in lastError.
      }
    }
  }

  Future<void> _dispatchRuntime(ExternalActionRequest request) async {
    switch (request.kind) {
      case ExternalActionKind.translateText:
        final text = request.text?.trim();
        if (text == null || text.isEmpty) return;
        await _translateText(text);
      case ExternalActionKind.translateSelection:
        await _triggers.trigger(TriggerAction.translateSelection);
      case ExternalActionKind.translateInput:
        await _triggers.openInputWindow();
      case ExternalActionKind.translateClipboard:
        await _triggers.trigger(TriggerAction.translateInput);
      case ExternalActionKind.captureTranslate:
        await _triggers.trigger(TriggerAction.captureAndTranslate);
      case ExternalActionKind.captureOcr:
        await _triggers.trigger(TriggerAction.captureOcr);
      case ExternalActionKind.clipboardOcr:
        await _triggers.trigger(TriggerAction.clipboardOcr);
      case ExternalActionKind.showTranslationWindow:
        await _triggers.showTranslationWindow();
      case ExternalActionKind.showOcrWindow:
        await showOcrWindow(position: ocrWindowPositionNearCursor());
      case ExternalActionKind.openSettings:
        showSettingsWindow();
    }
  }

  Future<void> _translateText(String text) async {
    _triggers.quickWindowRequest.value = QuickWindowRequest(
      text: text,
      submit: true,
      clearExisting: true,
    );
    await showMiniTranslatorWindow(
      position: miniTranslatorPositionNearCursor(),
    );
  }
}

final externalActionController = ExternalActionController();
