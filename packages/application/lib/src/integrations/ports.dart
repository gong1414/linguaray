import 'package:linguaray_application/src/settings/models.dart';

abstract interface class IntegrationSettingsRepository {
  Future<ApiServerStatus> loadApiServer();

  Future<ApiServerStatus> setApiServerEnabled(bool enabled);

  Future<ApiServerStatus> setApiServerPort(int port);

  Future<NetworkSettings> loadNetworkSettings();

  Future<NetworkSettings> saveNetworkSettings(NetworkSettings settings);
}
