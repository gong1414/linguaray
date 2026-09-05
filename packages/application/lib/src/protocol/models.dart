enum ProtocolAction {
  translate,
  translateSelection,
  translateInput,
  translateClipboard,
  captureTranslate,
  captureOcr,
  clipboardOcr,
  showTranslationWindow,
  showOcrWindow,
  settings,
  ignored,
}

/// Maximum decoded `text` payload accepted from a `linguaray://` URL.
const int kProtocolMaxTextBytes = 32 * 1024;

final class ProtocolCommand {
  const ProtocolCommand({required this.action, this.text, this.errorCode});

  const ProtocolCommand.translate(this.text)
    : action = ProtocolAction.translate,
      errorCode = null;

  const ProtocolCommand.settings()
    : action = ProtocolAction.settings,
      text = null,
      errorCode = null;

  const ProtocolCommand.action(this.action) : text = null, errorCode = null;

  const ProtocolCommand.ignored([this.errorCode])
    : action = ProtocolAction.ignored,
      text = null;

  final ProtocolAction action;
  final String? text;
  final String? errorCode;
}
