import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../i18n/i18n.dart';
import 'data_transfer_view.dart';
import 'data_transfer_view_model.dart';

class DataTransferSettingsScreen extends ConsumerWidget {
  const DataTransferSettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final labels = t.settings.data_transfer;
    return DataTransferView(
      labels: DataTransferViewLabels(
        title: labels.title,
        description: labels.description,
        exportTitle: labels.export_title,
        exportDescription: labels.export_description,
        exportAction: labels.export_action,
        restoreTitle: labels.restore_title,
        restoreDescription: labels.restore_description,
        restoreAction: labels.restore_action,
        secretsNotice: labels.secrets_notice,
        working: labels.working,
        exported: labels.exported,
        restored: labels.restored,
        failed: labels.failed,
      ),
      state: ref.watch(dataTransferViewModelProvider),
      onExport: () => unawaited(
        ref.read(dataTransferViewModelProvider.notifier).exportBackup(),
      ),
      onRestore: () => unawaited(_confirmRestore(context, ref)),
    );
  }
}

Future<void> _confirmRestore(BuildContext context, WidgetRef ref) async {
  final labels = t.settings.data_transfer;
  final confirmed = await showDialog<bool>(
    context: context,
    builder: (context) => AlertDialog(
      title: Text(labels.restore_confirm_title),
      content: Text(labels.restore_confirm_description),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context, false),
          child: Text(t.common.ui.button.cancel),
        ),
        FilledButton(
          onPressed: () => Navigator.pop(context, true),
          child: Text(labels.restore_action),
        ),
      ],
    ),
  );
  if (confirmed == true) {
    unawaited(ref.read(dataTransferViewModelProvider.notifier).restoreBackup());
  }
}
