import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart' show ThemeMode;
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/src/app/settings/settings_effects.dart';
import 'package:linguaray_desktop/src/app/settings/settings_section.dart';
import 'package:linguaray_runtime/linguaray_runtime.dart';

void main() {
  test('section listenables isolate unrelated settings writes', () {
    final sections = SettingsSectionListenables();
    var general = 0;
    var appearance = 0;
    var merged = 0;
    sections.of(SettingsSection.general).addListener(() => general++);
    sections.of(SettingsSection.appearance).addListener(() => appearance++);
    sections
        .merge(const [SettingsSection.general, SettingsSection.shortcuts])
        .addListener(() => merged++);

    sections.notify(SettingsSection.general);
    expect(general, 1);
    expect(appearance, 0);
    expect(merged, 1);

    sections.notify(SettingsSection.appearance);
    expect(general, 1);
    expect(appearance, 1);
    expect(merged, 1);

    sections.notify(SettingsSection.shortcuts);
    expect(merged, 2);
    sections.dispose();
  });

  test('start applies login, appearance, API, and a missing target', () async {
    final store = _FakeSnapshot(
      general: _general(launchAtLogin: true),
      appearance: AppearanceSettings(
        language: 'en',
        themeMode: 'dark',
        theme: 'bright',
      ),
      advanced: _advanced(enabled: true, port: 4780),
    );
    final effects = _FakeEffects(enabled: false, supported: true);
    final coordinator = SettingsEffectsCoordinator(
      store: store,
      effects: effects,
      defaultTranslationTarget: _target,
    );

    await coordinator.start();

    expect(effects.loginApplies, [true]);
    expect(effects.appearanceApplies, [ThemeMode.dark]);
    expect(effects.apiApplies, 1);
    expect(store.general.translationTargets, hasLength(1));
    expect(store.general.translationTargets.single.target, 'en');
    coordinator.dispose();
  });

  test('rejected login item writes the OS value back', () async {
    final store = _FakeSnapshot(general: _general(launchAtLogin: true));
    final effects = _FakeEffects(enabled: false, supported: true, apply: false);
    final coordinator = SettingsEffectsCoordinator(
      store: store,
      effects: effects,
      defaultTranslationTarget: _target,
    );

    final result = await coordinator.syncGeneral();

    expect(result.rejected, isTrue);
    expect(store.general.launchAtLogin, isFalse);
    expect(store.patches, hasLength(1));
    expect(store.patches.single.launchAtLogin, isFalse);
    coordinator.dispose();
  });

  test('unsupported login items are left unchanged', () async {
    final store = _FakeSnapshot(general: _general(launchAtLogin: true));
    final effects = _FakeEffects(enabled: false, supported: false);
    final coordinator = SettingsEffectsCoordinator(
      store: store,
      effects: effects,
      defaultTranslationTarget: _target,
    );

    expect((await coordinator.syncGeneral()).rejected, isFalse);
    expect(effects.loginApplies, isEmpty);
    expect(store.patches, isEmpty);
    coordinator.dispose();
  });

  test('concurrent advanced syncs share one apply', () async {
    final started = Completer<void>();
    final finish = Completer<ApiServerInfo?>();
    final store = _FakeSnapshot(advanced: _advanced(enabled: true));
    final effects = _FakeEffects(
      applyApiServer: (settings) async {
        if (!started.isCompleted) started.complete();
        return finish.future;
      },
    );
    final coordinator = SettingsEffectsCoordinator(
      store: store,
      effects: effects,
      defaultTranslationTarget: _target,
    );

    final first = coordinator.syncAdvanced();
    await started.future;
    final second = coordinator.syncAdvanced();
    finish.complete(
      ApiServerInfo(host: '127.0.0.1', port: 9, baseUrl: 'http://127.0.0.1:9'),
    );

    expect((await first)?.port, 9);
    expect((await second)?.port, 9);
    expect(effects.apiApplies, 1);
    coordinator.dispose();
  });

  test('dispose stops later section notifications', () async {
    final store = _FakeSnapshot();
    final effects = _FakeEffects();
    final coordinator = SettingsEffectsCoordinator(
      store: store,
      effects: effects,
      defaultTranslationTarget: _target,
    );
    await coordinator.start();
    effects.appearanceApplies.clear();
    coordinator.dispose();
    store.appearance = AppearanceSettings(
      language: 'zh-Hans',
      themeMode: 'light',
      theme: 'bright',
    );
    store.sections.notify(SettingsSection.appearance);
    await Future<void>.delayed(Duration.zero);
    expect(effects.appearanceApplies, isEmpty);
  });
}

TranslationTarget _target(String language) => TranslationTarget(
  source: 'auto',
  target: language.startsWith('en') ? 'en' : 'zh-Hans',
  enabled: true,
);

GeneralSettings _general({bool launchAtLogin = false}) => GeneralSettings(
  launchAtLogin: launchAtLogin,
  showInMenuBar: true,
  defaultOcrService: '',
  autoCopyDetectedText: true,
  defaultDirectoryService: '',
  defaultTranslationService: '',
  translationTargets: const [],
  inputSubmitMode: InputSubmitMode.enter,
  doubleClickCopyResult: true,
  commonLanguages: const ['en', 'zh-Hans'],
  translationServiceOrder: const [],
);

AdvancedSettings _advanced({bool enabled = false, int port = 0}) =>
    AdvancedSettings(
      apiServerEnabled: enabled,
      apiServerHost: '127.0.0.1',
      apiServerPort: port,
      proxyMode: 'system',
      proxyUrl: '',
      proxyBypass: 'localhost,127.0.0.1',
      checkUpdatesOnLaunch: true,
    );

final class _FakeSnapshot implements SettingsSnapshotSource {
  _FakeSnapshot({
    GeneralSettings? general,
    AppearanceSettings? appearance,
    AdvancedSettings? advanced,
  }) : general = general ?? _general(),
       appearance =
           appearance ??
           AppearanceSettings(
             language: 'zh-Hans',
             themeMode: 'system',
             theme: 'bright',
           ),
       advanced = advanced ?? _advanced();

  @override
  GeneralSettings general;
  @override
  AppearanceSettings appearance;
  @override
  AdvancedSettings advanced;
  final sections = SettingsSectionListenables();
  final patches = <GeneralSettingsPatch>[];

  @override
  Listenable listenableFor(SettingsSection section) => sections.of(section);

  @override
  Future<void> updateGeneral(GeneralSettingsPatch patch) async {
    patches.add(patch);
    general = GeneralSettings(
      launchAtLogin: patch.launchAtLogin ?? general.launchAtLogin,
      showInMenuBar: patch.showInMenuBar ?? general.showInMenuBar,
      defaultOcrService: patch.defaultOcrService ?? general.defaultOcrService,
      autoCopyDetectedText:
          patch.autoCopyDetectedText ?? general.autoCopyDetectedText,
      defaultDirectoryService:
          patch.defaultDirectoryService ?? general.defaultDirectoryService,
      defaultTranslationService:
          patch.defaultTranslationService ?? general.defaultTranslationService,
      translationTargets:
          patch.translationTargets ?? general.translationTargets,
      inputSubmitMode: patch.inputSubmitMode ?? general.inputSubmitMode,
      doubleClickCopyResult:
          patch.doubleClickCopyResult ?? general.doubleClickCopyResult,
      commonLanguages: patch.commonLanguages ?? general.commonLanguages,
      translationServiceOrder:
          patch.translationServiceOrder ?? general.translationServiceOrder,
    );
    sections.notify(SettingsSection.general);
  }
}

final class _FakeEffects implements SettingsSystemEffects {
  _FakeEffects({
    this.enabled = false,
    this.supported = true,
    this.apply = true,
    this._applyApiServer,
  });

  bool enabled;
  final bool supported;
  final bool apply;
  final Future<ApiServerInfo?> Function(AdvancedSettings settings)?
  _applyApiServer;
  final loginApplies = <bool>[];
  final appearanceApplies = <ThemeMode>[];
  var apiApplies = 0;

  @override
  bool get launchAtLoginSupported => supported;

  @override
  bool get launchAtLoginEnabled => enabled;

  @override
  bool applyLaunchAtLogin(bool value) {
    loginApplies.add(value);
    if (!apply) return false;
    enabled = value;
    return true;
  }

  @override
  Future<void> applyAppearance(ThemeMode mode) async {
    appearanceApplies.add(mode);
  }

  @override
  Future<ApiServerInfo?> applyApiServer(AdvancedSettings settings) async {
    apiApplies++;
    if (_applyApiServer != null) return _applyApiServer(settings);
    if (!settings.apiServerEnabled) return null;
    return ApiServerInfo(
      host: settings.apiServerHost,
      port: settings.apiServerPort == 0 ? 1 : settings.apiServerPort,
      baseUrl: 'http://${settings.apiServerHost}:${settings.apiServerPort}',
    );
  }

  @override
  void dispose() {}
}
