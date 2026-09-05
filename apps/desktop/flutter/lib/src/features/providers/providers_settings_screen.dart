import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../app/dependencies.dart';
import '../../i18n/i18n.dart';
import '../../shared/i18n_labels.dart';
import 'provider_model_discovery_controller.dart';
import 'providers_settings_view.dart';
import 'providers_settings_view_model.dart';

Future<void> showProviderManager(BuildContext context) {
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
  late final ProviderModelDiscoveryController _discovery =
      ProviderModelDiscoveryController(
        (draft) => ref
            .read(providerSettingsRepositoryProvider)
            .discoverProviderModels(draft),
      );

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
    _discovery.addListener(_modelsChanged);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _scheduleModels(immediately: true);
    });
  }

  @override
  void dispose() {
    _discovery.dispose();
    super.dispose();
  }

  void _modelsChanged() {
    if (mounted) setState(() {});
  }

  void _scheduleModels({bool immediately = false}) {
    final type = _selected;
    final valid =
        type?.isLlm == true &&
        validateProviderDraft(
          draft: _draft,
          type: type,
          storedSecretKeys: _storedSecrets,
          ignoredRequiredFields: const {'defaultModel'},
        ).isValid;
    _discovery.schedule(valid ? _draft : null, immediately: immediately);
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
      discovery: _discovery.result,
      loadingModels: _discovery.loading,
      operationError: switch (state.operationErrorCode) {
        'validation_missing' => providersSettingsLabels().validationMissing,
        'save_failed' => providersSettingsLabels().saveFailed,
        _ => null,
      },
      idReadOnly: widget.providerId != null,
      onIdChanged: (value) {
        ref.read(providersSettingsViewModelProvider.notifier).clearFeedback();
        setState(() => _id = value);
        _scheduleModels();
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
        _scheduleModels();
      },
      onFieldChanged: (key, value) {
        ref.read(providersSettingsViewModelProvider.notifier).clearFeedback();
        setState(() {
          _fields[key] = value;
          // Older saved drafts may contain the preset's derived models URL.
          // When the API root changes, let discovery follow that root again.
          if (key == 'baseUrl' &&
              _fields['modelsUrl'] == _selected?.modelsUrl) {
            _fields.remove('modelsUrl');
          }
        });
        if (key != 'defaultModel') _scheduleModels();
      },
      onFetchModels: () => _scheduleModels(immediately: true),
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
