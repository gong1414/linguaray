// ignore_for_file: prefer_initializing_formals

import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart' show ThemeMode;
import 'package:nativeapi/nativeapi.dart';

import '../../platform/platform_util.dart';
import '../../platform/windows/mac_app_presentation.dart';
import '../../shared/language_util.dart';
import '../runtime.dart';
import 'settings_section.dart';
import 'settings_store.dart';

final class LoginItemSync {
  const LoginItemSync._({required this.rejected});

  const LoginItemSync.applied() : this._(rejected: false);
  const LoginItemSync.rejected() : this._(rejected: true);

  final bool rejected;
}

abstract interface class SettingsSystemEffects {
  bool get launchAtLoginSupported;
  bool get launchAtLoginEnabled;
  bool applyLaunchAtLogin(bool enabled);
  Future<void> applyAppearance(ThemeMode mode);
  Future<ApiServerInfo?> applyApiServer(AdvancedSettings settings);
  void dispose();
}

final class DesktopSettingsSystemEffects implements SettingsSystemEffects {
  DesktopSettingsSystemEffects()
    : _launchAtLogin = LaunchAtLogin(
        id: 'io.github.gong1414.linguaray',
        displayName: 'LinguaRay',
      );

  final LaunchAtLogin _launchAtLogin;

  @override
  bool get launchAtLoginSupported =>
      (kIsMacOS || kIsWindows) && LaunchAtLogin.isSupported;

  @override
  bool get launchAtLoginEnabled => _launchAtLogin.isEnabled;

  @override
  bool applyLaunchAtLogin(bool enabled) {
    if (!launchAtLoginSupported) return false;
    if (_launchAtLogin.isEnabled == enabled) return true;
    return enabled ? _launchAtLogin.enable() : _launchAtLogin.disable();
  }

  @override
  Future<void> applyAppearance(ThemeMode mode) {
    return MacAppPresentation.setThemeMode(mode);
  }

  @override
  Future<ApiServerInfo?> applyApiServer(AdvancedSettings settings) {
    return applyApiServerSettings(settings);
  }

  @override
  void dispose() => _launchAtLogin.dispose();
}

/// Applies OS login items, native appearance, and the local API from the
/// settings cache. The cache itself only stores snapshots and errors.
final class SettingsEffectsCoordinator {
  SettingsEffectsCoordinator({
    required SettingsSnapshotSource store,
    SettingsSystemEffects? effects,
    TranslationTarget Function(String appLanguage)? defaultTranslationTarget,
  }) : _store = store,
       _effects = effects ?? DesktopSettingsSystemEffects(),
       _defaultTranslationTarget =
           defaultTranslationTarget ?? _defaultTranslationTargetForLanguage;

  final SettingsSnapshotSource _store;
  final SettingsSystemEffects _effects;
  final TranslationTarget Function(String appLanguage)
  _defaultTranslationTarget;

  bool _started = false;
  bool _disposed = false;
  Future<LoginItemSync>? _generalSync;
  Future<void>? _appearanceSync;
  Future<ApiServerInfo?>? _advancedSync;

  Future<void> start() async {
    if (_started || _disposed) return;
    _started = true;
    _store.listenableFor(SettingsSection.general).addListener(_onGeneral);
    _store.listenableFor(SettingsSection.appearance).addListener(_onAppearance);
    _store.listenableFor(SettingsSection.advanced).addListener(_onAdvanced);
    await _ensureDefaultTranslationTarget();
    await Future.wait<void>([syncGeneral(), syncAppearance(), syncAdvanced()]);
  }

  void dispose() {
    if (_disposed) return;
    _disposed = true;
    if (_started) {
      _store.listenableFor(SettingsSection.general).removeListener(_onGeneral);
      _store
          .listenableFor(SettingsSection.appearance)
          .removeListener(_onAppearance);
      _store
          .listenableFor(SettingsSection.advanced)
          .removeListener(_onAdvanced);
    }
    _effects.dispose();
  }

  void _onGeneral() => unawaited(syncGeneral());
  void _onAppearance() => unawaited(syncAppearance());
  void _onAdvanced() => unawaited(syncAdvanced());

  Future<LoginItemSync> syncGeneral() {
    final inFlight = _generalSync;
    if (inFlight != null) return inFlight;
    final future = _syncGeneral();
    _generalSync = future;
    return future.whenComplete(() {
      if (identical(_generalSync, future)) _generalSync = null;
    });
  }

  Future<void> syncAppearance() {
    final inFlight = _appearanceSync;
    if (inFlight != null) return inFlight;
    final future = _syncAppearance();
    _appearanceSync = future;
    return future.whenComplete(() {
      if (identical(_appearanceSync, future)) _appearanceSync = null;
    });
  }

  Future<ApiServerInfo?> syncAdvanced() {
    final inFlight = _advancedSync;
    if (inFlight != null) return inFlight;
    final future = _syncAdvanced();
    _advancedSync = future;
    return future.whenComplete(() {
      if (identical(_advancedSync, future)) _advancedSync = null;
    });
  }

  Future<void> _ensureDefaultTranslationTarget() async {
    if (_store.general.translationTargets.isNotEmpty) return;
    await _store.updateGeneral(
      GeneralSettingsPatch(
        translationTargets: [
          _defaultTranslationTarget(_store.appearance.language),
        ],
      ),
    );
  }

  Future<LoginItemSync> _syncGeneral() async {
    if (_disposed) return const LoginItemSync.applied();
    if (!_effects.launchAtLoginSupported) return const LoginItemSync.applied();
    final wanted = _store.general.launchAtLogin;
    if (_effects.launchAtLoginEnabled == wanted) {
      return const LoginItemSync.applied();
    }
    if (_effects.applyLaunchAtLogin(wanted)) {
      return const LoginItemSync.applied();
    }
    final actual = _effects.launchAtLoginEnabled;
    if (actual != wanted) {
      await _store.updateGeneral(GeneralSettingsPatch(launchAtLogin: actual));
    }
    return const LoginItemSync.rejected();
  }

  Future<void> _syncAppearance() async {
    if (_disposed) return;
    try {
      await _effects.applyAppearance(
        themeModeFromAppearance(_store.appearance.themeMode),
      );
    } catch (error, stackTrace) {
      debugPrint('Failed to apply native appearance: $error\n$stackTrace');
    }
  }

  Future<ApiServerInfo?> _syncAdvanced() async {
    if (_disposed) return null;
    return _effects.applyApiServer(_store.advanced);
  }
}

TranslationTarget _defaultTranslationTargetForLanguage(String language) {
  return TranslationTarget(
    source: kAutoSource,
    target: defaultTargetLanguageForAppLanguage(language),
    enabled: true,
  );
}

final settingsEffects = SettingsEffectsCoordinator(store: settingsStore);
