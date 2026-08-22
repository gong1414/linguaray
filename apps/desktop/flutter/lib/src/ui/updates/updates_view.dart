import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../shared/status_message.dart';

final class UpdatesViewLabels {
  const UpdatesViewLabels({
    required this.title,
    required this.current,
    required this.check,
    required this.checking,
    required this.upToDate,
    required this.available,
    required this.download,
    required this.downloading,
    required this.ready,
    required this.install,
    required this.unsigned,
    required this.notes,
    required this.retry,
    this.errorMessage,
  });

  final String title;
  final String current;
  final String check;
  final String checking;
  final String upToDate;
  final String Function(String version) available;
  final String download;
  final String downloading;
  final String ready;
  final String install;
  final String unsigned;
  final String notes;
  final String retry;
  final String Function(String? code)? errorMessage;
}

class UpdatesView extends StatelessWidget {
  const UpdatesView({
    required this.labels,
    required this.state,
    required this.onCheck,
    required this.onDownload,
    required this.onInstall,
    super.key,
  });

  final UpdatesViewLabels labels;
  final UpdateState state;
  final VoidCallback onCheck;
  final VoidCallback onDownload;
  final VoidCallback onInstall;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Material(
      color: theme.scaffoldBackgroundColor,
      child: ListView(
        padding: const EdgeInsets.fromLTRB(24, 16, 24, 32),
        children: [
          Text(labels.title, style: theme.textTheme.headlineMedium),
          const SizedBox(height: 12),
          ListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(labels.current),
            subtitle: Text(state.currentVersion),
            trailing: FilledButton(
              onPressed: state.status == UpdateStatus.checking ? null : onCheck,
              child: Text(
                state.status == UpdateStatus.checking
                    ? labels.checking
                    : labels.check,
              ),
            ),
          ),
          const SizedBox(height: 12),
          ..._status(context),
        ],
      ),
    );
  }

  List<Widget> _status(BuildContext context) {
    switch (state.status) {
      case UpdateStatus.idle:
        return const [];
      case UpdateStatus.checking:
        return [const StatusMessage(kind: StatusKind.progress, title: '')];
      case UpdateStatus.upToDate:
        return [
          StatusMessage(kind: StatusKind.success, title: labels.upToDate),
        ];
      case UpdateStatus.available:
        final manifest = state.manifest;
        return [
          StatusMessage(
            kind: manifest?.hasChecksum == true
                ? StatusKind.info
                : StatusKind.warning,
            title: labels.available(manifest?.version ?? ''),
            body: manifest?.hasChecksum == true
                ? manifest?.notes
                : labels.unsigned,
            action: manifest?.hasChecksum == true
                ? FilledButton(
                    onPressed: onDownload,
                    child: Text(labels.download),
                  )
                : null,
          ),
        ];
      case UpdateStatus.downloading:
        return [
          StatusMessage(
            kind: StatusKind.progress,
            title: labels.downloading,
            body: state.progress == null
                ? null
                : '${(state.progress! * 100).round()}%',
          ),
        ];
      case UpdateStatus.readyToInstall:
        return [
          StatusMessage(
            kind: StatusKind.success,
            title: labels.ready,
            action: FilledButton(
              onPressed: state.canInstall ? onInstall : null,
              child: Text(labels.install),
            ),
          ),
        ];
      case UpdateStatus.failed:
        return [
          StatusMessage(
            kind: StatusKind.error,
            title: labels.errorMessage?.call(state.errorCode) ?? labels.retry,
            action: OutlinedButton(
              onPressed: onCheck,
              child: Text(labels.retry),
            ),
          ),
        ];
    }
  }
}
