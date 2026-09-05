import 'dart:async';
import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/main.dart' as app;
import 'package:linguaray_desktop/src/app/dependencies.dart';
import 'package:linguaray_desktop/src/app/navigation/library_settings_screens.dart';
import 'package:linguaray_desktop/src/app/navigation/settings_screens.dart';
import 'package:linguaray_desktop/src/app/navigation/settings_shell_view.dart';
import 'package:linguaray_desktop/src/app/runtime.dart';
import 'package:linguaray_desktop/src/app/windows/app_windows.dart';
import 'package:linguaray_desktop/src/features/translation/data/llm_stream.dart';
import 'package:linguaray_desktop/src/features/translation/quick_translate/quick_translate_screen.dart';
import 'package:linguaray_desktop/src/platform/credentials/secret_store.dart';
import 'package:linguaray_desktop/src/platform/platform_types.dart';

const _testSystemServices = bool.fromEnvironment(
  'LINGUARAY_SYSTEM_SERVICES_SMOKE',
);

bool get _isHeadlessWindowsCi =>
    Platform.isWindows && Platform.environment['GITHUB_ACTIONS'] == 'true';

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
    final providerContainer = ProviderScope.containerOf(
      tester.element(find.byType(SettingsShellView)),
    );

    await tester.runAsync(() async {
      final storage = NativeSecretStore();
      final id = 'vault-smoke-${DateTime.now().microsecondsSinceEpoch}';
      try {
        await storage.write(
          providerId: id,
          field: 'apiKey',
          value: 'public-vault-fixture',
        );
        expect(
          await NativeSecretStore().read(providerId: id, field: 'apiKey'),
          'public-vault-fixture',
        );
      } finally {
        await storage.delete(providerId: id, field: 'apiKey');
      }
      expect(await storage.read(providerId: id, field: 'apiKey'), isNull);
    });
    debugPrint('[smoke] native credential storage roundtrip passed');

    await tester.runAsync(() async {
      final shortcuts = providerContainer.read(shortcutServiceProvider);
      final deadline = DateTime.now().add(const Duration(seconds: 10));
      while (shortcuts.bindings.length != TriggerAction.values.length &&
          DateTime.now().isBefore(deadline)) {
        await Future<void>.delayed(const Duration(milliseconds: 100));
      }
    });
    final shortcutBindings = providerContainer
        .read(shortcutServiceProvider)
        .bindings;
    expect(shortcutBindings, hasLength(TriggerAction.values.length));
    final configuredShortcuts = shortcutBindings
        .where((binding) => binding.accelerator.isNotEmpty)
        .toList(growable: false);
    expect(configuredShortcuts.length, greaterThanOrEqualTo(6));
    expect(
      configuredShortcuts.every(
        (binding) => binding.state == ShortcutRegistrationState.registered,
      ),
      isTrue,
    );
    expect(
      shortcutBindings
          .where((binding) => binding.accelerator.isEmpty)
          .every(
            (binding) =>
                binding.state == ShortcutRegistrationState.unregistered,
          ),
      isTrue,
    );

    await tester.runAsync(() async {
      final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
      server.listen((request) async {
        await request.drain<void>();
        request.response.headers.contentType = ContentType(
          'text',
          'event-stream',
          charset: 'utf-8',
        );
        request.response.write(
          'data: {"choices":[{"index":0,"delta":{"content":"本地测试译文"},"finish_reason":"stop"}]}\n\n',
        );
        await request.response.close();
      });
      try {
        await runtime.settings().updateProvider(
          providerId: 'resident-smoke',
          providerType: 'openai_compatible',
          presetId: null,
          fields: {
            'baseUrl': 'http://127.0.0.1:${server.port}/v1',
            'defaultModel': 'local-test',
          },
        );
        final chunks = await LlmStream.translate(
          providerId: 'resident-smoke',
          sourceLang: 'en',
          targetLang: 'zh-Hans',
          text: 'Public smoke test',
        ).toList().timeout(const Duration(seconds: 8));
        expect(chunks.map((chunk) => chunk.content).join(), '本地测试译文');
        expect(chunks.last.finishReason, 'stop');
        final pending = runtime
            .llm(providerId: 'resident-smoke')
            .startTranslation(
              sourceLang: 'en',
              targetLang: 'zh-Hans',
              text: 'Cancel smoke test',
            );
        final next = pending.next();
        pending.cancel();
        expect(await next.timeout(const Duration(seconds: 3)), isNull);
      } finally {
        await runtime.settings().deleteProvider(providerId: 'resident-smoke');
        await server.close(force: true);
      }
    });
    debugPrint(
      '[smoke] Dart to Rust cancellable stream delivered local response',
    );

    final permissions = await providerContainer
        .read(permissionControllerProvider)
        .refresh();
    debugPrint('[smoke] permissions refreshed');
    expect(permissions.accessibility, isNot(PermissionState.unknown));
    expect(permissions.screenRecording, isNot(PermissionState.unknown));

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
      if (_isHeadlessWindowsCi && started.status == SpeechStatus.failed) {
        expect(started.errorCode, AppErrorCode.speechFailed.wireName);
        debugPrint('[smoke] speech device unavailable on headless Windows CI');
      } else {
        expect(started.status, SpeechStatus.speaking);
        await speechIdle.future.timeout(const Duration(seconds: 15));
        expect(
          speechStates,
          containsAllInOrder([SpeechStatus.speaking, SpeechStatus.idle]),
        );
      }
    } finally {
      await speech.stop().timeout(
        const Duration(seconds: 3),
        onTimeout: SpeechState.idle,
      );
      await speechSubscription.cancel();
    }
    debugPrint('[smoke] speech completed');

    final builtInServices = await runtime.settings().listServices();
    expect(
      builtInServices.map((service) => service.id),
      contains('ecdict+dictionary'),
    );

    final dictionary = providerContainer.read(dictionaryRepositoryProvider);
    final services = await dictionary.listCompatibleServiceIds();
    debugPrint('[smoke] dictionary catalog loaded');
    expect(services, contains('ecdict+dictionary'));
    final entry = await dictionary.lookup(
      const DictionaryLookupQuery(
        word: 'apple',
        sourceLanguage: 'en',
        targetLanguage: 'zh-Hans',
        serviceId: 'ecdict+dictionary',
      ),
    );
    debugPrint('[smoke] built-in ECDICT lookup completed');
    expect(entry.word.toLowerCase(), 'apple');
    expect(entry.providerName, 'ECDICT');
    expect(entry.isEmpty, isFalse);
    expect(entry.translations.join('\n'), contains('苹果'));

    if (Platform.isMacOS) {
      expect(
        builtInServices.map((service) => service.id),
        containsAll(['system+translation', 'system+dictionary']),
      );

      if (_testSystemServices) {
        final response = await runtime
            .translation(providerId: 'system+translation')
            .translate(
              request: TranslateRequest(
                sourceLanguage: 'en',
                targetLanguage: 'zh-Hans',
                text: 'Hello, this is a LinguaRay system translation test.',
              ),
            )
            .timeout(const Duration(seconds: 125));
        expect(response.translations, isNotEmpty);
        expect(response.translations.first.text.trim(), isNotEmpty);
        debugPrint(
          '[smoke] system translation completed: '
          '${response.translations.first.text}',
        );
      }
    }

    final showQuickWindow = providerContainer
        .read(triggerControllerProvider)
        .trigger(TriggerAction.toggleQuickWindow);
    await tester.pump();
    await showQuickWindow;
    await tester.pumpAndSettle();
    debugPrint('[smoke] quick window shown');
    expect(appSurface.value, AppSurface.miniTranslator);
    expect(find.byType(QuickTranslateScreen), findsOneWidget);
    expect(find.byType(ErrorWidget), findsNothing);
    final sourcePane = tester.getRect(
      find.byKey(const ValueKey('quick-source-pane')),
    );
    final resultPane = tester.getRect(
      find.byKey(const ValueKey('quick-result-pane')),
    );
    expect(resultPane.top, greaterThanOrEqualTo(sourcePane.bottom));
    expect(miniTranslatorWindowController.window.contentSize.width, 460);
    expect(miniTranslatorWindowController.window.isResizable, isTrue);
    if (Platform.isMacOS) {
      expect(
        await const MethodChannel('linguaray/mac_app_presentation')
            .invokeMethod<bool>('isDockIconVisible'),
        isFalse,
      );
    }
    miniTranslatorWindowController.window.setContentSize(760, 420);
    await tester.pumpAndSettle();
    expect(
      tester.getRect(find.byKey(const ValueKey('quick-result-pane'))).left,
      greaterThanOrEqualTo(
        tester.getRect(find.byKey(const ValueKey('quick-source-pane'))).right,
      ),
    );
    debugPrint(
      '[smoke] native quick window uses the two-column reading layout',
    );

    showSettingsWindow();
    await tester.pumpAndSettle();
    debugPrint('[smoke] settings window shown');
    expect(appSurface.value, AppSurface.settings);
    expect(find.byType(ErrorWidget), findsNothing);
    expect(settingsWindowController.window.isVisible, isTrue);
    if (Platform.isMacOS) {
      expect(
        await const MethodChannel('linguaray/mac_app_presentation')
            .invokeMethod<bool>('isDockIconVisible'),
        isFalse,
      );
    }
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
