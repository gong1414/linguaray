import 'package:linguaray_application/src/updates/models.dart';

abstract interface class UpdateRepository {
  Future<String> currentVersion();

  Future<UpdateManifest?> checkLatest();

  Future<String> download({
    required UpdateManifest manifest,
    void Function(double progress)? onProgress,
  });

  Future<void> verifyChecksum({
    required String filePath,
    required String sha256,
  });

  Future<void> verifyPlatformSignature({required String filePath});
}

abstract interface class UpdateInstaller {
  Future<void> handOff({
    required String filePath,
    required UpdateManifest manifest,
  });
}
