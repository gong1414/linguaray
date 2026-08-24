import 'package:flutter/widgets.dart' show Rect;
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/platform/external_action_controller.dart';
import 'package:linguaray_desktop/src/platform/platform_types.dart';
import 'package:linguaray_desktop/src/platform/trigger_controller.dart';

void main() {
  test('protocol actions share the typed runtime action dispatcher', () async {
    final triggers = _RecordingTriggerController();
    final controller = ExternalActionController(triggers: triggers);

    await controller.dispatchProtocol(
      const ProtocolCommand(action: ProtocolAction.translateSelection),
    );
    await controller.dispatchProtocol(
      const ProtocolCommand(action: ProtocolAction.translateInput),
    );
    await controller.dispatchProtocol(
      const ProtocolCommand(action: ProtocolAction.translateClipboard),
    );
    await controller.dispatchProtocol(
      const ProtocolCommand(action: ProtocolAction.captureTranslate),
    );
    await controller.dispatchProtocol(
      const ProtocolCommand(action: ProtocolAction.captureOcr),
    );
    await controller.dispatchProtocol(
      const ProtocolCommand(action: ProtocolAction.clipboardOcr),
    );
    await controller.dispatchProtocol(
      const ProtocolCommand(action: ProtocolAction.showTranslationWindow),
    );
    await controller.dispatchProtocol(
      const ProtocolCommand(action: ProtocolAction.ignored),
    );

    expect(triggers.inputWindowCount, 1);
    expect(triggers.translationWindowCount, 1);
    expect(triggers.actions, [
      TriggerAction.translateSelection,
      TriggerAction.translateInput,
      TriggerAction.captureAndTranslate,
      TriggerAction.captureOcr,
      TriggerAction.clipboardOcr,
    ]);
  });
}

final class _RecordingTriggerController extends TriggerController {
  final List<TriggerAction> actions = [];
  int inputWindowCount = 0;
  int translationWindowCount = 0;

  @override
  Future<void> trigger(TriggerAction action) async {
    actions.add(action);
  }

  @override
  Future<void> openInputWindow({Rect? trayBounds}) async {
    inputWindowCount++;
  }

  @override
  Future<void> showTranslationWindow({Rect? trayBounds}) async {
    translationWindowCount++;
  }
}
