import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/main.dart' as app;
import 'package:linguaray_desktop/src/app/commands/trigger_controller.dart';
import 'package:linguaray_desktop/src/app/dependencies.dart';
import 'package:linguaray_desktop/src/app/navigation/settings_shell_view.dart';
import 'package:linguaray_desktop/src/app/runtime.dart';
import 'package:linguaray_desktop/src/app/windows/app_windows.dart';
import 'package:linguaray_desktop/src/features/providers/provider_model_discovery_controller.dart';
import 'package:linguaray_desktop/src/features/providers/providers_settings_view.dart';
import 'package:linguaray_desktop/src/features/services/services_settings_view.dart';
import 'package:linguaray_desktop/src/features/translation/view_models/translation_view_model.dart';
import 'package:linguaray_desktop/src/features/updates/update_coordinator.dart';
import 'package:linguaray_desktop/src/platform/credentials/secret_store.dart';
import 'package:linguaray_desktop/src/platform/platform_types.dart';

// Explicitly opt in. Pass a path inside the macOS app container, never the key
// itself as a Dart define (defines are embedded in build products).
const _keyFile = String.fromEnvironment('LINGUARAY_DEEPSEEK_KEY_FILE');
const _isolatedDirectory = String.fromEnvironment('LINGUARAY_RUNTIME_DATA_DIR');

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  testWidgets(
    'DeepSeek live discovery, saved credentials, translation and refresh',
    (tester) async {
      expect(
        _isolatedDirectory.isNotEmpty,
        isTrue,
        reason: 'Live tests require an isolated runtime directory.',
      );
      final key = (await File(_keyFile).readAsString()).trim();
      expect(key.isNotEmpty, isTrue);
      final providerId =
          'deepseek-live-${DateTime.now().microsecondsSinceEpoch}';
      await app.main();
      await tester.pumpAndSettle();
      final container = ProviderScope.containerOf(
        tester.element(find.byType(SettingsShellView)),
      );
      final repository = container.read(providerSettingsRepositoryProvider);
      // Reclaim credentials from interrupted earlier runs in this isolated test
      // directory. IDs come only from this suite's public translation fixtures.
      final previousIds = <String>{};
      for (final file
          in runtimeDataDirectory.listSync(recursive: true).whereType<File>()) {
        if (file.path.endsWith('.json')) {
          previousIds.addAll(
            RegExp(r'deepseek-live-[0-9]+')
                .allMatches(await file.readAsString())
                .map((match) => match.group(0)!),
          );
        }
      }
      for (final id in previousIds) {
        await NativeSecretStore().delete(providerId: id, field: 'apiKey');
      }
      final settingsBefore = await runtime.settings().getJson();
      try {
        showSettingsWindow(
          destination: SettingsDestination.settingsTranslationServices,
        );
        await tester.pumpAndSettle();
        tester
            .widget<ServicesSettingsView>(find.byType(ServicesSettingsView))
            .onConfigureProviders();
        await tester.pumpAndSettle();
        tester
            .widget<ProvidersSettingsView>(find.byType(ProvidersSettingsView))
            .onAdd();
        await tester.pumpAndSettle();
        await _until(
          tester,
          () => tester
              .widget<ProviderEditorView>(find.byType(ProviderEditorView))
              .types
              .any((type) => type.id == 'deepseek'),
        );
        await tester.enterText(
          find.byKey(const ValueKey('provider-preset-search')),
          'DeepSeek',
        );
        await tester.pumpAndSettle();
        await tester.tap(
          find.byKey(const ValueKey('provider-preset-deepseek')),
        );
        await tester.pumpAndSettle();
        ProviderEditorView editor() =>
            tester.widget<ProviderEditorView>(find.byType(ProviderEditorView));
        editor().onIdChanged(providerId);
        await tester.pump();
        final keyInput = find.byKey(const ValueKey('provider-field-apiKey'));
        await tester.enterText(keyInput, key);
        await _until(tester, () => editor().discovery != null);
        final discovered = editor().discovery!;
        expect(
          discovered.succeeded,
          isTrue,
          reason: discovered.errorCode ?? 'Model discovery must succeed.',
        );
        expect(discovered.liveModels, contains('deepseek-v4-flash'));
        expect(discovered.liveModels, contains('deepseek-v4-pro'));
        debugPrint(
          '[deepseek-live] automatic discovery: ${discovered.liveModels.join(', ')}',
        );
        // Discovery of an unsaved draft must not write credentials or settings.
        expect(await runtime.settings().getJson() == settingsBefore, isTrue);
        expect(
          await runtime.settings().getProvider(providerId: providerId),
          isNull,
        );

        final firstQueryTime = discovered.queriedAt!;
        await tester.ensureVisible(find.byIcon(Icons.refresh_rounded));
        await tester.tap(find.byIcon(Icons.refresh_rounded));
        await _until(
          tester,
          () => !editor().loadingModels && editor().discovery != null,
        );
        expect(editor().discovery!.succeeded, isTrue);
        expect(editor().discovery!.queriedAt!.isAfter(firstQueryTime), isTrue);
        debugPrint(
          '[deepseek-live] manual refresh completed with a new query timestamp',
        );

        await tester.ensureVisible(keyInput);
        final refreshTime = editor().discovery!.queriedAt!;
        // The native refresh button moves focus while the test binding still
        // remembers this EditableText. Force a fresh input connection.
        tester.binding.focusedEditable = null;
        await tester.enterText(keyInput, 'invalid-live-test-key');
        await tester.pump();
        expect(
          editor().fields['apiKey'] == 'invalid-live-test-key',
          isTrue,
          reason: 'The edited credential must reach the provider draft.',
        );
        await _until(
          tester,
          () => editor().discovery?.queriedAt?.isAfter(refreshTime) == true,
        );
        expect(editor().discovery!.errorCode, 'auth_error');
        expect(editor().discovery!.liveModels, isEmpty);
        debugPrint(
          '[deepseek-live] invalid credentials clear the live list and show auth_error',
        );
        await tester.enterText(keyInput, key);
        await _until(tester, () => editor().discovery?.succeeded == true);
        final flash = find.byKey(
          const ValueKey('provider-model-deepseek-v4-flash'),
        );
        await tester.ensureVisible(flash);
        await tester.tap(flash);
        await tester.pump();
        expect(editor().fields['defaultModel'], 'deepseek-v4-flash');
        editor().onTest();
        await _until(
          tester,
          () => editor().testResult?.status == ProviderTestStatus.passed,
          timeout: const Duration(seconds: 100),
        );
        debugPrint('[deepseek-live] selected-model connection test passed');
        editor().onSave();
        await _until(
          tester,
          () =>
              find.byType(ProviderEditorView).evaluate().isEmpty ||
              editor().operationError != null,
        );
        expect(
          find.byType(ProviderEditorView),
          findsNothing,
          reason: 'Saving a provider through the OS vault must succeed.',
        );
        final saved = (await repository.listProviders()).singleWhere(
          (provider) => provider.id == providerId,
        );
        expect(saved.storedSecretKeys, contains('apiKey'));
        expect(saved.publicFields.containsKey('apiKey'), isFalse);
        expect((await runtime.settings().getJson()).contains(key), isFalse);
        for (final file
            in runtimeDataDirectory
                .listSync(recursive: true)
                .whereType<File>()) {
          if (file.path.endsWith('.json')) {
            expect(
              (await file.readAsString()).contains(key),
              isFalse,
              reason: 'Runtime JSON must not contain credentials.',
            );
          }
        }
        tester
            .widget<ProvidersSettingsView>(find.byType(ProvidersSettingsView))
            .onEdit(providerId);
        await _until(
          tester,
          () => find.byType(ProviderEditorView).evaluate().isNotEmpty,
        );
        await _until(tester, () => editor().discovery?.succeeded == true);
        expect(
          tester.widget<TextField>(keyInput).controller!.text.isEmpty,
          isTrue,
        );
        debugPrint(
          '[deepseek-live] reopened editor discovers models from the stored credential',
        );
        editor().onCancel();
        await tester.pumpAndSettle();
        Navigator.of(tester.element(find.byType(ProvidersSettingsView))).pop();
        await tester.pumpAndSettle();

        await container
            .read(serviceSettingsRepositoryProvider)
            .setDefaultTranslationService('$providerId+translation');
        final quick = triggerController.trigger(
          TriggerAction.toggleQuickWindow,
        );
        await tester.pump();
        await quick;
        await tester.pumpAndSettle();
        final translation = container.read(
          translationViewModelProvider.notifier,
        );
        await translation.initialize();
        expect(
          container.read(translationViewModelProvider).selectedServiceId,
          '$providerId+translation',
        );
        translation.setSourceLanguage('en');
        translation.setTargetLanguage('zh-Hans');
        translation.setSourceText(
          'The meeting starts at 9 a.m. Please bring the project report.',
        );
        await tester.runAsync(
          () => translation.submit().timeout(const Duration(seconds: 100)),
        );
        await tester.pumpAndSettle();
        final firstResult = container
            .read(translationViewModelProvider)
            .selectedResult!;
        expect(
          firstResult.status,
          TranslationResultStatus.completed,
          reason: firstResult.errorCode,
        );
        expect(firstResult.text.trimLeft().startsWith('{'), isFalse);
        expect(firstResult.text, contains('会议'));
        expect(firstResult.text, contains('报告'));
        debugPrint('[deepseek-live] flash en -> zh: ${firstResult.text}');

        // Update an already-saved provider without re-entering its secret or
        // restarting the app; the next translation must use the new model.
        await repository.saveProvider(
          ProviderDraft(
            id: providerId,
            typeId: 'deepseek',
            presetId: 'deepseek',
            fields: {...saved.publicFields, 'defaultModel': 'deepseek-v4-pro'},
          ),
        );
        final updated = await runtime.settings().getProvider(
          providerId: providerId,
        );
        expect(updated!.fields['defaultModel'], 'deepseek-v4-pro');
        translation.setSourceLanguage('zh-Hans');
        translation.setTargetLanguage('en');
        translation.setSourceText('请在周五前提交测试报告。');
        await tester.runAsync(
          () => translation.submit().timeout(const Duration(seconds: 100)),
        );
        await tester.pumpAndSettle();
        final secondResult = container
            .read(translationViewModelProvider)
            .selectedResult!;
        expect(
          secondResult.status,
          TranslationResultStatus.completed,
          reason: secondResult.errorCode,
        );
        expect(secondResult.text.trimLeft().startsWith('{'), isFalse);
        expect(secondResult.text.toLowerCase(), contains('friday'));
        expect(secondResult.text.toLowerCase(), contains('report'));
        debugPrint(
          '[deepseek-live] pro zh -> en after live model change: ${secondResult.text}',
        );

        await _verifyChangingModelList(tester, repository);
        showSettingsWindow(destination: SettingsDestination.settingsUpdates);
        await tester.pumpAndSettle();
        await tester.runAsync(
          () => container
              .read(updateCoordinatorProvider.notifier)
              .check()
              .timeout(const Duration(seconds: 40)),
        );
        await tester.pumpAndSettle();
        final update = container.read(updateCoordinatorProvider);
        expect(update.status, isNot(UpdateStatus.checking));
        debugPrint(
          '[deepseek-live] software update check: ${update.status.name}, '
          'current=${update.currentVersion}, error=${update.errorCode ?? 'none'}',
        );
        expect(find.byType(ErrorWidget), findsNothing);
      } finally {
        container.read(translationViewModelProvider.notifier).cancel();
        await container
            .read(serviceSettingsRepositoryProvider)
            .setDefaultTranslationService(null);
        await repository.deleteProvider(providerId);
      }
      expect(
        (await repository.listProviders()).any((p) => p.id == providerId),
        isFalse,
      );
      expect(
        await NativeSecretStore().read(providerId: providerId, field: 'apiKey'),
        isNull,
      );
      debugPrint(
        '[deepseek-live] temporary provider and secure credential removed',
      );
    },
    skip: _keyFile.isEmpty,
    timeout: const Timeout(Duration(minutes: 8)),
  );
}

Future<void> _until(
  WidgetTester tester,
  bool Function() ready, {
  Duration timeout = const Duration(seconds: 40),
}) async {
  // Publish any setState triggered by the preceding interaction before reading
  // the widget's immutable view data.
  await tester.pump();
  final deadline = DateTime.now().add(timeout);
  while (!ready()) {
    if (DateTime.now().isAfter(deadline)) {
      fail('Timed out waiting for the live application state.');
    }
    await tester.pump(const Duration(milliseconds: 100));
  }
  await tester.pump();
}

Future<void> _verifyChangingModelList(
  WidgetTester tester,
  ProviderSettingsRepository repository,
) async {
  // A controlled endpoint proves additions and removals independently of
  // whether DeepSeek happens to deploy a model during this test. No real key
  // is ever sent to this endpoint.
  var ids = ['fixture-old', 'fixture-kept'];
  var queries = 0;
  final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
  server.listen((request) async {
    queries++;
    await request.drain<void>();
    request.response.headers.contentType = ContentType.json;
    request.response.write(
      jsonEncode({
        'data': [
          for (final id in ids) {'id': id},
        ],
      }),
    );
    await request.response.close();
  });
  final controller = ProviderModelDiscoveryController(
    repository.discoverProviderModels,
  );
  final draft = ProviderDraft(
    id: 'controlled-discovery',
    typeId: 'deepseek',
    presetId: 'deepseek',
    fields: {
      'apiKey': 'local-fixture-only',
      'baseUrl': 'http://127.0.0.1:${server.port}/v1',
    },
  );
  try {
    controller.schedule(draft, immediately: true);
    await _until(tester, () => !controller.loading);
    expect(controller.result!.liveModels, ['fixture-kept', 'fixture-old']);
    ids = ['fixture-kept', 'fixture-new'];
    controller.schedule(draft, immediately: true);
    await _until(tester, () => !controller.loading);
    expect(controller.result!.liveModels, ['fixture-kept', 'fixture-new']);
    expect(queries, 2);
    debugPrint(
      '[deepseek-live] controlled refresh adds new IDs and removes stale IDs',
    );
  } finally {
    controller.dispose();
    await server.close(force: true);
  }
}
