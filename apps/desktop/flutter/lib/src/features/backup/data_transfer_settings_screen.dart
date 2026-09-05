import 'dart:async';

import 'package:flutter/material.dart';

import '../../i18n/i18n.dart';
import '../../shared/settings_page.dart';
import '../../shared/status_message.dart';
import 'data_transfer_controller.dart';

class DataTransferSettingsScreen extends StatefulWidget {
  const DataTransferSettingsScreen({super.key});

  @override
  State<DataTransferSettingsScreen> createState() =>
      _DataTransferSettingsScreenState();
}

class _DataTransferSettingsScreenState
    extends State<DataTransferSettingsScreen> {
  @override
  void initState() {
    super.initState();
    dataTransferController.addListener(_changed);
  }

  @override
  void dispose() {
    dataTransferController.removeListener(_changed);
    super.dispose();
  }

  void _changed() => setState(() {});

  @override
  Widget build(BuildContext context) {
    final labels = t.settings.data_transfer;
    final state = dataTransferController.operation;
    final busy =
        state == DataTransferOperation.exporting ||
        state == DataTransferOperation.restoring;
    return SettingsPage(
      title: labels.title,
      children: [
        Text(labels.description),
        const SizedBox(height: 20),
        ListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(labels.export_title),
          subtitle: Text(labels.export_description),
          trailing: FilledButton.tonalIcon(
            onPressed: busy
                ? null
                : () => unawaited(dataTransferController.exportBackup()),
            icon: const Icon(Icons.file_upload_outlined),
            label: Text(labels.export_action),
          ),
        ),
        const Divider(),
        ListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(labels.restore_title),
          subtitle: Text(labels.restore_description),
          trailing: OutlinedButton.icon(
            onPressed: busy ? null : _confirmRestore,
            icon: const Icon(Icons.file_download_outlined),
            label: Text(labels.restore_action),
          ),
        ),
        const SizedBox(height: 16),
        Text(labels.secrets_notice),
        if (state == DataTransferOperation.exporting ||
            state == DataTransferOperation.restoring) ...[
          const SizedBox(height: 16),
          StatusMessage(kind: StatusKind.progress, title: labels.working),
        ],
        if (state == DataTransferOperation.exported ||
            state == DataTransferOperation.restored) ...[
          const SizedBox(height: 16),
          StatusMessage(
            kind: StatusKind.success,
            title: state == DataTransferOperation.exported
                ? labels.exported
                : labels.restored,
            body: dataTransferController.selectedPath,
          ),
        ],
        if (state == DataTransferOperation.failed) ...[
          const SizedBox(height: 16),
          StatusMessage(
            kind: StatusKind.error,
            title: labels.failed,
            body: dataTransferController.error,
          ),
        ],
      ],
    );
  }

  Future<void> _confirmRestore() async {
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
      unawaited(dataTransferController.restoreBackup());
    }
  }
}
