import 'package:linguaray_application/linguaray_application.dart';
import 'package:test/test.dart';

void main() {
  test('compares semantic versions', () {
    expect(isNewerVersion('0.6.0', '0.5.0'), isTrue);
    expect(isNewerVersion('v1.0.0', '0.9.9'), isTrue);
    expect(isNewerVersion('0.5.0', '0.5.0'), isFalse);
    expect(isNewerVersion('0.4.9', '0.5.0'), isFalse);
  });

  test(
    'check for update reports missing checksum without claiming safety',
    () async {
      final repository = _FakeUpdateRepository(
        latest: const UpdateManifest(
          version: '0.6.0',
          notes: 'New release',
          assetName: 'LinguaRay-macos.zip',
          assetUrl: 'https://example.invalid/app.zip',
        ),
      );

      final state = await CheckForUpdate(repository)();
      expect(state.status, UpdateStatus.available);
      expect(state.errorCode, AppErrorCode.updateChecksumMissing.wireName);
      expect(state.canInstall, isFalse);
    },
  );

  test('download refuses a missing checksum', () async {
    const manifest = UpdateManifest(
      version: '0.6.0',
      notes: '',
      assetName: 'LinguaRay-macos.zip',
      assetUrl: 'https://example.invalid/app.zip',
    );
    final state = await DownloadVerifiedUpdate(_FakeUpdateRepository())(
      currentVersion: '0.5.0',
      manifest: manifest,
    );
    expect(state.status, UpdateStatus.failed);
    expect(state.errorCode, AppErrorCode.updateChecksumMissing.wireName);
  });
}

final class _FakeUpdateRepository implements UpdateRepository {
  _FakeUpdateRepository({this.latest});

  final UpdateManifest? latest;

  @override
  Future<UpdateManifest?> checkLatest() async => latest;

  @override
  Future<String> currentVersion() async => '0.5.0';

  @override
  Future<String> download({
    required UpdateManifest manifest,
    void Function(double progress)? onProgress,
  }) async => '/tmp/app.zip';

  @override
  Future<void> verifyChecksum({
    required String filePath,
    required String sha256,
  }) async {}
}
