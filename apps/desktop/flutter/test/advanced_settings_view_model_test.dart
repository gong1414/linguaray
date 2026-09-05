import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/app/dependencies.dart';
import 'package:linguaray_desktop/src/features/integrations/advanced_settings_view_model.dart';

void main() {
  test('advanced settings load and reject an invalid custom proxy', () async {
    final repository = _FakeIntegration();
    final container = ProviderContainer(
      overrides: [
        integrationSettingsRepositoryProvider.overrideWithValue(repository),
      ],
    );
    addTearDown(container.dispose);
    final subscription = container.listen(
      advancedSettingsViewModelProvider,
      (_, _) {},
      fireImmediately: true,
    );
    addTearDown(subscription.close);
    await _waitFor(
      () => !container.read(advancedSettingsViewModelProvider).loading,
    );

    expect(container.read(advancedSettingsViewModelProvider).api?.port, 9);
    expect(
      container.read(advancedSettingsViewModelProvider).network?.proxyMode,
      NetworkProxyMode.system,
    );

    await container
        .read(advancedSettingsViewModelProvider.notifier)
        .saveNetwork(
          const NetworkSettings(
            proxyMode: NetworkProxyMode.custom,
            proxyUrl: 'not-a-url',
            proxyBypass: '',
            checkUpdatesOnLaunch: true,
          ),
          applyNetwork: () async {},
        );
    expect(
      container.read(advancedSettingsViewModelProvider).networkError,
      'proxy_configuration_invalid',
    );
    expect(repository.saved, isEmpty);
  });
}

Future<void> _waitFor(bool Function() condition) async {
  for (var attempt = 0; attempt < 50; attempt++) {
    if (condition()) return;
    await Future<void>.delayed(const Duration(milliseconds: 1));
  }
  fail('Timed out waiting for advanced settings.');
}

final class _FakeIntegration implements IntegrationSettingsRepository {
  final saved = <NetworkSettings>[];

  @override
  Future<ApiServerStatus> loadApiServer() async =>
      const ApiServerStatus(enabled: true, host: '127.0.0.1', port: 9);

  @override
  Future<ApiServerStatus> setApiServerEnabled(bool enabled) async =>
      loadApiServer();

  @override
  Future<ApiServerStatus> setApiServerPort(int port) async => loadApiServer();

  @override
  Future<NetworkSettings> loadNetworkSettings() async => const NetworkSettings(
    proxyMode: NetworkProxyMode.system,
    proxyUrl: '',
    proxyBypass: 'localhost',
    checkUpdatesOnLaunch: true,
  );

  @override
  Future<NetworkSettings> saveNetworkSettings(NetworkSettings settings) async {
    saved.add(settings);
    return settings;
  }
}
