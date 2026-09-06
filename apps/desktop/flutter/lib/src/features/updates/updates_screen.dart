import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../i18n/i18n.dart';
import '../../shared/i18n_labels.dart';
import 'update_coordinator.dart';
import 'updates_view.dart';

class UpdatesSettingsScreen extends ConsumerWidget {
  const UpdatesSettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(updateCoordinatorProvider);
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
          unawaited(ref.read(updateCoordinatorProvider.notifier).check()),
      onDownload: () =>
          unawaited(ref.read(updateCoordinatorProvider.notifier).download()),
      onInstall: () =>
          unawaited(ref.read(updateCoordinatorProvider.notifier).install()),
    );
  }
}
