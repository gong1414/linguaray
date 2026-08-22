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
import 'package:linguaray_desktop/src/ui/chrome/workbench_shell_view.dart';
import 'package:linguaray_desktop/src/ui/quick_translate/quick_translate_screen.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('launches a styled LinguaRay workbench surface', (tester) async {
    await app.main();
    await tester.pumpAndSettle();

    expect(find.byType(WorkbenchShellView), findsOneWidget);
    expect(find.byType(ErrorWidget), findsNothing);

    expect(ShortcutService.instance.bindings, hasLength(4));
    expect(
      ShortcutService.instance.bindings.every(
        (binding) => binding.state == ShortcutRegistrationState.registered,
      ),
      isTrue,
    );

    final permissions = await permissionController.refresh();
    expect(permissions.accessibility, isNot(PermissionState.unknown));
    expect(permissions.screenRecording, isNot(PermissionState.unknown));

    final providerContainer = ProviderScope.containerOf(
      tester.element(find.byType(WorkbenchShellView)),
    );
    final speech = providerContainer.read(speechServiceProvider);
    expect(await speech.isAvailable(), isTrue);
    final speechStates = <SpeechStatus>[];
    final speechSubscription = speech.states.listen(
      (state) => speechStates.add(state.status),
    );
    final started = await speech.speak(
      text: 'LinguaRay speech check',
      kind: SpeechUtteranceKind.source,
      language: 'en-US',
    );
    expect(started.status, SpeechStatus.speaking);
    for (
      var attempt = 0;
      attempt < 50 && !speechStates.contains(SpeechStatus.idle);
      attempt++
    ) {
      await tester.pump(const Duration(milliseconds: 200));
    }
    expect(
      speechStates,
      containsAllInOrder([SpeechStatus.speaking, SpeechStatus.idle]),
    );
    await speechSubscription.cancel();

    if (Platform.isMacOS) {
      final dictionary = providerContainer.read(dictionaryRepositoryProvider);
      final services = await dictionary.listCompatibleServiceIds();
      expect(services, contains('system+dictionary'));
      final entry = await dictionary.lookup(
        const DictionaryLookupQuery(
          word: 'apple',
          sourceLanguage: 'en',
          targetLanguage: 'zh-Hans',
          serviceId: 'system+dictionary',
        ),
      );
      expect(entry.word.toLowerCase(), 'apple');
      expect(entry.isEmpty, isFalse);
    }

    final showQuickWindow = triggerController.trigger(
      TriggerAction.toggleQuickWindow,
    );
    await tester.pump();
    await showQuickWindow;
    await tester.pumpAndSettle();
    expect(appSurface.value, AppSurface.miniTranslator);
    expect(find.byType(QuickTranslateScreen), findsOneWidget);
    expect(find.byType(ErrorWidget), findsNothing);

    showWorkbenchWindow();
    await tester.pumpAndSettle();
    expect(appSurface.value, AppSurface.workbench);
    expect(find.byType(ErrorWidget), findsNothing);
    expect(workbenchWindowController.window.isVisible, isTrue);
    expect(
      workbenchWindowController.window.size.width,
      greaterThanOrEqualTo(840),
    );
    final logicalViewWidth =
        tester.view.physicalSize.width / tester.view.devicePixelRatio;
    expect(logicalViewWidth, greaterThanOrEqualTo(840));
  });

  // The following workflows require a signed desktop session, accessibility /
  // screen-recording grants, and a real display. They are defined here for the
  // testing AI and must not block implementation:
  //
  // 1. Tray icon shows/hides the workbench and opens Settings.
  // 2. Global shortcuts: toggle quick window, selection, clipboard, capture.
  // 3. Selection clipboard snapshot is restored; restoration failure is shown.
  // 4. Capture/OCR: cancelled, failed, OCR not configured, OCR empty.
  // 5. Permission refresh on resume, focus, and immediately before capture.
  // 6. Multi-display placement of the quick translator inside the work area.
  // 7. Quick translator closes on blur unless pinned.
}
