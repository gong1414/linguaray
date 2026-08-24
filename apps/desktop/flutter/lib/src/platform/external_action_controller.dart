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
    final kind = _externalKindFor(command.action);
    if (kind == null) return;
    await _dispatch(kind, command.text);
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
    await _dispatch(request.kind, request.text);
  }

  Future<void> _dispatch(ExternalActionKind kind, String? rawText) async {
    switch (kind) {
      case ExternalActionKind.translateText:
        final text = rawText?.trim();
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

ExternalActionKind? _externalKindFor(ProtocolAction action) => switch (action) {
  ProtocolAction.translate => ExternalActionKind.translateText,
  ProtocolAction.translateSelection => ExternalActionKind.translateSelection,
  ProtocolAction.translateInput => ExternalActionKind.translateInput,
  ProtocolAction.translateClipboard => ExternalActionKind.translateClipboard,
  ProtocolAction.captureTranslate => ExternalActionKind.captureTranslate,
  ProtocolAction.captureOcr => ExternalActionKind.captureOcr,
  ProtocolAction.clipboardOcr => ExternalActionKind.clipboardOcr,
  ProtocolAction.showTranslationWindow =>
    ExternalActionKind.showTranslationWindow,
  ProtocolAction.showOcrWindow => ExternalActionKind.showOcrWindow,
  ProtocolAction.settings => ExternalActionKind.openSettings,
  ProtocolAction.ignored => null,
};

final externalActionController = ExternalActionController();
