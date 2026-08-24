import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/config/dependencies.dart';
import 'package:linguaray_desktop/src/ui/settings/settings_intent_controller.dart';
import 'package:linguaray_desktop/src/ui/settings/settings_labels.dart';
import 'package:linguaray_desktop/src/ui/settings/settings_screens.dart';
import 'package:linguaray_desktop/src/ui/settings/view_models/settings_view_model.dart';
import 'package:linguaray_desktop/src/ui/settings/view_models/shortcuts_view_model.dart';
import 'package:linguaray_desktop/src/ui/settings/views/shortcuts_settings_view.dart';

void main() {
  test(
    'provider validation prevents repository writes and reports the error',
    () async {
      final repository = _FakeWorkspaceSettingsRepository();
      final container = ProviderContainer(
        overrides: [
          workspaceSettingsRepositoryProvider.overrideWithValue(repository),
        ],
      );
      addTearDown(container.dispose);
      final subscription = container.listen(
        providersSettingsViewModelProvider,
        (_, _) {},
        fireImmediately: true,
      );
      addTearDown(subscription.close);
      await _waitFor(
        () => !container.read(providersSettingsViewModelProvider).loading,
      );

      const invalid = ProviderDraft(id: '', typeId: 'openai', fields: {});
      final notifier = container.read(
        providersSettingsViewModelProvider.notifier,
      );

      expect(await notifier.save(invalid), isFalse);
      expect(repository.saveCalls, 0);
      expect(
        container.read(providersSettingsViewModelProvider).operationErrorCode,
        'validation_missing',
      );

      await notifier.test(invalid);
      expect(repository.testCalls, 0);
      expect(
        container
            .read(providersSettingsViewModelProvider)
            .testResult
            ?.errorCode,
        'validation_missing',
      );
    },
  );

  test(
    'provider save failure is contained and always clears saving state',
    () async {
      final repository = _FakeWorkspaceSettingsRepository(throwOnSave: true);
      final container = ProviderContainer(
        overrides: [
          workspaceSettingsRepositoryProvider.overrideWithValue(repository),
        ],
      );
      addTearDown(container.dispose);
      final subscription = container.listen(
        providersSettingsViewModelProvider,
        (_, _) {},
        fireImmediately: true,
      );
      addTearDown(subscription.close);
      await _waitFor(
        () => !container.read(providersSettingsViewModelProvider).loading,
      );

      final saved = await container
          .read(providersSettingsViewModelProvider.notifier)
          .save(
            const ProviderDraft(
              id: 'openai',
              typeId: 'openai',
              fields: {'apiKey': 'secret', 'defaultModel': 'gpt-test'},
            ),
          );

      final state = container.read(providersSettingsViewModelProvider);
      expect(saved, isFalse);
      expect(repository.saveCalls, 1);
      expect(state.saving, isFalse);
      expect(state.operationErrorCode, 'save_failed');
    },
  );

  test(
    'shortcut recording can be cancelled and clears after submit failure',
    () async {
      final repository = _FakeShortcutRepository();
      final container = ProviderContainer(
        overrides: [shortcutRepositoryProvider.overrideWithValue(repository)],
      );
      addTearDown(container.dispose);
      final subscription = container.listen(
        shortcutsViewModelProvider,
        (_, _) {},
        fireImmediately: true,
      );
      addTearDown(subscription.close);
      await _waitFor(() => !container.read(shortcutsViewModelProvider).loading);
      final notifier = container.read(shortcutsViewModelProvider.notifier);

      await notifier.startRecording('toggleQuickWindow');
      expect(repository.beginRecordingCalls, 1);
      expect(
        container.read(shortcutsViewModelProvider).recordingActionId,
        'toggleQuickWindow',
      );
      await notifier.cancelRecording();
      expect(repository.endRecordingCalls, 1);
      expect(
        container.read(shortcutsViewModelProvider).recordingActionId,
        isNull,
      );

      repository.throwOnSet = true;
      await notifier.startRecording('toggleQuickWindow');
      await notifier.submitRecording('Control+X');
      expect(repository.endRecordingCalls, 2);
      expect(
        container.read(shortcutsViewModelProvider).recordingActionId,
        isNull,
      );
    },
  );

  testWidgets('clicking outside the active shortcut recorder cancels it', (
    tester,
  ) async {
    var cancelled = false;
    final labels = ShortcutsSettingsLabels(
      title: 'Shortcuts',
      record: 'Record',
      recording: 'Press keys',
      clear: 'Clear',
      reset: 'Reset',
      resetConfirmTitle: 'Reset?',
      resetConfirmBody: 'Reset shortcuts?',
      registered: 'Registered',
      unregistered: 'Unregistered',
      invalid: 'Invalid',
      conflict: (label) => 'Conflict: $label',
      cancel: 'Cancel',
      confirm: 'Confirm',
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ShortcutsSettingsView(
            labels: labels,
            shortcuts: const [
              ShortcutRecord(
                actionId: 'toggleQuickWindow',
                labelKey: 'Toggle quick window',
                accelerator: 'Option+1',
                status: ShortcutStatus.registered,
              ),
            ],
            recordingActionId: 'toggleQuickWindow',
            onStartRecording: (_) {},
            onCancelRecording: () => cancelled = true,
            onClear: (_) {},
            onReset: () {},
          ),
        ),
      ),
    );

    await tester.tap(find.text('Shortcuts'));
    await tester.pump();

    expect(cancelled, isTrue);
  });

  test('translation service reorder persists the visible order', () async {
    final repository = _FakeWorkspaceSettingsRepository(
      services: const [
        ServiceRecord(
          id: 'google-web+translation',
          name: 'Google Web',
          providerId: 'google-web',
          providerName: 'Google Web',
          kind: 'translation',
          enabled: true,
          isDefault: true,
        ),
        ServiceRecord(
          id: 'system+translation',
          name: 'System',
          providerId: 'system',
          providerName: 'System',
          kind: 'translation',
          enabled: false,
          isDefault: false,
        ),
      ],
    );
    final container = ProviderContainer(
      overrides: [
        workspaceSettingsRepositoryProvider.overrideWithValue(repository),
      ],
    );
    addTearDown(container.dispose);
    final subscription = container.listen(
      servicesSettingsViewModelProvider,
      (_, _) {},
      fireImmediately: true,
    );
    addTearDown(subscription.close);
    await _waitFor(
      () => !container.read(servicesSettingsViewModelProvider).loading,
    );

    await container
        .read(servicesSettingsViewModelProvider.notifier)
        .reorderTranslation(0, 1);

    expect(repository.lastTranslationOrder, [
      'system+translation',
      'google-web+translation',
    ]);
  });

  test('general settings persists translation target changes', () async {
    final repository = _FakeWorkspaceSettingsRepository();
    final container = ProviderContainer(
      overrides: [
        workspaceSettingsRepositoryProvider.overrideWithValue(repository),
      ],
    );
    addTearDown(container.dispose);
    final subscription = container.listen(
      generalSettingsViewModelProvider,
      (_, _) {},
      fireImmediately: true,
    );
    addTearDown(subscription.close);
    await _waitFor(
      () => !container.read(generalSettingsViewModelProvider).loading,
    );

    const targets = [
      TranslationTargetRule(source: 'auto', target: 'zh-Hans'),
      TranslationTargetRule(source: 'zh-Hans', target: 'en', enabled: false),
    ];
    await container
        .read(generalSettingsViewModelProvider.notifier)
        .setTranslationTargets(targets);

    expect(repository.lastTranslationTargets, targets);
  });

  testWidgets(
    'quick-window add-target intent reaches the current settings UI',
    (tester) async {
      final repository = _FakeWorkspaceSettingsRepository(
        translationLanguages: const [
          LanguageChoice(code: 'en', name: 'English'),
          LanguageChoice(code: 'zh-Hans', name: '简体中文'),
        ],
      );
      generalSettingsIntentController.request(
        GeneralSettingsIntent.addTranslationTarget,
      );

      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            workspaceSettingsRepositoryProvider.overrideWithValue(repository),
          ],
          child: const MaterialApp(
            home: Scaffold(body: GeneralSettingsScreen()),
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byKey(const ValueKey('source-auto')), findsOneWidget);
      expect(find.byKey(const ValueKey('target-zh-Hans')), findsOneWidget);
      await tester.tap(find.byType(FilledButton));
      await tester.pumpAndSettle();
      expect(repository.lastTranslationTargets, const [
        TranslationTargetRule(source: 'auto', target: 'zh-Hans'),
      ]);
      expect(generalSettingsIntentController.hasPending, isFalse);
    },
  );
}

Future<void> _waitFor(bool Function() condition) async {
  for (var attempt = 0; attempt < 50; attempt++) {
    if (condition()) return;
    await Future<void>.delayed(const Duration(milliseconds: 1));
  }
  fail('Timed out waiting for view-model state.');
}

final class _FakeWorkspaceSettingsRepository
    implements WorkspaceSettingsRepository {
  _FakeWorkspaceSettingsRepository({
    this.throwOnSave = false,
    this.services = const [],
    this.translationLanguages = const [],
  });

  final bool throwOnSave;
  final List<ServiceRecord> services;
  final List<LanguageChoice> translationLanguages;
  final GeneralPreferences generalPreferences = const GeneralPreferences(
    launchAtLogin: false,
    showInMenuBar: true,
    language: 'en',
    themeMode: ThemePreference.system,
  );
  int saveCalls = 0;
  int testCalls = 0;
  List<String>? lastTranslationOrder;
  List<TranslationTargetRule>? lastTranslationTargets;

  @override
  Future<List<ProviderTypeOption>> listProviderTypes() async => const [
    ProviderTypeOption(
      id: 'openai',
      label: 'OpenAI',
      isLlm: true,
      fields: [
        ProviderFieldSpec(
          key: 'apiKey',
          label: 'API key',
          secret: true,
          requiredField: true,
        ),
        ProviderFieldSpec(
          key: 'defaultModel',
          label: 'Default model',
          secret: false,
          requiredField: true,
        ),
      ],
    ),
  ];

  @override
  Future<List<ProviderRecord>> listProviders() async => const [];

  @override
  Future<void> saveProvider(ProviderDraft draft) async {
    saveCalls++;
    if (throwOnSave) throw StateError('save failed');
  }

  @override
  Future<ProviderTestResult> testProvider(ProviderDraft draft) async {
    testCalls++;
    return const ProviderTestResult(status: ProviderTestStatus.passed);
  }

  @override
  Future<AboutInfo> loadAbout() async => const AboutInfo(
    appName: 'LinguaRay',
    version: '0',
    buildNumber: '0',
    platformLabel: 'test',
    license: 'MIT',
  );

  @override
  Future<void> deleteProvider(String providerId) async {}

  @override
  Future<List<String>> listProviderModels(String providerId) async => const [];

  @override
  Future<List<String>> discoverProviderModels(ProviderDraft draft) async =>
      const [];

  @override
  Future<void> reorderTranslationServices(List<String> serviceIds) async {
    lastTranslationOrder = List.of(serviceIds);
  }

  @override
  Future<List<LanguageChoice>> listAppLanguages() async => const [];
  @override
  Future<List<ServiceRecord>> listServices() async => services;
  @override
  Future<List<LanguageChoice>> listTranslationLanguages() async =>
      translationLanguages;
  @override
  Future<GeneralPreferences> loadGeneral() async => generalPreferences;
  @override
  Future<List<String>> loadCommonLanguages() async => const [];
  @override
  Future<String?> loadDefaultOcrService() async => null;
  @override
  Future<String?> loadDefaultTranslationService() async => null;
  @override
  Future<void> setCommonLanguages(List<String> codes) async {}
  @override
  Future<void> setDefaultOcrService(String? serviceId) async {}
  @override
  Future<void> setDefaultTranslationService(String? serviceId) async {}
  @override
  Future<void> setLanguage(String language) async {}
  @override
  Future<void> setLaunchAtLogin(bool value) async {}
  @override
  Future<void> setServiceEnabled({
    required String serviceId,
    required bool enabled,
  }) async {}
  @override
  Future<void> setShowInMenuBar(bool value) async {}
  @override
  Future<void> setThemeMode(ThemePreference mode) async {}

  @override
  Future<List<TranslationTargetRule>> loadTranslationTargets() async =>
      const [];

  @override
  Future<void> setTranslationTargets(
    List<TranslationTargetRule> targets,
  ) async {
    lastTranslationTargets = List.of(targets);
  }

  @override
  Future<InputSubmitMode> loadInputSubmitMode() async => InputSubmitMode.enter;

  @override
  Future<void> setInputSubmitMode(InputSubmitMode mode) async {}

  @override
  Future<bool> loadAutoCopyDetectedText() async => true;

  @override
  Future<void> setAutoCopyDetectedText(bool value) async {}

  @override
  Future<bool> loadDoubleClickCopyResult() async => true;

  @override
  Future<void> setDoubleClickCopyResult(bool value) async {}

  @override
  Future<String?> loadDefaultDictionaryService() async => null;

  @override
  Future<void> setDefaultDictionaryService(String? serviceId) async {}

  @override
  Future<void> saveService(ServiceDraft draft) async {}

  @override
  Future<void> deleteService(String serviceId) async {}

  @override
  Future<ApiServerStatus> loadApiServer() async =>
      const ApiServerStatus(enabled: false, host: '127.0.0.1', port: 0);

  @override
  Future<ApiServerStatus> setApiServerEnabled(bool enabled) async =>
      loadApiServer();

  @override
  Future<ApiServerStatus> setApiServerPort(int port) async => loadApiServer();

  @override
  Future<NetworkSettings> loadNetworkSettings() async => const NetworkSettings(
    proxyMode: NetworkProxyMode.system,
    proxyUrl: '',
    proxyBypass: 'localhost,127.0.0.1',
    checkUpdatesOnLaunch: true,
  );

  @override
  Future<NetworkSettings> saveNetworkSettings(NetworkSettings settings) async =>
      settings;

  @override
  Future<PlatformCapabilities> loadCapabilities() async =>
      const PlatformCapabilities.macos();
}

final class _FakeShortcutRepository implements ShortcutRepository {
  bool throwOnSet = false;
  int beginRecordingCalls = 0;
  int endRecordingCalls = 0;

  @override
  Future<void> beginRecording() async {
    beginRecordingCalls++;
  }

  @override
  Future<void> endRecording() async {
    endRecordingCalls++;
  }

  @override
  Future<List<ShortcutRecord>> load() async => const [
    ShortcutRecord(
      actionId: 'toggleQuickWindow',
      labelKey: 'toggleQuickWindow',
      accelerator: 'Option+1',
      status: ShortcutStatus.registered,
    ),
  ];

  @override
  Future<void> setAccelerator({
    required String actionId,
    required String accelerator,
  }) async {
    if (throwOnSet) throw StateError('shortcut failed');
  }

  @override
  Future<void> clear(String actionId) async {}
  @override
  Future<void> resetDefaults() async {}
}
