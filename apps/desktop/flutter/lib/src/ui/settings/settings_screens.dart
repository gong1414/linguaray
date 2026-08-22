import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../routes/settings/about.dart' show openExternalUrl;
import '../i18n_labels.dart';
import 'settings_labels.dart';
import 'settings_shell_view.dart';
import 'view_models/permissions_view_model.dart';
import 'view_models/settings_view_model.dart';
import 'view_models/shortcuts_view_model.dart';
import 'views/about_settings_view.dart';
import 'views/general_settings_view.dart';
import 'views/permissions_settings_view.dart';
import 'views/providers_settings_view.dart';
import 'views/services_settings_view.dart';
import 'views/shortcuts_settings_view.dart';

class SettingsHostScreen extends StatelessWidget {
  const SettingsHostScreen({
    required this.location,
    required this.child,
    super.key,
  });

  final String location;
  final Widget child;

  SettingsSection get _section {
    if (location.startsWith('/settings/services')) {
      return SettingsSection.services;
    }
    if (location.startsWith('/settings/providers')) {
      return SettingsSection.providers;
    }
    if (location.startsWith('/settings/shortcuts')) {
      return SettingsSection.shortcuts;
    }
    if (location.startsWith('/settings/permissions')) {
      return SettingsSection.permissions;
    }
    if (location.startsWith('/settings/about')) {
      return SettingsSection.about;
    }
    return SettingsSection.general;
  }

  @override
  Widget build(BuildContext context) {
    return SettingsShellView(
      labels: settingsShellLabels(),
      section: _section,
      onSectionSelected: (section) => context.go(switch (section) {
        SettingsSection.general => '/settings/general',
        SettingsSection.services => '/settings/services',
        SettingsSection.providers => '/settings/providers',
        SettingsSection.shortcuts => '/settings/shortcuts',
        SettingsSection.permissions => '/settings/permissions',
        SettingsSection.about => '/settings/about',
      }),
      child: child,
    );
  }
}

class GeneralSettingsScreen extends ConsumerWidget {
  const GeneralSettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(generalSettingsViewModelProvider);
    final preferences = state.preferences;
    if (preferences == null || state.loading) {
      return const Center(child: CircularProgressIndicator());
    }
    return GeneralSettingsView(
      labels: generalSettingsLabels(),
      preferences: preferences,
      languages: state.languages,
      onLaunchAtLoginChanged: (value) => unawaited(
        ref
            .read(generalSettingsViewModelProvider.notifier)
            .setLaunchAtLogin(value),
      ),
      onShowInMenuBarChanged: (value) => unawaited(
        ref
            .read(generalSettingsViewModelProvider.notifier)
            .setShowInMenuBar(value),
      ),
      onLanguageChanged: (value) => unawaited(
        ref.read(generalSettingsViewModelProvider.notifier).setLanguage(value),
      ),
      onThemeModeChanged: (value) => unawaited(
        ref.read(generalSettingsViewModelProvider.notifier).setThemeMode(value),
      ),
    );
  }
}

class ServicesSettingsScreen extends ConsumerWidget {
  const ServicesSettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(servicesSettingsViewModelProvider);
    return ServicesSettingsView(
      labels: servicesSettingsLabels(),
      services: state.services,
      loading: state.loading,
      onEnabledChanged: (id, enabled) => unawaited(
        ref
            .read(servicesSettingsViewModelProvider.notifier)
            .setEnabled(id, enabled),
      ),
      onMakeDefault: (id) => unawaited(
        ref.read(servicesSettingsViewModelProvider.notifier).makeDefault(id),
      ),
      onConfigureProviders: () => context.go('/settings/providers'),
    );
  }
}

class ProvidersSettingsScreen extends ConsumerWidget {
  const ProvidersSettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(providersSettingsViewModelProvider);
    return ProvidersSettingsView(
      labels: providersSettingsLabels(),
      providers: state.providers,
      loading: state.loading,
      onAdd: () => unawaited(_openEditor(context, ref)),
      onEdit: (id) => unawaited(_openEditor(context, ref, providerId: id)),
      onDelete: (id) => unawaited(_confirmDelete(context, ref, id)),
    );
  }

  Future<void> _confirmDelete(
    BuildContext context,
    WidgetRef ref,
    String id,
  ) async {
    final labels = providersSettingsLabels();
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(labels.deleteConfirmTitle),
        content: Text(labels.deleteConfirmBody),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: Text(labels.cancel),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(labels.delete),
          ),
        ],
      ),
    );
    if (confirmed == true) {
      await ref.read(providersSettingsViewModelProvider.notifier).delete(id);
    }
  }

  Future<void> _openEditor(
    BuildContext context,
    WidgetRef ref, {
    String? providerId,
  }) async {
    ref.read(providersSettingsViewModelProvider.notifier).clearFeedback();
    await showDialog<void>(
      context: context,
      barrierDismissible: false,
      builder: (_) => _ProviderEditorDialog(providerId: providerId),
    );
  }
}

class _ProviderEditorDialog extends ConsumerStatefulWidget {
  const _ProviderEditorDialog({this.providerId});

  final String? providerId;

  @override
  ConsumerState<_ProviderEditorDialog> createState() =>
      _ProviderEditorDialogState();
}

class _ProviderEditorDialogState extends ConsumerState<_ProviderEditorDialog> {
  late String _id = widget.providerId ?? '';
  late String _typeId = 'openai';
  final Map<String, String> _fields = {};
  Set<String> _storedSecrets = {};

  @override
  void initState() {
    super.initState();
    final state = ref.read(providersSettingsViewModelProvider);
    if (widget.providerId != null) {
      final provider = state.providers
          .where((item) => item.id == widget.providerId)
          .firstOrNull;
      if (provider != null) {
        _typeId = provider.typeId;
        _fields.addAll(provider.publicFields);
        _storedSecrets = provider.storedSecretKeys;
      }
    } else if (state.types.isNotEmpty) {
      _typeId = state.types.first.id;
    }
  }

  ProviderDraft get _draft =>
      ProviderDraft(id: _id.trim(), typeId: _typeId, fields: Map.of(_fields));

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(providersSettingsViewModelProvider);
    return ProviderEditorView(
      labels: providersSettingsLabels(),
      types: state.types,
      draftId: _id,
      typeId: _typeId,
      fields: _fields,
      storedSecretKeys: _storedSecrets,
      testing: state.testing,
      testResult: state.testResult,
      saving: state.saving,
      operationError: switch (state.operationErrorCode) {
        'validation_missing' => providersSettingsLabels().validationMissing,
        'save_failed' => providersSettingsLabels().saveFailed,
        _ => null,
      },
      idReadOnly: widget.providerId != null,
      onIdChanged: (value) {
        ref.read(providersSettingsViewModelProvider.notifier).clearFeedback();
        setState(() => _id = value);
      },
      onTypeChanged: (value) => setState(() {
        ref.read(providersSettingsViewModelProvider.notifier).clearFeedback();
        _typeId = value;
        _fields.clear();
      }),
      onFieldChanged: (key, value) {
        ref.read(providersSettingsViewModelProvider.notifier).clearFeedback();
        setState(() => _fields[key] = value);
      },
      onTest: () => unawaited(
        ref.read(providersSettingsViewModelProvider.notifier).test(_draft),
      ),
      onSave: () async {
        final saved = await ref
            .read(providersSettingsViewModelProvider.notifier)
            .save(_draft);
        if (saved && context.mounted) Navigator.pop(context);
      },
      onCancel: () => Navigator.pop(context),
    );
  }
}

class ShortcutsSettingsScreen extends ConsumerStatefulWidget {
  const ShortcutsSettingsScreen({super.key});

  @override
  ConsumerState<ShortcutsSettingsScreen> createState() =>
      _ShortcutsSettingsScreenState();
}

class _ShortcutsSettingsScreenState
    extends ConsumerState<ShortcutsSettingsScreen> {
  final FocusNode _focusNode = FocusNode(debugLabel: 'shortcut-recorder');

  @override
  void dispose() {
    _focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(shortcutsViewModelProvider);
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
        shortcuts: [
          for (final item in state.shortcuts)
            ShortcutRecord(
              actionId: item.actionId,
              labelKey: shortcutActionLabel(item.actionId),
              accelerator: item.accelerator,
              status: item.status,
              conflictReason: item.conflictReason,
            ),
        ],
        recordingActionId: state.recordingActionId,
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

class PermissionsSettingsScreen extends ConsumerWidget {
  const PermissionsSettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return PermissionsSettingsView(
      labels: permissionsSettingsLabels(),
      snapshot: ref.watch(permissionsViewModelProvider),
      onGrantAccessibility: () => unawaited(
        ref.read(permissionsViewModelProvider.notifier).requestAccessibility(),
      ),
      onGrantScreenRecording: () => unawaited(
        ref
            .read(permissionsViewModelProvider.notifier)
            .requestScreenRecording(),
      ),
      onRecheck: () =>
          unawaited(ref.read(permissionsViewModelProvider.notifier).refresh()),
    );
  }
}

class AboutSettingsScreen extends ConsumerStatefulWidget {
  const AboutSettingsScreen({super.key});

  @override
  ConsumerState<AboutSettingsScreen> createState() =>
      _AboutSettingsScreenState();
}

class _AboutSettingsScreenState extends ConsumerState<AboutSettingsScreen> {
  bool _copied = false;

  @override
  Widget build(BuildContext context) {
    final info = ref.watch(aboutViewModelProvider);
    if (info == null) {
      return const Center(child: CircularProgressIndicator());
    }
    return AboutSettingsView(
      labels: aboutSettingsLabels(),
      info: info,
      copied: _copied,
      onCopyVersion: () async {
        await Clipboard.setData(
          ClipboardData(text: 'v${info.version} (${info.buildNumber})'),
        );
        setState(() => _copied = true);
      },
      onOpenWebsite: () =>
          unawaited(openExternalUrl('https://github.com/gong1414/linguaray')),
      onOpenChangelog: () => unawaited(
        openExternalUrl('https://github.com/gong1414/linguaray/releases'),
      ),
      onOpenIssues: () => unawaited(
        openExternalUrl('https://github.com/gong1414/linguaray/issues'),
      ),
      onOpenLicense: () => unawaited(
        openExternalUrl(
          'https://github.com/gong1414/linguaray/blob/main/LICENSE',
        ),
      ),
    );
  }
}

bool get isDesktopWindows => Platform.isWindows;
