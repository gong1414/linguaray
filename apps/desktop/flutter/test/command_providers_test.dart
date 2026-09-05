import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/src/app/commands/trigger_controller.dart';
import 'package:linguaray_desktop/src/app/dependencies.dart';
import 'package:linguaray_desktop/src/platform/capture/capture_controller.dart';
import 'package:linguaray_desktop/src/platform/shortcuts/shortcut_service.dart';

void main() {
  test(
    'capture, trigger and shortcut providers expose composition-root instances',
    () {
      final container = ProviderContainer();
      addTearDown(container.dispose);
      expect(
        container.read(captureControllerProvider),
        same(captureController),
      );
      expect(
        container.read(triggerControllerProvider),
        same(triggerController),
      );
      expect(container.read(shortcutServiceProvider), same(shortcutService));
    },
  );

  test('capture, trigger and shortcut providers can be overridden', () {
    final capture = CaptureController();
    final triggers = TriggerController(capture: capture);
    final shortcuts = ShortcutService();
    final container = ProviderContainer(
      overrides: [
        captureControllerProvider.overrideWithValue(capture),
        triggerControllerProvider.overrideWithValue(triggers),
        shortcutServiceProvider.overrideWithValue(shortcuts),
      ],
    );
    addTearDown(container.dispose);
    expect(container.read(captureControllerProvider), same(capture));
    expect(container.read(triggerControllerProvider), same(triggers));
    expect(container.read(shortcutServiceProvider), same(shortcuts));
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
  });
}
