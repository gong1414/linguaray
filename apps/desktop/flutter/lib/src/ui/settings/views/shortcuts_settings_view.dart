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
    this.title,
    this.additionalChildren = const [],
    this.descriptionBuilder,
  });

  final ShortcutsSettingsLabels labels;
  final String? title;
  final List<ShortcutRecord> shortcuts;
  final String? recordingActionId;
  final ValueChanged<String> onStartRecording;
  final VoidCallback onCancelRecording;
  final ValueChanged<String> onClear;
  final VoidCallback onReset;
  final List<Widget> additionalChildren;
  final String Function(String actionId)? descriptionBuilder;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final cardColor = theme.colorScheme.surfaceContainerLowest;
    return ListView(
      padding: const EdgeInsets.fromLTRB(32, 36, 32, 36),
      children: [
        Text(title ?? labels.title, style: theme.textTheme.titleLarge),
        const SizedBox(height: 22),
        Row(
          children: [
            if ((title ?? labels.title) != labels.title)
              Expanded(
                child: Text(
                  labels.title,
                  style: theme.textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.w600,
                  ),
                ),
              )
            else
              const Spacer(),
            TextButton(onPressed: onReset, child: Text(labels.reset)),
          ],
        ),
        const SizedBox(height: 8),
        DecoratedBox(
          decoration: BoxDecoration(
            color: cardColor,
            borderRadius: BorderRadius.circular(12),
            border: Border.all(color: theme.colorScheme.outlineVariant),
          ),
          child: Column(
            children: [
              for (final (index, shortcut) in shortcuts.indexed) ...[
                if (index > 0)
                  Divider(
                    height: 1,
                    indent: 12,
                    endIndent: 12,
                    color: theme.dividerColor.withValues(alpha: 0.5),
                  ),
                _ShortcutRow(
                  labels: labels,
                  shortcut: shortcut,
                  recording: recordingActionId == shortcut.actionId,
                  onStartRecording: onStartRecording,
                  onCancelRecording: onCancelRecording,
                  onClear: onClear,
                  descriptionBuilder: descriptionBuilder,
                ),
              ],
            ],
          ),
        ),
        ...additionalChildren,
      ],
    );
  }
}

class _ShortcutRow extends StatelessWidget {
  const _ShortcutRow({
    required this.labels,
    required this.shortcut,
    required this.recording,
    required this.onStartRecording,
    required this.onCancelRecording,
    required this.onClear,
    this.descriptionBuilder,
  });

  final ShortcutsSettingsLabels labels;
  final ShortcutRecord shortcut;
  final bool recording;
  final ValueChanged<String> onStartRecording;
  final VoidCallback onCancelRecording;
  final ValueChanged<String> onClear;
  final String Function(String actionId)? descriptionBuilder;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.fromLTRB(12, 8, 10, 8),
    child: Row(
      children: [
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                shortcut.labelKey,
                style: Theme.of(context).textTheme.bodyMedium
                    ?.copyWith(fontWeight: FontWeight.w600),
              ),
              const SizedBox(height: 2),
              Text(
                _subtitle(shortcut),
                style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
              ),
            ],
          ),
        ),
        TapRegion(
          onTapOutside: recording ? (_) => onCancelRecording() : null,
          child: OutlinedButton(
            onPressed: recording
                ? onCancelRecording
                : () => onStartRecording(shortcut.actionId),
            child: Text(
              recording
                  ? labels.recording
                  : shortcut.accelerator.isEmpty
                  ? labels.record
                  : shortcut.accelerator,
            ),
          ),
        ),
        const SizedBox(width: 3),
        IconButton(
          tooltip: labels.clear,
          onPressed: shortcut.accelerator.isEmpty
              ? null
              : () => onClear(shortcut.actionId),
          icon: const Icon(Icons.cancel_rounded, size: 17),
        ),
      ],
    ),
  );

  String _status(ShortcutRecord shortcut) => switch (shortcut.status) {
    ShortcutStatus.registered => labels.registered,
    ShortcutStatus.unregistered => labels.unregistered,
    ShortcutStatus.recording => labels.recording,
    ShortcutStatus.invalid => labels.invalid,
    ShortcutStatus.localDuplicate || ShortcutStatus.osConflict =>
      labels.conflict(shortcut.conflictReason ?? shortcut.accelerator),
  };

  String _subtitle(ShortcutRecord shortcut) {
    if (shortcut.status == ShortcutStatus.registered) {
      final description = descriptionBuilder?.call(shortcut.actionId) ?? '';
      if (description.isNotEmpty) return description;
    }
    return _status(shortcut);
  }
}
