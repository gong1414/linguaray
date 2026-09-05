import 'package:linguaray_application/src/errors/models.dart';
import 'package:linguaray_application/src/updates/models.dart';
import 'package:linguaray_application/src/updates/ports.dart';
import 'package:linguaray_application/src/updates/semver.dart';

final class CheckForUpdate {
  const CheckForUpdate(this._repository);

  final UpdateRepository _repository;

  Future<UpdateState> call() async {
    var current = '';
    try {
      current = await _repository.currentVersion();
      final latest = await _repository.checkLatest();
      if (latest == null || !isNewerVersion(latest.version, current)) {
        return UpdateState(
          status: UpdateStatus.upToDate,
          currentVersion: current,
        );
      }
      return UpdateState(
        status: UpdateStatus.available,
        currentVersion: current,
        manifest: latest,
        errorCode: latest.hasChecksum
            ? null
            : AppErrorCode.updateChecksumMissing.wireName,
      );
    } on AppFailure catch (error) {
      return UpdateState(
        status: UpdateStatus.failed,
        currentVersion: current,
        errorCode: error.wireName,
      );
    } catch (_) {
      return UpdateState(
        status: UpdateStatus.failed,
        currentVersion: current,
        errorCode: AppErrorCode.updateCheckFailed.wireName,
      );
    }
  }
}

final class DownloadVerifiedUpdate {
  const DownloadVerifiedUpdate(this._repository);

  final UpdateRepository _repository;

  Future<UpdateState> call({
    required String currentVersion,
    required UpdateManifest manifest,
    void Function(double progress)? onProgress,
  }) async {
    if (!manifest.hasChecksum) {
      return UpdateState(
        status: UpdateStatus.failed,
        currentVersion: currentVersion,
        manifest: manifest,
        errorCode: AppErrorCode.updateChecksumMissing.wireName,
      );
    }
    try {
      await _repository.verifyManifest(manifest);
      final path = await _repository.download(
        manifest: manifest,
        onProgress: onProgress,
      );
      await _repository.verifyChecksum(
        filePath: path,
        sha256: manifest.checksumSha256!,
      );
      await _repository.verifyPlatformSignature(
        filePath: path,
        manifest: manifest,
      );
      return UpdateState(
        status: UpdateStatus.readyToInstall,
        currentVersion: currentVersion,
        manifest: manifest,
        downloadedPath: path,
      );
    } on AppFailure catch (error) {
      return UpdateState(
        status: UpdateStatus.failed,
        currentVersion: currentVersion,
        manifest: manifest,
        errorCode: error.wireName,
      );
    } catch (_) {
      return UpdateState(
        status: UpdateStatus.failed,
        currentVersion: currentVersion,
        manifest: manifest,
        errorCode: AppErrorCode.updateDownloadFailed.wireName,
      );
    }
  }
}
