import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/app/dependencies.dart';
import 'package:linguaray_desktop/src/features/updates/update_coordinator.dart';

const _manifest = UpdateManifest(
  version: '0.7.0',
  notes: 'Test',
  assetName: 'test.zip',
  assetUrl: 'https://example.invalid/test.zip',
  checksumSha256: 'fixture',
);

void main() {
  late _Repository repository;
  late _Installer installer;
  late ProviderContainer container;
  setUp(() {
    repository = _Repository();
    installer = _Installer();
    container = ProviderContainer(
      overrides: [
        updateCurrentVersionProvider.overrideWithValue('0.6.1'),
        updateRepositoryProvider.overrideWithValue(repository),
        updateInstallerProvider.overrideWithValue(installer),
      ],
    );
  });
  tearDown(() => container.dispose());

  test('a synchronous state listener cannot start a second request', () async {
    final coordinator = container.read(updateCoordinatorProvider.notifier);
    Future<void>? repeated;
    final listener = container.listen(updateCoordinatorProvider, (_, next) {
      if (next.status == UpdateStatus.checking) repeated = coordinator.check();
    });
    final first = coordinator.check();
    await first;
    expect(identical(first, repeated), isTrue);
    expect(repository.checks, 1);
    listener.close();
  });

  test('automatic and manual checks share one request and state', () async {
    repository.checkPending = Completer<UpdateManifest?>();
    final coordinator = container.read(updateCoordinatorProvider.notifier);
    final automatic = coordinator.check();
    final manual = coordinator.check();
    expect(identical(automatic, manual), isTrue);
    await Future<void>.delayed(Duration.zero);
    expect(repository.checks, 1);
    expect(
      container.read(updateCoordinatorProvider).status,
      UpdateStatus.checking,
    );
    repository.checkPending!.complete(_manifest);
    await automatic;
    expect(
      container.read(updateCoordinatorProvider).status,
      UpdateStatus.available,
    );
  });

  test(
    'checks during download preserve progress and the verified installer',
    () async {
      final coordinator = container.read(updateCoordinatorProvider.notifier);
      await coordinator.check();
      repository.downloadPending = Completer<String>();
      final download = coordinator.download();
      await Future<void>.delayed(Duration.zero);
      repository.progress!(0.5);
      final check = coordinator.check();
      expect(identical(download, check), isTrue);
      expect(repository.checks, 1);
      expect(container.read(updateCoordinatorProvider).progress, 0.5);
      repository.downloadPending!.complete('/fixture/update.zip');
      await download;
      expect(repository.verifications, ['manifest', 'checksum', 'platform']);
      expect(container.read(updateCoordinatorProvider).canInstall, isTrue);
      await coordinator.check();
      expect(repository.checks, 1);
      expect(
        container.read(updateCoordinatorProvider).downloadedPath,
        '/fixture/update.zip',
      );
    },
  );

  test(
    'closing and reopening the observer preserves ready-to-install state',
    () async {
      final firstWindow = container.listen(
        updateCoordinatorProvider,
        (_, _) {},
      );
      final coordinator = container.read(updateCoordinatorProvider.notifier);
      await coordinator.check();
      await coordinator.download();
      firstWindow.close();
      await container.pump();
      final secondWindow = container.listen(
        updateCoordinatorProvider,
        (_, _) {},
      );
      expect(
        identical(
          container.read(updateCoordinatorProvider.notifier),
          coordinator,
        ),
        isTrue,
      );
      expect(secondWindow.read().canInstall, isTrue);
      secondWindow.close();
    },
  );

  test('a failed download can retry without another check', () async {
    final coordinator = container.read(updateCoordinatorProvider.notifier);
    await coordinator.check();
    repository.failDownload = true;
    await coordinator.download();
    expect(
      container.read(updateCoordinatorProvider).status,
      UpdateStatus.failed,
    );
    repository.failDownload = false;
    await coordinator.download();
    expect(container.read(updateCoordinatorProvider).canInstall, isTrue);
    expect(repository.checks, 1);
  });

  test(
    'repeated install actions share one handoff and expose a retryable failure',
    () async {
      final coordinator = container.read(updateCoordinatorProvider.notifier);
      await coordinator.check();
      await coordinator.download();
      installer.pending = Completer<void>();
      final first = coordinator.install();
      final second = coordinator.install();
      expect(identical(first, second), isTrue);
      expect(installer.calls, 1);
      installer.pending!.completeError(StateError('injected handoff failure'));
      await first;
      expect(
        container.read(updateCoordinatorProvider).errorCode,
        AppErrorCode.updateInstallFailed.wireName,
      );
      await coordinator.download();
      expect(container.read(updateCoordinatorProvider).canInstall, isTrue);
    },
  );

  test(
    'completion after container disposal does not publish to a dead observer',
    () async {
      final scoped = ProviderContainer(
        overrides: [
          updateCurrentVersionProvider.overrideWithValue('0.6.1'),
          updateRepositoryProvider.overrideWithValue(repository),
        ],
      );
      repository.checkPending = Completer<UpdateManifest?>();
      final pending = scoped.read(updateCoordinatorProvider.notifier).check();
      await Future<void>.delayed(Duration.zero);
      scoped.dispose();
      repository.checkPending!.complete(_manifest);
      await pending;
    },
  );
}

final class _Repository implements UpdateRepository {
  int checks = 0;
  bool failDownload = false;
  Completer<UpdateManifest?>? checkPending;
  Completer<String>? downloadPending;
  void Function(double)? progress;
  final List<String> verifications = [];

  @override
  Future<String> currentVersion() async => '0.6.1';
  @override
  Future<UpdateManifest?> checkLatest() async {
    checks++;
    return checkPending == null ? _manifest : await checkPending!.future;
  }

  @override
  Future<String> download({
    required UpdateManifest manifest,
    void Function(double)? onProgress,
  }) async {
    progress = onProgress;
    if (failDownload) throw StateError('injected download failure');
    return downloadPending == null
        ? '/fixture/update.zip'
        : await downloadPending!.future;
  }

  @override
  Future<void> verifyManifest(UpdateManifest manifest) async =>
      verifications.add('manifest');
  @override
  Future<void> verifyChecksum({
    required String filePath,
    required String sha256,
  }) async => verifications.add('checksum');
  @override
  Future<void> verifyPlatformSignature({
    required String filePath,
    required UpdateManifest manifest,
  }) async => verifications.add('platform');
}

final class _Installer implements UpdateInstaller {
  int calls = 0;
  Completer<void>? pending;
  @override
  Future<void> handOff({
    required String filePath,
    required UpdateManifest manifest,
  }) async {
    calls++;
    await pending?.future;
  }
}
