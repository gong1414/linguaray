import 'dart:async';
import 'dart:io';

import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/main.dart' as app;
import 'package:linguaray_desktop/src/config/dependencies.dart';
import 'package:linguaray_desktop/src/platform/permission_controller.dart';
import 'package:linguaray_desktop/src/platform/platform_types.dart';
import 'package:linguaray_desktop/src/platform/trigger_controller.dart';
import 'package:linguaray_desktop/src/services/app_windows.dart';
import 'package:linguaray_desktop/src/services/shortcut_service/shortcut_service.dart';
import 'package:linguaray_desktop/src/ui/quick_translate/quick_translate_screen.dart';
import 'package:linguaray_desktop/src/ui/settings/library_settings_screens.dart';
import 'package:linguaray_desktop/src/ui/settings/settings_screens.dart';
import 'package:linguaray_desktop/src/ui/settings/settings_shell_view.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('launches resident and exposes settings plus quick translation', (
    tester,
  ) async {
    debugPrint('[smoke] starting app');
    await app.main();
    await tester.pumpAndSettle();
    debugPrint('[smoke] app mounted');

    expect(find.byType(SettingsShellView), findsOneWidget);
    expect(find.byType(ErrorWidget), findsNothing);
    expect(settingsWindowController.window.isVisible, isFalse);

    expect(ShortcutService.instance.bindings, hasLength(6));
    expect(
      ShortcutService.instance.bindings.every(
        (binding) => binding.state == ShortcutRegistrationState.registered,
      ),
      isTrue,
    );

    final permissions = await permissionController.refresh();
    debugPrint('[smoke] permissions refreshed');
    expect(permissions.accessibility, isNot(PermissionState.unknown));
    expect(permissions.screenRecording, isNot(PermissionState.unknown));

    final providerContainer = ProviderScope.containerOf(
      tester.element(find.byType(SettingsShellView)),
    );
    final speech = providerContainer.read(speechServiceProvider);
    expect(await speech.isAvailable(), isTrue);
    debugPrint('[smoke] speech available');
    final speechStates = <SpeechStatus>[];
    final speechIdle = Completer<void>();
    final speechSubscription = speech.states.listen((state) {
      speechStates.add(state.status);
      if (state.status == SpeechStatus.idle && !speechIdle.isCompleted) {
        speechIdle.complete();
      }
    });
    try {
      final started = await speech.speak(
        text: 'LinguaRay speech check',
        kind: SpeechUtteranceKind.source,
        language: 'en-US',
      );
      debugPrint('[smoke] speech started');
      expect(started.status, SpeechStatus.speaking);
      await speechIdle.future.timeout(const Duration(seconds: 15));
      expect(
        speechStates,
        containsAllInOrder([SpeechStatus.speaking, SpeechStatus.idle]),
      );
    } finally {
      await speech.stop().timeout(
        const Duration(seconds: 3),
        onTimeout: SpeechState.idle,
      );
      await speechSubscription.cancel();
    }
    debugPrint('[smoke] speech completed');

    if (Platform.isMacOS) {
      final dictionary = providerContainer.read(dictionaryRepositoryProvider);
      final services = await dictionary.listCompatibleServiceIds();
      debugPrint('[smoke] dictionary catalog loaded');
      expect(services, contains('system+dictionary'));
      final entry = await dictionary.lookup(
        const DictionaryLookupQuery(
          word: 'apple',
          sourceLanguage: 'en',
          targetLanguage: 'zh-Hans',
          serviceId: 'system+dictionary',
        ),
      );
      debugPrint('[smoke] dictionary lookup completed');
      expect(entry.word.toLowerCase(), 'apple');
      expect(entry.isEmpty, isFalse);
    }

    final showQuickWindow = triggerController.trigger(
      TriggerAction.toggleQuickWindow,
    );
    await tester.pump();
    await showQuickWindow;
    await tester.pumpAndSettle();
    debugPrint('[smoke] quick window shown');
    expect(appSurface.value, AppSurface.miniTranslator);
    expect(find.byType(QuickTranslateScreen), findsOneWidget);
    expect(find.byType(ErrorWidget), findsNothing);

    showSettingsWindow();
    await tester.pumpAndSettle();
    debugPrint('[smoke] settings window shown');
    expect(appSurface.value, AppSurface.settings);
    expect(find.byType(ErrorWidget), findsNothing);
    expect(settingsWindowController.window.isVisible, isTrue);
    expect(
      settingsWindowController.window.size.width,
      greaterThanOrEqualTo(840),
    );
    final logicalViewWidth =
        tester.view.physicalSize.width / tester.view.devicePixelRatio;
    expect(logicalViewWidth, greaterThanOrEqualTo(840));

    for (final destination in SettingsDestination.values) {
      showSettingsWindow(destination: destination);
      await tester.pumpAndSettle();
      expect(
        find.byType(SettingsShellView),
        findsOneWidget,
        reason: '${destination.name} must stay inside the settings window',
      );
      expect(
        find.byType(ErrorWidget),
        findsNothing,
        reason: '${destination.name} must resolve to a working settings page',
      );
    }
    debugPrint('[smoke] all settings destinations resolved');

    for (final (destination, screenType) in [
      (SettingsDestination.settingsGlossary, GlossarySettingsScreen),
      (SettingsDestination.settingsVocabulary, VocabularySettingsScreen),
      (SettingsDestination.settingsIntegration, AdvancedSettingsScreen),
    ]) {
      showSettingsWindow(destination: destination);
      await tester.pumpAndSettle();
      expect(find.byType(screenType), findsOneWidget);
      expect(find.byType(ErrorWidget), findsNothing);
    }
  });

  // The following workflows require a signed desktop session, accessibility /
  // screen-recording grants, and a real display. They are defined here for the
  // testing AI and must not block implementation:
  //
  // 1. Tray icon opens the native action menu; Settings is a menu item.
  // 2. Global shortcuts: toggle quick window, selection, clipboard, capture.
  // 3. Selection clipboard snapshot is restored; restoration failure is shown.
  // 4. Capture/OCR: cancelled, failed, OCR not configured, OCR empty.
  // 5. Permission refresh on resume, focus, and immediately before capture.
  // 6. Multi-display placement of the quick translator inside the work area.
  // 7. Quick translator closes on blur unless pinned.
}
