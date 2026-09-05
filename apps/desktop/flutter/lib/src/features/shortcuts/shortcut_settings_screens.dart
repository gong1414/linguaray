import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../i18n/i18n.dart';
import '../../shared/i18n_labels.dart';
import '../preferences/general_settings_view_model.dart';
import '../preferences/translation_preference_dialogs.dart';
import 'shortcuts_settings_view.dart';
import 'shortcuts_view_model.dart';

class TranslationSettingsScreen extends StatelessWidget {
  const TranslationSettingsScreen({super.key});

  @override
  Widget build(BuildContext context) => _ShortcutsSettingsScreen(
    title: t.settings.navigation.translation_settings,
    actionIds: const {
      'toggleQuickWindow',
      'translateSelection',
      'captureAndTranslate',
      'openInputWindow',
      'translateInput',
    },
    preferenceKind: _PreferenceKind.translation,
  );
}

class OcrSettingsScreen extends StatelessWidget {
  const OcrSettingsScreen({super.key});

  @override
  Widget build(BuildContext context) => _ShortcutsSettingsScreen(
    title: t.settings.navigation.ocr_settings,
    actionIds: const {
      'captureOcr',
      'silentCaptureOcr',
      'fileOcr',
      'clipboardOcr',
      'showOcrWindow',
    },
    preferenceKind: _PreferenceKind.ocr,
  );
}

enum _PreferenceKind { translation, ocr }

class _ShortcutsSettingsScreen extends ConsumerStatefulWidget {
  const _ShortcutsSettingsScreen({
    this.title,
    this.actionIds,
    this.preferenceKind,
  });

  final String? title;
  final Set<String>? actionIds;
  final _PreferenceKind? preferenceKind;

  @override
  ConsumerState<_ShortcutsSettingsScreen> createState() =>
      _ShortcutsSettingsScreenState();
}

class _ShortcutsSettingsScreenState
    extends ConsumerState<_ShortcutsSettingsScreen> {
  final FocusNode _focusNode = FocusNode(debugLabel: 'shortcut-recorder');

  @override
  void dispose() {
    _focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(shortcutsViewModelProvider);
    final generalState = ref.watch(generalSettingsViewModelProvider);
    final labels = shortcutsSettingsLabels();
    return Focus(
      focusNode: _focusNode,
      autofocus: true,
      onKeyEvent: (node, event) {
        final recording = state.recordingActionId;
        if (recording == null || event is! KeyDownEvent) {
          return KeyEventResult.ignored;
        }
        if (event.logicalKey == LogicalKeyboardKey.escape) {
          unawaited(
            ref.read(shortcutsViewModelProvider.notifier).cancelRecording(),
          );
          return KeyEventResult.handled;
        }
        final accelerator = _acceleratorFrom(event);
        if (accelerator == null) return KeyEventResult.ignored;
        unawaited(
          ref
              .read(shortcutsViewModelProvider.notifier)
              .submitRecording(accelerator),
        );
        return KeyEventResult.handled;
      },
      child: ShortcutsSettingsView(
        labels: labels,
        title: widget.title,
        shortcuts: [
          for (final item in state.shortcuts)
            if (widget.actionIds?.contains(item.actionId) ?? true)
              ShortcutRecord(
                actionId: item.actionId,
                labelKey: shortcutActionLabel(item.actionId),
                accelerator: item.accelerator,
                status: item.status,
                conflictReason: item.conflictReason,
              ),
        ],
        recordingActionId: state.recordingActionId,
        descriptionBuilder: shortcutActionDescription,
        additionalChildren: _preferenceChildren(context, generalState),
        onStartRecording: (id) {
          unawaited(
            ref.read(shortcutsViewModelProvider.notifier).startRecording(id),
          );
          _focusNode.requestFocus();
        },
        onCancelRecording: () {
          unawaited(
            ref.read(shortcutsViewModelProvider.notifier).cancelRecording(),
          );
          _focusNode.requestFocus();
        },
        onClear: (id) =>
            unawaited(ref.read(shortcutsViewModelProvider.notifier).clear(id)),
        onReset: () => unawaited(_confirmReset(context, ref)),
      ),
    );
  }

  List<Widget> _preferenceChildren(
    BuildContext context,
    GeneralSettingsViewState state,
  ) {
    final preferences = state.preferences;
    if (widget.preferenceKind == null) return const [];
    if (preferences == null || state.loading) {
      return const [
        SizedBox(height: 28),
        Center(child: CircularProgressIndicator()),
      ];
    }
    if (widget.preferenceKind == _PreferenceKind.ocr) {
      return [
        const SizedBox(height: 28),
        _PreferenceHeading(t.settings.general.section.ocr_behaviour),
        const SizedBox(height: 8),
        _PreferenceCard(
          children: [
            SwitchListTile(
              contentPadding: const EdgeInsets.symmetric(horizontal: 12),
              title: Text(t.settings.general.row.auto_copy_detected_text),
              value: preferences.autoCopyDetectedText,
              onChanged: (value) => unawaited(
                ref
                    .read(generalSettingsViewModelProvider.notifier)
                    .setAutoCopyDetectedText(value),
              ),
            ),
            ListTile(
              contentPadding: const EdgeInsets.symmetric(horizontal: 12),
              title: Text(t.settings.permissions.title),
              subtitle: Text(t.settings.general.row.screen_capture_access_hint),
              trailing: const Icon(Icons.chevron_right_rounded),
              onTap: () => context.go('/settings/permissions'),
            ),
          ],
        ),
      ];
    }

    return [
      const SizedBox(height: 28),
      _PreferenceHeading(t.settings.general.section.languages),
      const SizedBox(height: 8),
      _PreferenceCard(
        children: [
          ListTile(
            contentPadding: const EdgeInsets.symmetric(horizontal: 12),
            title: Text(t.settings.general.section.translation_target),
            subtitle: Text(
              preferences.translationTargets.isEmpty
                  ? t.settings.general.row.no_translation_targets
                  : preferences.translationTargets
                        .map((rule) => '${rule.source} → ${rule.target}')
                        .join(' · '),
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
            ),
            trailing: TextButton(
              onPressed: () =>
                  unawaited(_editTranslationTargets(context, state)),
              child: Text(t.common.ui.button.manage),
            ),
          ),
          ListTile(
            contentPadding: const EdgeInsets.symmetric(horizontal: 12),
            title: Text(t.settings.general.row.common_languages),
            subtitle: Text(
              state.translationLanguages
                  .where(
                    (language) =>
                        preferences.commonLanguages.contains(language.code),
                  )
                  .map((language) => language.name)
                  .join(' · '),
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
            ),
            trailing: TextButton(
              onPressed: () => unawaited(_editCommonLanguages(context, state)),
              child: Text(t.common.ui.button.manage),
            ),
          ),
        ],
      ),
      const SizedBox(height: 24),
      _PreferenceHeading(t.settings.general.section.translation_behaviour),
      const SizedBox(height: 8),
      _PreferenceCard(
        children: [
          SwitchListTile(
            contentPadding: const EdgeInsets.symmetric(horizontal: 12),
            title: Text(t.settings.general.row.submit_with_enter),
            value: preferences.inputSubmitMode == InputSubmitMode.enter,
            onChanged: (value) => unawaited(
              ref
                  .read(generalSettingsViewModelProvider.notifier)
                  .setInputSubmitMode(
                    value
                        ? InputSubmitMode.enter
                        : InputSubmitMode.commandEnter,
                  ),
            ),
          ),
          SwitchListTile(
            contentPadding: const EdgeInsets.symmetric(horizontal: 12),
            title: Text(t.settings.general.row.double_click_copy_result),
            value: preferences.doubleClickCopyResult,
            onChanged: (value) => unawaited(
              ref
                  .read(generalSettingsViewModelProvider.notifier)
                  .setDoubleClickCopyResult(value),
            ),
          ),
        ],
      ),
    ];
  }

  Future<void> _editTranslationTargets(
    BuildContext context,
    GeneralSettingsViewState state,
  ) async {
    final result = await showTranslationTargetsDialog(context, state);
    if (result == null || !mounted) return;
    await ref
        .read(generalSettingsViewModelProvider.notifier)
        .setTranslationTargets(result);
  }

  Future<void> _editCommonLanguages(
    BuildContext context,
    GeneralSettingsViewState state,
  ) async {
    final result = await showCommonLanguagesDialog(context, state);
    if (result == null || !mounted) return;
    await ref
        .read(generalSettingsViewModelProvider.notifier)
        .setCommonLanguages(result);
  }

  Future<void> _confirmReset(BuildContext context, WidgetRef ref) async {
    final labels = shortcutsSettingsLabels();
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(labels.resetConfirmTitle),
        content: Text(labels.resetConfirmBody),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: Text(labels.cancel),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(labels.confirm),
          ),
        ],
      ),
    );
    if (confirmed == true) {
      await ref.read(shortcutsViewModelProvider.notifier).reset();
    }
  }
}

class _PreferenceHeading extends StatelessWidget {
  const _PreferenceHeading(this.label);

  final String label;

  @override
  Widget build(BuildContext context) => Text(
    label,
    style: Theme.of(context).textTheme.titleMedium
        ?.copyWith(fontWeight: FontWeight.w600),
  );
}

class _PreferenceCard extends StatelessWidget {
  const _PreferenceCard({required this.children});

  final List<Widget> children;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Material(
      color: theme.colorScheme.surfaceContainerLowest,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: theme.colorScheme.outlineVariant),
      ),
      clipBehavior: Clip.antiAlias,
      child: Column(
        children: [
          for (final (index, child) in children.indexed) ...[
            if (index > 0)
              Divider(
                height: 1,
                indent: 12,
                endIndent: 12,
                color: theme.dividerColor.withValues(alpha: 0.5),
              ),
            child,
          ],
        ],
      ),
    );
  }
}

String? _acceleratorFrom(KeyDownEvent event) {
  final key = event.logicalKey;
  if (key == LogicalKeyboardKey.shift ||
      key == LogicalKeyboardKey.control ||
      key == LogicalKeyboardKey.alt ||
      key == LogicalKeyboardKey.meta) {
    return null;
  }
  final parts = <String>[];
  if (HardwareKeyboard.instance.isMetaPressed) parts.add('Command');
  if (HardwareKeyboard.instance.isControlPressed) parts.add('Control');
  if (HardwareKeyboard.instance.isAltPressed) parts.add('Option');
  if (HardwareKeyboard.instance.isShiftPressed) parts.add('Shift');
  if (parts.isEmpty) return null;
  final label = key.keyLabel.isEmpty
      ? key.debugName ?? ''
      : key.keyLabel.toUpperCase();
  if (label.isEmpty) return null;
  parts.add(label);
  return parts.join('+');
}
