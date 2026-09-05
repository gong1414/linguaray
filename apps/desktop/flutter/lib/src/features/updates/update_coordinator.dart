import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../app/dependencies.dart';

final updateCoordinatorProvider =
    NotifierProvider<UpdateCoordinator, UpdateState>(UpdateCoordinator.new);

/// One resident state owner for automatic checks, user checks and installation.
/// Window disposal does not discard a verified download or create a new request.
final class UpdateCoordinator extends Notifier<UpdateState> {
  Future<void>? _pending;

  @override
  UpdateState build() =>
      UpdateState.idle(ref.read(updateCurrentVersionProvider));

  Future<void> check() {
    if (_pending != null) return _pending!;
    if (state.status == UpdateStatus.readyToInstall) return Future.value();
    return _run(_check);
  }

  Future<void> _check() async {
    final check = ref.read(checkForUpdateProvider);
    state = UpdateState(
      status: UpdateStatus.checking,
      currentVersion: state.currentVersion,
    );
    final result = await check();
    if (ref.mounted) state = result;
  }

  Future<void> download() {
    if (_pending != null) return _pending!;
    final manifest = state.manifest;
    if (manifest == null || state.canInstall) return Future.value();
    return _run(() => _download(manifest));
  }

  Future<void> _download(UpdateManifest manifest) async {
    final download = ref.read(downloadVerifiedUpdateProvider);
    final currentVersion = state.currentVersion;
    void progress(double value) {
      if (!ref.mounted) return;
      state = UpdateState(
        status: UpdateStatus.downloading,
        currentVersion: currentVersion,
        manifest: manifest,
        progress: value,
      );
    }

    progress(0);
    final result = await download(
      currentVersion: currentVersion,
      manifest: manifest,
      onProgress: progress,
    );
    if (ref.mounted) state = result;
  }

  Future<void> install() {
    if (_pending != null) return _pending!;
    final manifest = state.manifest;
    final path = state.downloadedPath;
    if (manifest == null || path == null || !state.canInstall) {
      return Future.value();
    }
    return _run(() => _install(path, manifest));
  }

  Future<void> _run(Future<void> Function() operation) {
    // Reserve ownership before publishing state: a synchronous listener may
    // issue another command while the first command publishes its progress.
    final completion = Completer<void>();
    _pending = completion.future;
    unawaited(
      Future<void>.sync(operation).then(
        (_) {
          _pending = null;
          completion.complete();
        },
        onError: (Object error, StackTrace stack) {
          _pending = null;
          completion.completeError(error, stack);
        },
      ),
    );
    return completion.future;
  }

  Future<void> _install(String path, UpdateManifest manifest) async {
    final currentVersion = state.currentVersion;
    try {
      await ref
          .read(updateInstallerProvider)
          .handOff(filePath: path, manifest: manifest);
    } catch (_) {
      if (!ref.mounted) return;
      state = UpdateState(
        status: UpdateStatus.failed,
        currentVersion: currentVersion,
        manifest: manifest,
        errorCode: AppErrorCode.updateInstallFailed.wireName,
      );
    }
  }
}
