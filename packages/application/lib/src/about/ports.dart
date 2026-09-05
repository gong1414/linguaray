import 'package:linguaray_application/src/settings/models.dart';

abstract interface class AppInfoRepository {
  Future<AboutInfo> loadAbout();

  Future<PlatformCapabilities> loadCapabilities();
}
