import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../shared/settings_page.dart';
import '../settings_labels.dart';

class PermissionsSettingsView extends StatelessWidget {
  const PermissionsSettingsView({
    required this.labels,
    required this.snapshot,
    required this.onGrantAccessibility,
    required this.onGrantScreenRecording,
    required this.onRecheck,
    super.key,
  });

  final PermissionsSettingsLabels labels;
  final AccessSnapshot snapshot;
  final VoidCallback onGrantAccessibility;
  final VoidCallback onGrantScreenRecording;
  final VoidCallback onRecheck;

  @override
  Widget build(BuildContext context) {
    final windows =
        snapshot.accessibility == AccessState.notRequired &&
        snapshot.screenRecording == AccessState.notRequired;

    return SettingsPage(
      title: labels.title,
      actions: [
        OutlinedButton(onPressed: onRecheck, child: Text(labels.recheck)),
      ],
      children: [
        if (windows)
          Text(
            labels.windowsNote,
            style: Theme.of(context).textTheme.bodyMedium,
          )
        else ...[
          _PermissionTile(
            title: labels.accessibility,
            hint: labels.accessibilityHint,
            state: snapshot.accessibility,
            labels: labels,
            onGrant: onGrantAccessibility,
          ),
          _PermissionTile(
            title: labels.screenRecording,
            hint: labels.screenRecordingHint,
            state: snapshot.screenRecording,
            labels: labels,
            onGrant: onGrantScreenRecording,
          ),
        ],
      ],
    );
  }
}

class _PermissionTile extends StatelessWidget {
  const _PermissionTile({
    required this.title,
    required this.hint,
    required this.state,
    required this.labels,
    required this.onGrant,
  });

  final String title;
  final String hint;
  final AccessState state;
  final PermissionsSettingsLabels labels;
  final VoidCallback onGrant;

  @override
  Widget build(BuildContext context) {
    final (status, color, grant) = switch (state) {
      AccessState.granted => (
        labels.granted,
        Theme.of(context).colorScheme.primary,
        false,
      ),
      AccessState.denied => (
        labels.denied,
        Theme.of(context).colorScheme.error,
        true,
      ),
      AccessState.notRequired => (
        labels.notRequired,
        Theme.of(context).colorScheme.onSurfaceVariant,
        false,
      ),
      AccessState.checking || AccessState.unknown => (
        labels.unknown,
        Theme.of(context).colorScheme.onSurfaceVariant,
        true,
      ),
    };

    return ListTile(
      contentPadding: EdgeInsets.zero,
      title: Text(title),
      subtitle: Text('$hint\n$status', style: TextStyle(color: color)),
      isThreeLine: true,
      trailing: grant
          ? FilledButton(onPressed: onGrant, child: Text(labels.grant))
          : Icon(Icons.check_circle_outline_rounded, color: color),
    );
  }
}
