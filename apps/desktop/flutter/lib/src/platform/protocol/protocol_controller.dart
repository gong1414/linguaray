import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../app/windows/app_windows.dart';

/// Receives `linguaray://` links from the host and routes them without
/// logging the translated text.
class ProtocolController {
  ProtocolController({
    MethodChannel? channel,
    this.parse = const ParseProtocolLink(),
  }) : _channel = channel ?? const MethodChannel('linguaray/protocol');

  final MethodChannel _channel;
  final ParseProtocolLink parse;
  final ValueNotifier<ProtocolCommand?> lastCommand = ValueNotifier(null);
  void Function(String text)? onTranslate;
  void Function()? onOpenSettings;
  Future<void> Function(ProtocolCommand command)? onCommand;

  void start() {
    _channel.setMethodCallHandler(_handle);
  }

  Future<void> handleRaw(String raw) async {
    final command = parse(raw);
    lastCommand.value = command;
    final commandHandler = onCommand;
    if (commandHandler != null && command.action != ProtocolAction.ignored) {
      await commandHandler(command);
      return;
    }
    switch (command.action) {
      case ProtocolAction.translate:
        final text = command.text;
        if (text == null || text.isEmpty) return;
        onTranslate?.call(text);
        try {
          await showMiniTranslatorWindow(
            position: miniTranslatorPositionNearCursor(),
          );
        } catch (_) {}
      case ProtocolAction.settings:
        final handler = onOpenSettings;
        if (handler != null) {
          handler();
        } else {
          try {
            showSettingsWindow();
          } catch (_) {}
        }
      case ProtocolAction.translateSelection ||
          ProtocolAction.translateInput ||
          ProtocolAction.translateClipboard ||
          ProtocolAction.captureTranslate ||
          ProtocolAction.captureOcr ||
          ProtocolAction.clipboardOcr ||
          ProtocolAction.showTranslationWindow ||
          ProtocolAction.showOcrWindow:
        break;
      case ProtocolAction.ignored:
        break;
    }
  }

  Future<dynamic> _handle(MethodCall call) async {
    if (call.method == 'open') {
      final raw = call.arguments as String? ?? '';
      await handleRaw(raw);
    }
  }
}

final protocolController = ProtocolController();
