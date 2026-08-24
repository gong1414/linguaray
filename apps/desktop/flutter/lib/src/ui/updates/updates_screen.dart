import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../config/dependencies.dart';
import '../../i18n/i18n.dart';
import '../../utils/env.dart';
import '../../platform/startup_update_controller.dart';
import '../i18n_labels.dart';
import 'updates_view.dart';

final updatesViewModelProvider =
    NotifierProvider<UpdatesViewModel, UpdateState>(UpdatesViewModel.new);

final class UpdatesViewModel extends Notifier<UpdateState> {
  @override
  UpdateState build() {
    return startupUpdateController.result.value ??
        UpdateState.idle(Env.instance.appVersion);
  }

  Future<void> check() async {
    state = UpdateState(
      status: UpdateStatus.checking,
      currentVersion: state.currentVersion,
    );
    state = await ref.read(checkForUpdateProvider)();
  }

  Future<void> download() async {
    final manifest = state.manifest;
    if (manifest == null) return;
    state = UpdateState(
      status: UpdateStatus.downloading,
      currentVersion: state.currentVersion,
      manifest: manifest,
      progress: 0,
    );
    state = await ref.read(downloadVerifiedUpdateProvider)(
      currentVersion: state.currentVersion,
      manifest: manifest,
      onProgress: (value) {
        state = UpdateState(
          status: UpdateStatus.downloading,
          currentVersion: state.currentVersion,
          manifest: manifest,
          progress: value,
        );
      },
    );
  }

  Future<void> install() async {
    final manifest = state.manifest;
    final path = state.downloadedPath;
    if (manifest == null || path == null || !state.canInstall) return;
    try {
      await ref
          .read(updateInstallerProvider)
          .handOff(filePath: path, manifest: manifest);
    } catch (_) {
      state = UpdateState(
        status: UpdateStatus.failed,
        currentVersion: state.currentVersion,
        manifest: manifest,
        errorCode: AppErrorCode.updateInstallFailed.wireName,
      );
    }
  }
}

class UpdatesSettingsScreen extends ConsumerWidget {
  const UpdatesSettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(updatesViewModelProvider);
    final labels = t.ui.updates;
    return UpdatesView(
      labels: UpdatesViewLabels(
        title: labels.title,
        current: labels.current,
        check: labels.check,
        checking: labels.checking,
        upToDate: labels.up_to_date,
        available: (version) => labels.available(version: version),
        download: labels.download,
        downloading: labels.downloading,
        ready: labels.ready,
        install: labels.install,
        unsigned: labels.unsigned,
        notes: labels.notes,
        retry: t.workbench.history_page.retry,
        errorMessage: appErrorMessage,
      ),
      state: state,
      onCheck: () =>
          unawaited(ref.read(updatesViewModelProvider.notifier).check()),
      onDownload: () =>
          unawaited(ref.read(updatesViewModelProvider.notifier).download()),
      onInstall: () =>
          unawaited(ref.read(updatesViewModelProvider.notifier).install()),
    );
  }
}
