import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart' show ThemeMode;
import 'package:linguaray_runtime/linguaray_runtime.dart';

import '../../shared/language_util.dart';
import '../runtime.dart' as runtime_service;
import 'settings_section.dart';

/// App-wide settings cache backed by the Rust runtime.
///
/// All settings persisted across app launches are owned by the Rust runtime
/// (`runtime.settings()`). This store provides a Flutter-friendly facade:
///
///   * loads the current snapshot at startup ([init])
///   * exposes synchronous getters for use in `build` methods
///   * notifies the changed [SettingsSection] (and [ChangeNotifier] listeners)
///   * keeps the last load [errorFor] a section without dropping a good cache
///
/// OS login items, native appearance, and the local API server are applied by
/// `SettingsEffectsCoordinator`. Window sizing and other UI-only state stay
/// outside this cache.
class SettingsStore extends ChangeNotifier implements SettingsSnapshotSource {
  SettingsStore._();

  static final SettingsStore instance = SettingsStore._();
  final SettingsSectionListenables _sections = SettingsSectionListenables();
  final Map<SettingsSection, Object?> _errors = {};

  /// Active subscription to runtime [SettingsChange] events. Started by
  /// [init] and stopped by [dispose]; while alive, every change made
  /// through any [Runtime] handle (Dart or Swift) triggers a reload of
  /// the affected section so this store always reflects the latest
  /// persisted state.
  SettingsSubscription? _subscription;
  bool _disposed = false;

  GeneralSettings _general = GeneralSettings(
    launchAtLogin: false,
    showInMenuBar: true,
    defaultOcrService: '',
    autoCopyDetectedText: true,
    defaultDirectoryService: '',
    defaultTranslationService: '',
    translationTargets: [],
    inputSubmitMode: InputSubmitMode.enter,
    doubleClickCopyResult: true,
    commonLanguages: defaultCommonLanguages(),
    translationServiceOrder: const [],
  );
  AppearanceSettings _appearance = AppearanceSettings(
    language: 'zh-Hans',
    themeMode: 'system',
    theme: 'bright',
  );
  ShortcutSettings _shortcuts = ShortcutSettings(
    toggleMiniTranslator: '',
    extractTextFromScreenSelection: '',
    extractTextFromScreenCapture: '',
    captureOcr: '',
    silentCaptureOcr: '',
    fileOcr: '',
    clipboardOcr: '',
    showOcrWindow: '',
    extractTextFromClipboard: '',
    translateInputContent: '',
  );
  AdvancedSettings _advanced = AdvancedSettings(
    apiServerEnabled: false,
    apiServerHost: '127.0.0.1',
    apiServerPort: 0,
    proxyMode: 'system',
    proxyUrl: '',
    proxyBypass: 'localhost,127.0.0.1',
    checkUpdatesOnLaunch: true,
  );
  List<ProviderConfigEntry> _providers = const [];
  List<ServiceConfigEntry> _services = const [];

  @override
  GeneralSettings get general => _general;
  @override
  AppearanceSettings get appearance => _appearance;
  ShortcutSettings get shortcuts => _shortcuts;
  @override
  AdvancedSettings get advanced => _advanced;
  List<ProviderConfigEntry> get providers => List.unmodifiable(_providers);
  List<ServiceConfigEntry> get services => List.unmodifiable(_services);

  String get appLanguage => _appearance.language;
  ThemeMode get themeMode {
    switch (_appearance.themeMode) {
      case 'light':
        return ThemeMode.light;
      case 'dark':
        return ThemeMode.dark;
      default:
        return ThemeMode.system;
    }
  }

  InputSubmitMode get inputSubmitMode => _general.inputSubmitMode;
  bool get autoCopyDetectedText => _general.autoCopyDetectedText;
  bool get doubleClickCopyResult => _general.doubleClickCopyResult;
  String get defaultOcrService => _general.defaultOcrService;
  String get defaultTranslationService => _general.defaultTranslationService;
  String get defaultDirectoryService => _general.defaultDirectoryService;

  @override
  Listenable listenableFor(SettingsSection section) => _sections.of(section);

  Listenable listenablesFor(Iterable<SettingsSection> sections) =>
      _sections.merge(sections);

  Object? errorFor(SettingsSection section) => _errors[section];

  Future<void> init() async {
    // Appearance owns the user's app language and must load first so the
    // lifecycle coordinator can seed a matching default translation target.
    await reloadAppearance();
    await Future.wait([
      reloadGeneral(),
      reloadShortcuts(),
      reloadAdvanced(),
      reloadProviders(),
      reloadServices(),
    ]);
    _startListeningForChanges();
  }

  @override
  void dispose() {
    _disposed = true;
    _subscription = null;
    _sections.dispose();
    super.dispose();
  }

  /// Subscribes to runtime change events. Idempotent.
  void _startListeningForChanges() {
    if (_subscription != null) return;
    final subscription = runtime_service.runtime.settings().subscribe();
    _subscription = subscription;
    unawaited(_consumeChanges(subscription));
  }

  Future<void> _consumeChanges(SettingsSubscription subscription) async {
    while (!_disposed && identical(_subscription, subscription)) {
      SettingsChange? change;
      try {
        change = await subscription.next();
      } catch (error, stackTrace) {
        debugPrint('SettingsStore subscription error: $error\n$stackTrace');
        break;
      }
      if (change == null) break;
      switch (change) {
        case SettingsChange.general:
          await reloadGeneral();
        case SettingsChange.appearance:
          await reloadAppearance();
        case SettingsChange.shortcuts:
          await reloadShortcuts();
        case SettingsChange.providers:
          await Future.wait([reloadProviders(), reloadServices()]);
        case SettingsChange.advanced:
          await reloadAdvanced();
        case SettingsChange.glossary:
          // Glossary views load through their repository when they become
          // active; glossary data does not belong in this settings cache.
          break;
        case SettingsChange.history:
          // History views load through their repository when they become
          // active; history data does not belong in this settings cache.
          break;
        case SettingsChange.vocabulary:
          break;
      }
    }
  }

  Future<void> reloadGeneral() async {
    final settings = runtime_service.runtime.settings();
    try {
      _general = await settings.getGeneral();
      _notify(SettingsSection.general);
    } catch (error, stackTrace) {
      debugPrint(
        'SettingsStore failed to reload general settings: $error\n$stackTrace',
      );
      _notify(SettingsSection.general, error: error);
    }
  }

  Future<void> reloadAppearance() async {
    final settings = runtime_service.runtime.settings();
    try {
      _appearance = await settings.getAppearance();
      _notify(SettingsSection.appearance);
    } catch (error, stackTrace) {
      debugPrint(
        'SettingsStore failed to reload appearance: $error\n$stackTrace',
      );
      _notify(SettingsSection.appearance, error: error);
    }
  }

  Future<void> reloadShortcuts() async {
    final settings = runtime_service.runtime.settings();
    try {
      _shortcuts = await settings.getShortcuts();
      _notify(SettingsSection.shortcuts);
    } catch (error, stackTrace) {
      debugPrint(
        'SettingsStore failed to reload shortcuts: $error\n$stackTrace',
      );
      _notify(SettingsSection.shortcuts, error: error);
    }
  }

  Future<void> reloadProviders() async {
    final settings = runtime_service.runtime.settings();
    try {
      _providers = await settings.listProviders();
      _notify(SettingsSection.providers);
    } catch (error, stackTrace) {
      debugPrint(
        'SettingsStore failed to reload providers: $error\n$stackTrace',
      );
      _notify(SettingsSection.providers, error: error);
    }
  }

  Future<void> reloadServices() async {
    final settings = runtime_service.runtime.settings();
    try {
      _services = await settings.listServices();
      _notify(SettingsSection.services);
    } catch (error, stackTrace) {
      debugPrint(
        'SettingsStore failed to reload services: $error\n$stackTrace',
      );
      _notify(SettingsSection.services, error: error);
    }
  }

  Future<void> reloadAdvanced() async {
    final settings = runtime_service.runtime.settings();
    try {
      _advanced = await settings.getAdvanced();
      _notify(SettingsSection.advanced);
    } catch (error, stackTrace) {
      debugPrint(
        'SettingsStore failed to reload advanced settings: $error\n$stackTrace',
      );
      _notify(SettingsSection.advanced, error: error);
    }
  }

  @override
  Future<void> updateGeneral(GeneralSettingsPatch patch) async {
    final settings = runtime_service.runtime.settings();
    _general = await settings.updateGeneral(patch: patch);
    _notify(SettingsSection.general);
  }

  Future<void> updateAppearance(AppearanceSettingsPatch patch) async {
    final settings = runtime_service.runtime.settings();
    _appearance = await settings.updateAppearance(patch: patch);
    _notify(SettingsSection.appearance);
  }

  Future<void> updateShortcuts(ShortcutSettingsPatch patch) async {
    final settings = runtime_service.runtime.settings();
    _shortcuts = await settings.updateShortcuts(patch: patch);
    _notify(SettingsSection.shortcuts);
  }

  Future<void> resetShortcuts() async {
    final settings = runtime_service.runtime.settings();
    _shortcuts = await settings.resetShortcuts();
    _notify(SettingsSection.shortcuts);
  }

  Future<void> updateAdvanced(AdvancedSettingsPatch patch) async {
    final settings = runtime_service.runtime.settings();
    _advanced = await settings.updateAdvanced(patch: patch);
    _notify(SettingsSection.advanced);
  }

  void _notify(SettingsSection section, {Object? error}) {
    if (_disposed) return;
    _errors[section] = error;
    _sections.notify(section);
    notifyListeners();
  }
}

/// Singleton accessor.
final settingsStore = SettingsStore.instance;
