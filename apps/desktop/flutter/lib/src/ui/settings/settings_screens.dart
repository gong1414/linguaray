import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../config/dependencies.dart';
import '../../i18n/i18n.dart';
import '../../routes/settings/provider_catalog.dart';
import '../../services/system_proxy.dart';
import '../../utils/external_url.dart';
import '../i18n_labels.dart';
import '../shared/status_message.dart';
import 'settings_intent_controller.dart';
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
    if (location == '/settings/translation') {
      return SettingsSection.translation;
    }
    if (location == '/settings/services/translation') {
      return SettingsSection.translationServices;
    }
    if (location == '/settings/favorites') {
      return SettingsSection.favorites;
    }
    if (location == '/settings/history') {
      return SettingsSection.history;
    }
    if (location == '/settings/glossary') return SettingsSection.glossary;
    if (location == '/settings/vocabulary') return SettingsSection.vocabulary;
    if (location == '/settings/ocr') {
      return SettingsSection.ocr;
    }
    if (location == '/settings/services/ocr') {
      return SettingsSection.ocrServices;
    }
    if (location.startsWith('/settings/permissions')) {
      return SettingsSection.permissions;
    }
    if (location.startsWith('/settings/integration')) {
      return SettingsSection.integration;
    }
    if (location.startsWith('/settings/data-transfer')) {
      return SettingsSection.dataTransfer;
    }
    if (location.startsWith('/settings/about')) {
      return SettingsSection.about;
    }
    if (location.startsWith('/settings/updates')) {
      return SettingsSection.updates;
    }
    return SettingsSection.general;
  }

  @override
  Widget build(BuildContext context) {
    return SettingsShellView(
      labels: settingsShellLabels(),
      section: _section,
      onSectionSelected: (section) => context.go(switch (section) {
        SettingsSection.translation => '/settings/translation',
        SettingsSection.translationServices => '/settings/services/translation',
        SettingsSection.favorites => '/settings/favorites',
        SettingsSection.history => '/settings/history',
        SettingsSection.glossary => '/settings/glossary',
        SettingsSection.vocabulary => '/settings/vocabulary',
        SettingsSection.ocr => '/settings/ocr',
        SettingsSection.ocrServices => '/settings/services/ocr',
        SettingsSection.general => '/settings/general',
        SettingsSection.permissions => '/settings/permissions',
        SettingsSection.integration => '/settings/integration',
        SettingsSection.dataTransfer => '/settings/data-transfer',
        SettingsSection.updates => '/settings/updates',
        SettingsSection.about => '/settings/about',
      }),
      child: child,
    );
  }
}

class GeneralSettingsScreen extends ConsumerStatefulWidget {
  const GeneralSettingsScreen({super.key});

  @override
  ConsumerState<GeneralSettingsScreen> createState() =>
      _GeneralSettingsScreenState();
}

class _GeneralSettingsScreenState extends ConsumerState<GeneralSettingsScreen> {
  bool _intentScheduled = false;

  @override
  void initState() {
    super.initState();
    generalSettingsIntentController.addListener(_scheduleIntent);
    _scheduleIntent();
  }

  @override
  void dispose() {
    generalSettingsIntentController.removeListener(_scheduleIntent);
    super.dispose();
  }

  void _scheduleIntent() {
    if (!mounted || _intentScheduled) return;
    _intentScheduled = true;
    WidgetsBinding.instance.addPostFrameCallback((_) async {
      _intentScheduled = false;
      if (!mounted) return;
      final state = ref.read(generalSettingsViewModelProvider);
      if (state.preferences == null || state.translationLanguages.isEmpty) {
        return;
      }
      final intent = generalSettingsIntentController.takePending();
      if (intent == null) return;
      switch (intent) {
        case GeneralSettingsIntent.manageCommonLanguages:
          await _manageCommonLanguages(state);
        case GeneralSettingsIntent.manageTranslationTargets:
          await _manageTranslationTargets(state);
        case GeneralSettingsIntent.addTranslationTarget:
          await _addTranslationTarget(state);
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(generalSettingsViewModelProvider);
    final preferences = state.preferences;
    if (preferences == null || state.loading) {
      return const Center(child: CircularProgressIndicator());
    }
    if (generalSettingsIntentController.hasPending) _scheduleIntent();
    return GeneralSettingsView(
      labels: generalSettingsLabels(),
      pageTitle: t.settings.navigation.general_settings,
      preferences: preferences,
      languages: state.languages,
      translationLanguages: state.translationLanguages,
      errorCode: state.errorCode,
      onRetry: () => unawaited(
        ref.read(generalSettingsViewModelProvider.notifier).reload(),
      ),
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
      onCommonLanguagesChanged: (value) => unawaited(
        ref
            .read(generalSettingsViewModelProvider.notifier)
            .setCommonLanguages(value),
      ),
      onInputSubmitModeChanged: (value) => unawaited(
        ref
            .read(generalSettingsViewModelProvider.notifier)
            .setInputSubmitMode(value),
      ),
      onAutoCopyChanged: (value) => unawaited(
        ref
            .read(generalSettingsViewModelProvider.notifier)
            .setAutoCopyDetectedText(value),
      ),
      onDoubleClickCopyChanged: (value) => unawaited(
        ref
            .read(generalSettingsViewModelProvider.notifier)
            .setDoubleClickCopyResult(value),
      ),
      onManageTranslationTargets: () => unawaited(
        _manageTranslationTargets(ref.read(generalSettingsViewModelProvider)),
      ),
      showTranslationSections: false,
    );
  }

  Future<void> _manageCommonLanguages(GeneralSettingsViewState state) async {
    final preferences = state.preferences;
    if (preferences == null) return;
    final selected = {...preferences.commonLanguages};
    final result = await showDialog<List<String>>(
      context: context,
      builder: (dialogContext) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: Text(t.settings.general.button.manage_languages),
          content: SizedBox(
            width: 420,
            height: 400,
            child: ListView(
              children: [
                for (final language in state.translationLanguages)
                  CheckboxListTile(
                    contentPadding: EdgeInsets.zero,
                    title: Text(language.name),
                    value: selected.contains(language.code),
                    onChanged: (checked) => setDialogState(() {
                      if (checked ?? false) {
                        selected.add(language.code);
                      } else {
                        selected.remove(language.code);
                      }
                    }),
                  ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(dialogContext),
              child: Text(t.common.ui.button.cancel),
            ),
            FilledButton(
              onPressed: () => Navigator.pop(dialogContext, selected.toList()),
              child: Text(t.common.ui.button.save),
            ),
          ],
        ),
      ),
    );
    if (result == null || !mounted) return;
    await ref
        .read(generalSettingsViewModelProvider.notifier)
        .setCommonLanguages(result);
  }

  Future<void> _manageTranslationTargets(GeneralSettingsViewState state) async {
    final preferences = state.preferences;
    if (preferences == null) return;
    final targets = [...preferences.translationTargets];
    final result = await showDialog<List<TranslationTargetRule>>(
      context: context,
      builder: (dialogContext) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: Text(t.settings.general.button.manage_targets),
          content: SizedBox(
            width: 520,
            height: 360,
            child: targets.isEmpty
                ? Center(
                    child: Text(t.settings.general.row.no_translation_targets),
                  )
                : ListView.builder(
                    itemCount: targets.length,
                    itemBuilder: (context, index) {
                      final target = targets[index];
                      return ListTile(
                        key: ValueKey(
                          'translation-target-${target.source}-${target.target}',
                        ),
                        contentPadding: EdgeInsets.zero,
                        title: Text(
                          '${_languageName(state, target.source, source: true)}'
                          ' → ${_languageName(state, target.target)}',
                        ),
                        leading: Switch(
                          value: target.enabled,
                          onChanged: (enabled) => setDialogState(() {
                            targets[index] = TranslationTargetRule(
                              source: target.source,
                              target: target.target,
                              enabled: enabled,
                            );
                          }),
                        ),
                        trailing: Wrap(
                          children: [
                            IconButton(
                              tooltip: t.common.ui.button.edit,
                              icon: const Icon(Icons.edit_outlined),
                              onPressed: () async {
                                final edited = await _showTargetEditor(
                                  state,
                                  targets: targets,
                                  editingIndex: index,
                                );
                                if (edited != null && context.mounted) {
                                  setDialogState(() => targets[index] = edited);
                                }
                              },
                            ),
                            IconButton(
                              tooltip: t.common.ui.button.delete,
                              icon: const Icon(Icons.delete_outline),
                              onPressed: () =>
                                  setDialogState(() => targets.removeAt(index)),
                            ),
                          ],
                        ),
                      );
                    },
                  ),
          ),
          actions: [
            TextButton.icon(
              key: const ValueKey('add-translation-target'),
              onPressed: () async {
                final added = await _showTargetEditor(state, targets: targets);
                if (added != null && context.mounted) {
                  setDialogState(() => targets.add(added));
                }
              },
              icon: const Icon(Icons.add),
              label: Text(t.settings.general.button.add_target),
            ),
            TextButton(
              onPressed: () => Navigator.pop(dialogContext),
              child: Text(t.common.ui.button.cancel),
            ),
            FilledButton(
              onPressed: () => Navigator.pop(dialogContext, targets),
              child: Text(t.common.ui.button.save),
            ),
          ],
        ),
      ),
    );
    if (result == null || !mounted) return;
    await ref
        .read(generalSettingsViewModelProvider.notifier)
        .setTranslationTargets(result);
  }

  Future<void> _addTranslationTarget(GeneralSettingsViewState state) async {
    final preferences = state.preferences;
    if (preferences == null) return;
    final targets = [...preferences.translationTargets];
    final added = await _showTargetEditor(state, targets: targets);
    if (added == null || !mounted) return;
    await ref
        .read(generalSettingsViewModelProvider.notifier)
        .setTranslationTargets([...targets, added]);
  }

  Future<TranslationTargetRule?> _showTargetEditor(
    GeneralSettingsViewState state, {
    required List<TranslationTargetRule> targets,
    int? editingIndex,
  }) async {
    if (state.translationLanguages.isEmpty) return null;
    final existing = editingIndex == null ? null : targets[editingIndex];
    var source = existing?.source ?? 'auto';
    var target = existing?.target ?? _defaultTarget(state.translationLanguages);
    var enabled = existing?.enabled ?? true;

    return showDialog<TranslationTargetRule>(
      context: context,
      builder: (dialogContext) => StatefulBuilder(
        builder: (context, setDialogState) {
          final duplicate = targets.indexed.any(
            (entry) =>
                entry.$1 != editingIndex &&
                entry.$2.source == source &&
                entry.$2.target == target,
          );
          final sameLanguage = source != 'auto' && source == target;
          final canSave = !duplicate && !sameLanguage;
          return AlertDialog(
            title: Text(
              existing == null
                  ? t.settings.general.editor.add_target_title
                  : t.settings.general.editor.edit_target_title,
            ),
            content: SizedBox(
              width: 440,
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  DropdownButtonFormField<String>(
                    key: ValueKey('source-$source'),
                    initialValue: source,
                    decoration: InputDecoration(
                      labelText: t.settings.general.editor.row.source_language,
                    ),
                    items: [
                      DropdownMenuItem(
                        value: 'auto',
                        child: Text(t.mini_translator.language.auto_detect),
                      ),
                      for (final language in state.translationLanguages)
                        DropdownMenuItem(
                          value: language.code,
                          child: Text(language.name),
                        ),
                    ],
                    onChanged: (value) {
                      if (value != null) {
                        setDialogState(() => source = value);
                      }
                    },
                  ),
                  const SizedBox(height: 14),
                  DropdownButtonFormField<String>(
                    key: ValueKey('target-$target'),
                    initialValue: target,
                    decoration: InputDecoration(
                      labelText: t.settings.general.editor.row.target_language,
                    ),
                    items: [
                      for (final language in state.translationLanguages)
                        DropdownMenuItem(
                          value: language.code,
                          child: Text(language.name),
                        ),
                    ],
                    onChanged: (value) {
                      if (value != null) {
                        setDialogState(() => target = value);
                      }
                    },
                  ),
                  SwitchListTile(
                    contentPadding: EdgeInsets.zero,
                    title: Text(t.settings.advanced.enable),
                    value: enabled,
                    onChanged: (value) => setDialogState(() => enabled = value),
                  ),
                  if (sameLanguage || duplicate)
                    Align(
                      alignment: Alignment.centerLeft,
                      child: Text(
                        sameLanguage
                            ? t.settings.general.editor.same_language
                            : t.settings.general.editor.duplicate,
                        style: TextStyle(
                          color: Theme.of(context).colorScheme.error,
                        ),
                      ),
                    ),
                ],
              ),
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(dialogContext),
                child: Text(t.common.ui.button.cancel),
              ),
              FilledButton(
                onPressed: canSave
                    ? () => Navigator.pop(
                        dialogContext,
                        TranslationTargetRule(
                          source: source,
                          target: target,
                          enabled: enabled,
                        ),
                      )
                    : null,
                child: Text(t.common.ui.button.save),
              ),
            ],
          );
        },
      ),
    );
  }

  String _defaultTarget(List<LanguageChoice> languages) {
    for (final preferred in const ['zh-Hans', 'en']) {
      if (languages.any((language) => language.code == preferred)) {
        return preferred;
      }
    }
    return languages.first.code;
  }

  String _languageName(
    GeneralSettingsViewState state,
    String code, {
    bool source = false,
  }) {
    if (source && code == 'auto') {
      return t.mini_translator.language.auto_detect;
    }
    for (final language in state.translationLanguages) {
      if (language.code == code) return language.name;
    }
    return code;
  }
}

class ServicesSettingsScreen extends ConsumerWidget {
  const ServicesSettingsScreen({required this.serviceKind, super.key});

  final String serviceKind;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(servicesSettingsViewModelProvider);
    return ServicesSettingsView(
      labels: servicesSettingsLabels(),
      pageTitle: serviceKind == 'ocr'
          ? t.settings.navigation.ocr_services
          : t.settings.navigation.translation_services,
      services: state.services,
      serviceKind: serviceKind,
      loading: state.loading,
      onEnabledChanged: (id, enabled) => unawaited(
        ref
            .read(servicesSettingsViewModelProvider.notifier)
            .setEnabled(id, enabled),
      ),
      onMakeDefault: (id) => unawaited(
        ref.read(servicesSettingsViewModelProvider.notifier).makeDefault(id),
      ),
      onDelete: (id) => unawaited(_confirmDeleteService(context, ref, id)),
      onReorderTranslation: (oldIndex, newIndex) => unawaited(
        ref
            .read(servicesSettingsViewModelProvider.notifier)
            .reorderTranslation(oldIndex, newIndex),
      ),
      onConfigureProviders: () => unawaited(_showProviderManager(context)),
      onAdd: () => unawaited(_addService(context, ref, serviceKind)),
      errorCode: state.operationErrorCode,
      onRetry: () => unawaited(
        ref.read(servicesSettingsViewModelProvider.notifier).reload(),
      ),
    );
  }
}

Future<void> _confirmDeleteService(
  BuildContext context,
  WidgetRef ref,
  String id,
) async {
  final labels = servicesSettingsLabels();
  final confirmed = await showDialog<bool>(
    context: context,
    builder: (dialogContext) => AlertDialog(
      title: Text(labels.delete),
      content: Text(labels.deleteConfirm),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(dialogContext, false),
          child: Text(t.common.ui.button.cancel),
        ),
        FilledButton(
          onPressed: () => Navigator.pop(dialogContext, true),
          child: Text(labels.delete),
        ),
      ],
    ),
  );
  if (confirmed == true) {
    await ref
        .read(servicesSettingsViewModelProvider.notifier)
        .deleteService(id);
  }
}

Future<void> _addService(
  BuildContext context,
  WidgetRef ref,
  String serviceKind,
) async {
  var providers = await ref
      .read(workspaceSettingsRepositoryProvider)
      .listProviders();
  if (providers.isEmpty && context.mounted) {
    await _showProviderManager(context);
    providers = await ref
        .read(workspaceSettingsRepositoryProvider)
        .listProviders();
  }
  if (providers.isEmpty || !context.mounted) return;
  var providerId = providers.first.id;
  final name = TextEditingController();
  final saved = await showDialog<bool>(
    context: context,
    builder: (context) {
      return StatefulBuilder(
        builder: (context, setState) => AlertDialog(
          title: Text(t.settings.services.button.add_service),
          content: SizedBox(
            width: 360,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                DropdownButton<String>(
                  value: providerId,
                  isExpanded: true,
                  items: [
                    for (final provider in providers)
                      DropdownMenuItem(
                        value: provider.id,
                        child: Text(provider.displayName),
                      ),
                  ],
                  onChanged: (value) {
                    if (value != null) setState(() => providerId = value);
                  },
                ),
                TextField(
                  controller: name,
                  decoration: const InputDecoration(),
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: Text(t.common.ui.button.cancel),
            ),
            FilledButton(
              onPressed: () => Navigator.pop(context, true),
              child: Text(t.common.ui.button.save),
            ),
          ],
        ),
      );
    },
  );
  if (saved != true) return;
  await ref
      .read(servicesSettingsViewModelProvider.notifier)
      .addService(
        ServiceDraft(
          providerId: providerId,
          kind: serviceKind,
          name: name.text.trim().isEmpty
              ? '$providerId $serviceKind'
              : name.text.trim(),
        ),
      );
}

Future<void> _showProviderManager(BuildContext context) {
  return showDialog<void>(
    context: context,
    builder: (dialogContext) => AlertDialog(
      contentPadding: EdgeInsets.zero,
      content: const SizedBox(
        width: 720,
        height: 520,
        child: ProvidersSettingsScreen(),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(dialogContext),
          child: Text(t.ui.shell.close),
        ),
      ],
    ),
  );
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
  late String _presetId = '';
  final Map<String, String> _fields = {};
  Set<String> _storedSecrets = {};
  List<String> _models = const [];
  bool _loadingModels = false;
  String? _modelsError;

  @override
  void initState() {
    super.initState();
    final state = ref.read(providersSettingsViewModelProvider);
    if (widget.providerId != null) {
      final provider = state.providers
          .where((item) => item.id == widget.providerId)
          .firstOrNull;
      if (provider != null) {
        final selected = findProviderCatalogOption(
          state.types,
          presetId: provider.presetId,
          engineTypeId: provider.typeId,
        );
        _presetId = selected?.id ?? provider.typeId;
        if (selected != null) {
          _fields.addAll(providerPresetInitialFields(selected));
        }
        _fields.addAll(provider.publicFields);
        _storedSecrets = provider.storedSecretKeys;
      }
    }
  }

  ProviderTypeOption? get _selected => findProviderCatalogOption(
    ref.read(providersSettingsViewModelProvider).types,
    presetId: _presetId,
  );

  ProviderDraft get _draft {
    final selected = _selected;
    return ProviderDraft(
      id: _id.trim(),
      typeId: selected?.engineTypeId ?? selected?.id ?? '',
      presetId: selected?.id,
      fields: Map.of(_fields),
    );
  }

  String _suggestId(String presetId, List<ProviderRecord> providers) {
    final used = {for (final provider in providers) provider.id};
    if (!used.contains(presetId)) return presetId;
    for (var suffix = 2; ; suffix++) {
      final candidate = '$presetId-$suffix';
      if (!used.contains(candidate)) return candidate;
    }
  }

  Future<void> _fetchModels() async {
    setState(() {
      _loadingModels = true;
      _modelsError = null;
    });
    try {
      final models = await ref
          .read(workspaceSettingsRepositoryProvider)
          .discoverProviderModels(_draft);
      if (!mounted) return;
      setState(() {
        _models = models;
        _modelsError = models.isEmpty ? t.settings.providers.model_empty : null;
      });
    } catch (_) {
      if (!mounted) return;
      setState(() => _modelsError = t.settings.providers.model_failed);
    } finally {
      if (mounted) setState(() => _loadingModels = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(providersSettingsViewModelProvider);
    return ProviderEditorView(
      labels: providersSettingsLabels(),
      types: state.types,
      draftId: _id,
      typeId: _presetId,
      fields: _fields,
      storedSecretKeys: _storedSecrets,
      testing: state.testing,
      testResult: state.testResult,
      saving: state.saving,
      models: _models,
      loadingModels: _loadingModels,
      modelsError: _modelsError,
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
      onTypeChanged: (value) {
        final selected = findProviderCatalogOption(
          state.types,
          presetId: value,
        );
        if (selected == null) return;
        setState(() {
          ref.read(providersSettingsViewModelProvider.notifier).clearFeedback();
          _presetId = selected.id;
          _fields
            ..clear()
            ..addAll(providerPresetInitialFields(selected));
          if (widget.providerId == null) {
            _id = _suggestId(selected.id, state.providers);
          }
          _storedSecrets = {};
        });
      },
      onFieldChanged: (key, value) {
        ref.read(providersSettingsViewModelProvider.notifier).clearFeedback();
        setState(() => _fields[key] = value);
      },
      onFetchModels: () => unawaited(_fetchModels()),
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
    final preferences = state.preferences;
    if (preferences == null || state.translationLanguages.isEmpty) return;
    final targets = [...preferences.translationTargets];
    final result = await showDialog<List<TranslationTargetRule>>(
      context: context,
      builder: (dialogContext) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: Text(t.settings.general.button.manage_targets),
          content: SizedBox(
            width: 500,
            height: 340,
            child: targets.isEmpty
                ? Center(
                    child: Text(t.settings.general.row.no_translation_targets),
                  )
                : ListView.builder(
                    itemCount: targets.length,
                    itemBuilder: (context, index) {
                      final target = targets[index];
                      return SwitchListTile(
                        contentPadding: EdgeInsets.zero,
                        title: Text('${target.source} → ${target.target}'),
                        value: target.enabled,
                        onChanged: (value) => setDialogState(() {
                          targets[index] = TranslationTargetRule(
                            source: target.source,
                            target: target.target,
                            enabled: value,
                          );
                        }),
                        secondary: IconButton(
                          icon: const Icon(Icons.delete_outline_rounded),
                          onPressed: () =>
                              setDialogState(() => targets.removeAt(index)),
                        ),
                      );
                    },
                  ),
          ),
          actions: [
            TextButton.icon(
              onPressed: () async {
                final added = await _addTranslationTargetDialog(
                  context,
                  state.translationLanguages,
                );
                if (added != null && context.mounted) {
                  setDialogState(() => targets.add(added));
                }
              },
              icon: const Icon(Icons.add_rounded),
              label: Text(t.common.ui.button.add),
            ),
            TextButton(
              onPressed: () => Navigator.pop(dialogContext),
              child: Text(t.common.ui.button.cancel),
            ),
            FilledButton(
              onPressed: () => Navigator.pop(dialogContext, targets),
              child: Text(t.common.ui.button.save),
            ),
          ],
        ),
      ),
    );
    if (result == null || !mounted) return;
    await ref
        .read(generalSettingsViewModelProvider.notifier)
        .setTranslationTargets(result);
  }

  Future<void> _editCommonLanguages(
    BuildContext context,
    GeneralSettingsViewState state,
  ) async {
    final preferences = state.preferences;
    if (preferences == null) return;
    final selected = {...preferences.commonLanguages};
    final result = await showDialog<List<String>>(
      context: context,
      builder: (dialogContext) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: Text(t.settings.general.button.manage_languages),
          content: SizedBox(
            width: 440,
            height: 430,
            child: ListView(
              children: [
                for (final language in state.translationLanguages)
                  CheckboxListTile(
                    contentPadding: EdgeInsets.zero,
                    title: Text(language.name),
                    value: selected.contains(language.code),
                    onChanged: (value) => setDialogState(() {
                      value ?? false
                          ? selected.add(language.code)
                          : selected.remove(language.code);
                    }),
                  ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(dialogContext),
              child: Text(t.common.ui.button.cancel),
            ),
            FilledButton(
              onPressed: () => Navigator.pop(dialogContext, selected.toList()),
              child: Text(t.common.ui.button.save),
            ),
          ],
        ),
      ),
    );
    if (result == null || !mounted) return;
    await ref
        .read(generalSettingsViewModelProvider.notifier)
        .setCommonLanguages(result);
  }

  Future<TranslationTargetRule?> _addTranslationTargetDialog(
    BuildContext context,
    List<LanguageChoice> languages,
  ) {
    var source = 'auto';
    var target = languages.first.code;
    return showDialog<TranslationTargetRule>(
      context: context,
      builder: (dialogContext) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: Text(t.settings.general.button.add_target),
          content: SizedBox(
            width: 420,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                DropdownButtonFormField<String>(
                  initialValue: source,
                  decoration: InputDecoration(
                    labelText: t.settings.general.editor.row.source_language,
                  ),
                  items: [
                    DropdownMenuItem(
                      value: 'auto',
                      child: Text(t.mini_translator.language.auto_detect),
                    ),
                    for (final language in languages)
                      DropdownMenuItem(
                        value: language.code,
                        child: Text(language.name),
                      ),
                  ],
                  onChanged: (value) =>
                      setDialogState(() => source = value ?? source),
                ),
                const SizedBox(height: 12),
                DropdownButtonFormField<String>(
                  initialValue: target,
                  decoration: InputDecoration(
                    labelText: t.settings.general.editor.row.target_language,
                  ),
                  items: [
                    for (final language in languages)
                      DropdownMenuItem(
                        value: language.code,
                        child: Text(language.name),
                      ),
                  ],
                  onChanged: (value) =>
                      setDialogState(() => target = value ?? target),
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(dialogContext),
              child: Text(t.common.ui.button.cancel),
            ),
            FilledButton(
              onPressed: source == target
                  ? null
                  : () => Navigator.pop(
                      dialogContext,
                      TranslationTargetRule(
                        source: source,
                        target: target,
                        enabled: true,
                      ),
                    ),
              child: Text(t.common.ui.button.add),
            ),
          ],
        ),
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

class AdvancedSettingsScreen extends ConsumerStatefulWidget {
  const AdvancedSettingsScreen({super.key});

  @override
  ConsumerState<AdvancedSettingsScreen> createState() =>
      _AdvancedSettingsScreenState();
}

class _AdvancedSettingsScreenState
    extends ConsumerState<AdvancedSettingsScreen> {
  ApiServerStatus? _status;
  NetworkSettings? _network;
  String? _apiError;
  String? _networkError;
  final TextEditingController _port = TextEditingController();
  final TextEditingController _proxyUrl = TextEditingController();
  final TextEditingController _proxyBypass = TextEditingController();

  @override
  void initState() {
    super.initState();
    unawaited(_reload());
  }

  @override
  void dispose() {
    _port.dispose();
    _proxyUrl.dispose();
    _proxyBypass.dispose();
    super.dispose();
  }

  Future<void> _reload() async {
    final repository = ref.read(workspaceSettingsRepositoryProvider);
    try {
      final status = await repository.loadApiServer();
      if (!mounted) return;
      setState(() {
        _status = status;
        _apiError = status.bindErrorCode;
        _port.text = '${status.port}';
      });
    } catch (_) {
      if (!mounted) return;
      setState(() => _apiError = AppErrorCode.apiServerBindFailed.wireName);
    }
    try {
      final network = await repository.loadNetworkSettings();
      if (!mounted) return;
      setState(() {
        _network = network;
        _networkError = null;
        _proxyUrl.text = network.proxyUrl;
        _proxyBypass.text = network.proxyBypass;
      });
    } catch (_) {
      if (!mounted) return;
      setState(() => _networkError = AppErrorCode.networkFailure.wireName);
    }
  }

  @override
  Widget build(BuildContext context) {
    final status = _status;
    final advanced = t.settings.advanced;
    return ListView(
      padding: const EdgeInsets.fromLTRB(24, 16, 24, 32),
      children: [
        Text(advanced.title, style: Theme.of(context).textTheme.headlineMedium),
        const SizedBox(height: 8),
        Text(advanced.api_server_description),
        if (status != null)
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(advanced.enable),
            value: status.enabled,
            onChanged: (value) async {
              try {
                final next = await ref
                    .read(workspaceSettingsRepositoryProvider)
                    .setApiServerEnabled(value);
                if (!mounted) return;
                setState(() {
                  _status = next;
                  _apiError = next.bindErrorCode;
                });
              } catch (_) {
                if (!mounted) return;
                setState(
                  () => _apiError = AppErrorCode.apiServerBindFailed.wireName,
                );
              }
            },
          ),
        ListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(advanced.port),
          trailing: SizedBox(
            width: 96,
            child: TextField(
              controller: _port,
              keyboardType: TextInputType.number,
              onSubmitted: (value) async {
                final port = int.tryParse(value);
                if (port == null) {
                  setState(() => _apiError = AppErrorCode.invalidPort.wireName);
                  return;
                }
                final next = await ref
                    .read(workspaceSettingsRepositoryProvider)
                    .setApiServerPort(port);
                if (!mounted) return;
                setState(() {
                  _status = next;
                  _apiError = next.bindErrorCode;
                });
              },
            ),
          ),
        ),
        if (status?.baseUrl != null) ...[
          ListTile(
            contentPadding: EdgeInsets.zero,
            title: Text('${advanced.running_at} ${status!.baseUrl}'),
            trailing: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                IconButton(
                  tooltip: t.common.ui.feedback.copied,
                  onPressed: () =>
                      Clipboard.setData(ClipboardData(text: status.baseUrl!)),
                  icon: const Icon(Icons.copy_rounded),
                ),
                IconButton(
                  onPressed: () => unawaited(openExternalUrl(status.baseUrl!)),
                  icon: const Icon(Icons.open_in_new_rounded),
                ),
              ],
            ),
          ),
        ],
        if (_apiError != null)
          StatusMessage(
            kind: StatusKind.error,
            title: appErrorMessage(_apiError),
          ),
        const SizedBox(height: 24),
        const Divider(),
        const SizedBox(height: 16),
        Text(advanced.network, style: Theme.of(context).textTheme.titleMedium),
        if (_network != null) ...[
          const SizedBox(height: 8),
          DropdownButtonFormField<NetworkProxyMode>(
            initialValue: _network!.proxyMode,
            decoration: InputDecoration(labelText: advanced.proxy_mode),
            items: [
              DropdownMenuItem(
                value: NetworkProxyMode.system,
                child: Text(advanced.proxy_system),
              ),
              DropdownMenuItem(
                value: NetworkProxyMode.direct,
                child: Text(advanced.proxy_direct),
              ),
              DropdownMenuItem(
                value: NetworkProxyMode.custom,
                child: Text(advanced.proxy_custom),
              ),
            ],
            onChanged: (mode) {
              if (mode == null) return;
              setState(
                () => _network = NetworkSettings(
                  proxyMode: mode,
                  proxyUrl: _proxyUrl.text,
                  proxyBypass: _proxyBypass.text,
                  checkUpdatesOnLaunch: _network!.checkUpdatesOnLaunch,
                ),
              );
            },
          ),
          if (_network!.proxyMode == NetworkProxyMode.custom) ...[
            const SizedBox(height: 12),
            TextField(
              controller: _proxyUrl,
              decoration: InputDecoration(
                labelText: advanced.proxy_url,
                hintText: advanced.proxy_url_hint,
              ),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: _proxyBypass,
              decoration: InputDecoration(
                labelText: advanced.proxy_bypass,
                hintText: advanced.proxy_bypass_hint,
              ),
            ),
          ],
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(advanced.check_updates_on_launch),
            value: _network!.checkUpdatesOnLaunch,
            onChanged: (value) => setState(
              () => _network = NetworkSettings(
                proxyMode: _network!.proxyMode,
                proxyUrl: _proxyUrl.text,
                proxyBypass: _proxyBypass.text,
                checkUpdatesOnLaunch: value,
              ),
            ),
          ),
          Align(
            alignment: Alignment.centerRight,
            child: FilledButton(
              onPressed: _saveNetwork,
              child: Text(advanced.save_network),
            ),
          ),
          if (_networkError != null) ...[
            const SizedBox(height: 12),
            StatusMessage(
              kind: StatusKind.error,
              title: appErrorMessage(_networkError),
            ),
          ],
        ],
      ],
    );
  }

  Future<void> _saveNetwork() async {
    final network = _network;
    if (network == null) return;
    if (network.proxyMode == NetworkProxyMode.custom) {
      final uri = Uri.tryParse(_proxyUrl.text.trim());
      if (uri == null ||
          (uri.scheme != 'http' && uri.scheme != 'https') ||
          uri.host.isEmpty ||
          uri.userInfo.isNotEmpty) {
        setState(
          () => _networkError = AppErrorCode.proxyConfigurationInvalid.wireName,
        );
        return;
      }
    }
    try {
      final saved = await ref
          .read(workspaceSettingsRepositoryProvider)
          .saveNetworkSettings(
            NetworkSettings(
              proxyMode: network.proxyMode,
              proxyUrl: _proxyUrl.text,
              proxyBypass: _proxyBypass.text,
              checkUpdatesOnLaunch: network.checkUpdatesOnLaunch,
            ),
          );
      await initializeSystemProxy();
      if (!mounted) return;
      setState(() {
        _network = saved;
        _networkError = null;
      });
    } catch (_) {
      if (!mounted) return;
      setState(() => _networkError = AppErrorCode.networkFailure.wireName);
    }
  }
}

bool get isDesktopWindows => Platform.isWindows;
