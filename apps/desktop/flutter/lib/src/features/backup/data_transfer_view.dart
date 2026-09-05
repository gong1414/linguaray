import 'package:flutter/material.dart';

import '../../shared/settings_page.dart';
import '../../shared/status_message.dart';
import 'data_transfer_view_model.dart';

final class DataTransferViewLabels {
  const DataTransferViewLabels({
    required this.title,
    required this.description,
    required this.exportTitle,
    required this.exportDescription,
    required this.exportAction,
    required this.restoreTitle,
    required this.restoreDescription,
    required this.restoreAction,
    required this.secretsNotice,
    required this.working,
    required this.exported,
    required this.restored,
    required this.failed,
  });

  final String title;
  final String description;
  final String exportTitle;
  final String exportDescription;
  final String exportAction;
  final String restoreTitle;
  final String restoreDescription;
  final String restoreAction;
  final String secretsNotice;
  final String working;
  final String exported;
  final String restored;
  final String failed;
}

class DataTransferView extends StatelessWidget {
  const DataTransferView({
    required this.labels,
    required this.state,
    required this.onExport,
    required this.onRestore,
    super.key,
  });

  final DataTransferViewLabels labels;
  final DataTransferViewState state;
  final VoidCallback onExport;
  final VoidCallback onRestore;

  @override
  Widget build(BuildContext context) {
    final busy = state.busy;
    return SettingsPage(
      title: labels.title,
      children: [
        Text(labels.description),
        const SizedBox(height: 20),
        ListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(labels.exportTitle),
          subtitle: Text(labels.exportDescription),
          trailing: FilledButton.tonalIcon(
            onPressed: busy ? null : onExport,
            icon: const Icon(Icons.file_upload_outlined),
            label: Text(labels.exportAction),
          ),
        ),
        const Divider(),
        ListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(labels.restoreTitle),
          subtitle: Text(labels.restoreDescription),
          trailing: OutlinedButton.icon(
            onPressed: busy ? null : onRestore,
            icon: const Icon(Icons.file_download_outlined),
            label: Text(labels.restoreAction),
          ),
        ),
        const SizedBox(height: 16),
        Text(labels.secretsNotice),
        if (state.operation == DataTransferOperation.exporting ||
            state.operation == DataTransferOperation.restoring) ...[
          const SizedBox(height: 16),
          StatusMessage(kind: StatusKind.progress, title: labels.working),
        ],
        if (state.operation == DataTransferOperation.exported ||
            state.operation == DataTransferOperation.restored) ...[
          const SizedBox(height: 16),
          StatusMessage(
            kind: StatusKind.success,
            title: state.operation == DataTransferOperation.exported
                ? labels.exported
                : labels.restored,
            body: state.selectedPath,
          ),
        ],
        if (state.operation == DataTransferOperation.failed) ...[
          const SizedBox(height: 16),
          StatusMessage(
            kind: StatusKind.error,
            title: labels.failed,
            body: state.error,
          ),
        ],
      ],
    );
  }
}
