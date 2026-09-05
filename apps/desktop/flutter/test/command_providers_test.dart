import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/src/app/commands/trigger_controller.dart';
import 'package:linguaray_desktop/src/app/dependencies.dart';
import 'package:linguaray_desktop/src/platform/capture/capture_controller.dart';
import 'package:linguaray_desktop/src/platform/permissions/permission_controller.dart';
import 'package:linguaray_desktop/src/platform/protocol/protocol_controller.dart';
import 'package:linguaray_desktop/src/platform/selection/selection_controller.dart';
import 'package:linguaray_desktop/src/platform/selection/selection_replacement_controller.dart';
import 'package:linguaray_desktop/src/platform/shortcuts/shortcut_service.dart';
import 'package:linguaray_desktop/src/platform/windows/dock_icon_controller.dart';

void main() {
  test('platform action providers expose composition-root instances', () {
    final container = ProviderContainer();
    addTearDown(container.dispose);
    expect(container.read(captureControllerProvider), same(captureController));
    expect(container.read(triggerControllerProvider), same(triggerController));
    expect(container.read(shortcutServiceProvider), same(shortcutService));
    expect(
      container.read(permissionControllerProvider),
      same(permissionController),
    );
    expect(
      container.read(selectionControllerProvider),
      same(selectionController),
    );
    expect(
      container.read(protocolControllerProvider),
      same(protocolController),
    );
    expect(
      container.read(selectionReplacementControllerProvider),
      same(selectionReplacementController),
    );
    expect(
      container.read(dockIconControllerProvider),
      same(dockIconController),
    );
  });

  test('platform action providers can be overridden', () {
    final permissions = PermissionController();
    final capture = CaptureController(permissions: permissions);
    final replacement = SelectionReplacementController();
    final selection = SelectionController(
      permissions: permissions,
      replacement: replacement,
    );
    final triggers = TriggerController(
      capture: capture,
      selection: selection,
      permissions: permissions,
    );
    final shortcuts = ShortcutService();
    final protocol = ProtocolController();
    final dockIcons = DockIconController();
    final container = ProviderContainer(
      overrides: [
        captureControllerProvider.overrideWithValue(capture),
        triggerControllerProvider.overrideWithValue(triggers),
        shortcutServiceProvider.overrideWithValue(shortcuts),
        permissionControllerProvider.overrideWithValue(permissions),
        selectionControllerProvider.overrideWithValue(selection),
        protocolControllerProvider.overrideWithValue(protocol),
        selectionReplacementControllerProvider.overrideWithValue(replacement),
        dockIconControllerProvider.overrideWithValue(dockIcons),
      ],
    );
    addTearDown(container.dispose);
    expect(container.read(captureControllerProvider), same(capture));
    expect(container.read(triggerControllerProvider), same(triggers));
    expect(container.read(shortcutServiceProvider), same(shortcuts));
    expect(container.read(permissionControllerProvider), same(permissions));
    expect(container.read(selectionControllerProvider), same(selection));
    expect(container.read(protocolControllerProvider), same(protocol));
    expect(
      container.read(selectionReplacementControllerProvider),
      same(replacement),
    );
    expect(container.read(dockIconControllerProvider), same(dockIcons));
    expect(
      container.read(captureControllerProvider),
      isNot(same(captureController)),
    );
    expect(
      container.read(triggerControllerProvider),
      isNot(same(triggerController)),
    );
    expect(
      container.read(shortcutServiceProvider),
      isNot(same(shortcutService)),
    );
    expect(
      container.read(permissionControllerProvider),
      isNot(same(permissionController)),
    );
    expect(
      container.read(selectionControllerProvider),
      isNot(same(selectionController)),
    );
    expect(
      container.read(protocolControllerProvider),
      isNot(same(protocolController)),
    );
    expect(
      container.read(selectionReplacementControllerProvider),
      isNot(same(selectionReplacementController)),
    );
    expect(
      container.read(dockIconControllerProvider),
      isNot(same(dockIconController)),
    );
  });
}
