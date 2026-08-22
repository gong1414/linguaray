import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../settings_labels.dart';

class ShortcutsSettingsView extends StatelessWidget {
  const ShortcutsSettingsView({
    required this.labels,
    required this.shortcuts,
    required this.recordingActionId,
    required this.onStartRecording,
    required this.onCancelRecording,
    required this.onClear,
    required this.onReset,
    super.key,
  });

  final ShortcutsSettingsLabels labels;
  final List<ShortcutRecord> shortcuts;
  final String? recordingActionId;
  final ValueChanged<String> onStartRecording;
  final VoidCallback onCancelRecording;
  final ValueChanged<String> onClear;
  final VoidCallback onReset;

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.fromLTRB(20, 16, 24, 24),
      children: [
        Row(
          children: [
            Expanded(
              child: Text(
                labels.title,
                style: Theme.of(context).textTheme.titleLarge,
              ),
            ),
            OutlinedButton(onPressed: onReset, child: Text(labels.reset)),
          ],
        ),
        const SizedBox(height: 12),
        for (final shortcut in shortcuts)
          ListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(shortcut.labelKey),
            subtitle: Text(_status(shortcut)),
            trailing: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                TapRegion(
                  onTapOutside: recordingActionId == shortcut.actionId
                      ? (_) => onCancelRecording()
                      : null,
                  child: OutlinedButton(
                    onPressed: recordingActionId == shortcut.actionId
                        ? onCancelRecording
                        : () => onStartRecording(shortcut.actionId),
                    child: Text(
                      recordingActionId == shortcut.actionId
                          ? labels.recording
                          : shortcut.accelerator.isEmpty
                          ? labels.record
                          : shortcut.accelerator,
                    ),
                  ),
                ),
                IconButton(
                  tooltip: labels.clear,
                  onPressed: shortcut.accelerator.isEmpty
                      ? null
                      : () => onClear(shortcut.actionId),
                  icon: const Icon(Icons.backspace_outlined),
                ),
              ],
            ),
          ),
      ],
    );
  }

  String _status(ShortcutRecord shortcut) {
    return switch (shortcut.status) {
      ShortcutStatus.registered => labels.registered,
      ShortcutStatus.unregistered => labels.unregistered,
      ShortcutStatus.recording => labels.recording,
      ShortcutStatus.invalid => labels.invalid,
      ShortcutStatus.localDuplicate || ShortcutStatus.osConflict =>
        labels.conflict(shortcut.conflictReason ?? shortcut.accelerator),
    };
  }
}
