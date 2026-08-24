import 'dart:convert';

import 'package:linguaray_application/src/errors/models.dart';
import 'package:linguaray_application/src/protocol/models.dart';

final class ParseProtocolLink {
  const ParseProtocolLink();

  ProtocolCommand call(String raw) {
    final uri = Uri.tryParse(raw.trim());
    if (uri == null) {
      return ProtocolCommand.ignored(AppErrorCode.protocolInvalid.wireName);
    }
    if (uri.scheme.toLowerCase() != 'linguaray') {
      return ProtocolCommand.ignored(AppErrorCode.protocolInvalid.wireName);
    }

    final action = uri.host.isNotEmpty
        ? uri.host.toLowerCase()
        : (uri.pathSegments.isEmpty
              ? ''
              : uri.pathSegments.first.toLowerCase());

    switch (action) {
      case 'settings':
        return const ProtocolCommand.settings();
      case 'selection-translate':
        return const ProtocolCommand.action(ProtocolAction.translateSelection);
      case 'input-translate':
        return const ProtocolCommand.action(ProtocolAction.translateInput);
      case 'clipboard-translate':
        return const ProtocolCommand.action(ProtocolAction.translateClipboard);
      case 'capture-translate':
        return const ProtocolCommand.action(ProtocolAction.captureTranslate);
      case 'capture-ocr':
        return const ProtocolCommand.action(ProtocolAction.captureOcr);
      case 'clipboard-ocr':
        return const ProtocolCommand.action(ProtocolAction.clipboardOcr);
      case 'show-translation':
        return const ProtocolCommand.action(
          ProtocolAction.showTranslationWindow,
        );
      case 'show-ocr':
        return const ProtocolCommand.action(ProtocolAction.showOcrWindow);
      case 'translate':
        final text = uri.queryParameters['text'] ?? '';
        if (text.isEmpty) {
          return ProtocolCommand.ignored(AppErrorCode.protocolInvalid.wireName);
        }
        final bytes = utf8.encode(text).length;
        if (bytes > kProtocolMaxTextBytes) {
          return ProtocolCommand.ignored(
            AppErrorCode.protocolTooLarge.wireName,
          );
        }
        return ProtocolCommand.translate(text);
      default:
        return const ProtocolCommand.ignored();
    }
  }
}
