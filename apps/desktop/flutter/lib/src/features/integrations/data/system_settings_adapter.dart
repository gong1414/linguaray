import 'dart:io';

import 'package:linguaray_application/linguaray_application.dart';

import '../../../app/env.dart';
import '../../../app/runtime.dart';
import '../../../app/settings/settings_store.dart';

final class RuntimeSystemSettingsAdapter {
  const RuntimeSystemSettingsAdapter(this._store);

  final SettingsStore _store;

  Future<ApiServerStatus> loadApiServer() async {
    await _store.reloadAdvanced();
    final advanced = _store.advanced;
    try {
      final info = await applyApiServerSettings(advanced);
      return ApiServerStatus(
        enabled: advanced.apiServerEnabled,
        host: advanced.apiServerHost,
        port: info?.port ?? advanced.apiServerPort,
        baseUrl: info?.baseUrl,
      );
    } catch (_) {
      return ApiServerStatus(
        enabled: advanced.apiServerEnabled,
        host: advanced.apiServerHost,
        port: advanced.apiServerPort,
        bindErrorCode: AppErrorCode.apiServerBindFailed.wireName,
      );
    }
  }

  Future<ApiServerStatus> setApiServerEnabled(bool enabled) async {
    await _store.updateAdvanced(
      AdvancedSettingsPatch(apiServerEnabled: enabled),
    );
    return loadApiServer();
  }

  Future<ApiServerStatus> setApiServerPort(int port) async {
    if (port < 0 || port > 65535) {
      return ApiServerStatus(
        enabled: _store.advanced.apiServerEnabled,
        host: _store.advanced.apiServerHost,
        port: port,
        bindErrorCode: AppErrorCode.invalidPort.wireName,
      );
    }
    await _store.updateAdvanced(AdvancedSettingsPatch(apiServerPort: port));
    return loadApiServer();
  }

  Future<NetworkSettings> loadNetworkSettings() async {
    await _store.reloadAdvanced();
    final advanced = _store.advanced;
    return NetworkSettings(
      proxyMode: switch (advanced.proxyMode) {
        'direct' => NetworkProxyMode.direct,
        'custom' => NetworkProxyMode.custom,
        _ => NetworkProxyMode.system,
      },
      proxyUrl: advanced.proxyUrl,
      proxyBypass: advanced.proxyBypass,
      checkUpdatesOnLaunch: advanced.checkUpdatesOnLaunch,
    );
  }

  Future<NetworkSettings> saveNetworkSettings(NetworkSettings settings) async {
    await _store.updateAdvanced(
      AdvancedSettingsPatch(
        proxyMode: settings.proxyMode.name,
        proxyUrl: settings.proxyUrl.trim(),
        proxyBypass: settings.proxyBypass.trim(),
        checkUpdatesOnLaunch: settings.checkUpdatesOnLaunch,
      ),
    );
    return loadNetworkSettings();
  }

  Future<PlatformCapabilities> loadCapabilities() async {
    if (Platform.isWindows) return const PlatformCapabilities.windows();
    if (Platform.isMacOS) return const PlatformCapabilities.macos();
    return const PlatformCapabilities.windows();
  }

  Future<AboutInfo> loadAbout() async => AboutInfo(
    appName: 'LinguaRay',
    version: Env.instance.appVersion,
    buildNumber: '${Env.instance.appBuildNumber}',
    platformLabel: Platform.operatingSystem,
    license: 'MIT',
  );
}
