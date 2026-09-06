import 'package:flutter/widgets.dart' show Rect;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/app/commands/external_action_controller.dart';
import 'package:linguaray_desktop/src/app/commands/trigger_controller.dart';
import 'package:linguaray_desktop/src/app/dependencies.dart';
import 'package:linguaray_desktop/src/app/settings/settings_store.dart';
import 'package:linguaray_desktop/src/features/ocr/ocr_controller.dart';
import 'package:linguaray_desktop/src/platform/capture/capture_controller.dart';
import 'package:linguaray_desktop/src/platform/permissions/permission_controller.dart';
import 'package:linguaray_desktop/src/platform/platform_types.dart';
import 'package:linguaray_desktop/src/platform/selection/selection_controller.dart';
import 'package:linguaray_desktop/src/platform/selection/selection_replacement_controller.dart';

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

  test(
    'external action provider dispatches through triggerControllerProvider',
    () async {
      final triggers = _RecordingTriggerController();
      final container = ProviderContainer(
        overrides: [triggerControllerProvider.overrideWithValue(triggers)],
      );
      addTearDown(container.dispose);

      await container
          .read(externalActionControllerProvider)
          .dispatchProtocol(
            const ProtocolCommand(action: ProtocolAction.translateSelection),
          );

      expect(triggers.actions, [TriggerAction.translateSelection]);
    },
  );
}

final class _RecordingTriggerController extends TriggerController {
  _RecordingTriggerController()
    : super(
        selection: SelectionController(
          permissions: PermissionController(),
          replacement: SelectionReplacementController(),
        ),
        capture: CaptureController(
          permissions: PermissionController(),
          store: settingsStore,
        ),
        ocr: OcrController(
          capture: CaptureController(
            permissions: PermissionController(),
            store: settingsStore,
          ),
          autoCopy: () => false,
          writeClipboard: (_) async {},
        ),
        permissions: PermissionController(),
      );

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
