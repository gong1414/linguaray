import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../i18n/i18n.dart';
import '../../../routes/settings/provider_catalog.dart';
import '../../shared/settings_page.dart';
import '../../shared/status_message.dart';
import '../settings_labels.dart';

class ProvidersSettingsView extends StatelessWidget {
  const ProvidersSettingsView({
    required this.labels,
    required this.providers,
    required this.loading,
    required this.onAdd,
    required this.onEdit,
    required this.onDelete,
    super.key,
  });

  final ProvidersSettingsLabels labels;
  final List<ProviderRecord> providers;
  final bool loading;
  final VoidCallback onAdd;
  final ValueChanged<String> onEdit;
  final ValueChanged<String> onDelete;

  @override
  Widget build(BuildContext context) {
    return SettingsPage(
      title: labels.title,
      actions: [
        FilledButton.icon(
          onPressed: onAdd,
          icon: const Icon(Icons.add_rounded),
          label: Text(labels.add),
        ),
      ],
      body: loading
          ? const Center(child: CircularProgressIndicator())
          : providers.isEmpty
          ? Padding(
              padding: EdgeInsets.zero,
              child: StatusMessage(
                kind: StatusKind.info,
                title: labels.empty,
                action: OutlinedButton(
                  onPressed: onAdd,
                  child: Text(labels.add),
                ),
              ),
            )
          : ListView.separated(
              padding: EdgeInsets.zero,
              itemCount: providers.length,
              separatorBuilder: (_, _) => const Padding(
                padding: EdgeInsets.only(left: 60),
                child: Divider(),
              ),
              itemBuilder: (context, index) {
                final provider = providers[index];
                final model = provider.publicFields['defaultModel'];
                final colors = Theme.of(context).colorScheme;
                return ListTile(
                  contentPadding: const EdgeInsets.symmetric(
                    horizontal: 8,
                    vertical: 8,
                  ),
                  leading: Container(
                    width: 36,
                    height: 36,
                    decoration: BoxDecoration(
                      color: colors.surfaceContainerLow,
                      borderRadius: BorderRadius.circular(9),
                    ),
                    alignment: Alignment.center,
                    child: Icon(
                      model == null
                          ? Icons.translate_rounded
                          : Icons.auto_awesome_outlined,
                      size: 18,
                      color: colors.onSurfaceVariant,
                    ),
                  ),
                  onTap: () => onEdit(provider.id),
                  title: Text(provider.displayName),
                  subtitle: Text(
                    [
                      if (model != null && model.isNotEmpty) model,
                      if (provider.hasStoredSecret) labels.secretStored,
                    ].join(' · '),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                  trailing: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      IconButton(
                        tooltip: labels.edit,
                        onPressed: () => onEdit(provider.id),
                        icon: const Icon(Icons.edit_outlined),
                      ),
                      IconButton(
                        tooltip: labels.delete,
                        onPressed: () => onDelete(provider.id),
                        icon: const Icon(Icons.delete_outline_rounded),
                      ),
                    ],
                  ),
                );
              },
            ),
    );
  }
}

class ProviderEditorView extends StatefulWidget {
  const ProviderEditorView({
    required this.labels,
    required this.types,
    required this.draftId,
    required this.typeId,
    required this.fields,
    required this.storedSecretKeys,
    required this.testing,
    required this.testResult,
    required this.saving,
    required this.operationError,
    required this.onIdChanged,
    required this.onTypeChanged,
    required this.onFieldChanged,
    required this.onTest,
    required this.onSave,
    required this.onCancel,
    super.key,
    this.idReadOnly = false,
    this.discovery,
    this.loadingModels = false,
    this.onFetchModels,
  });

  final ProvidersSettingsLabels labels;
  final List<ProviderTypeOption> types;
  final String draftId;
  final String typeId;
  final Map<String, String> fields;
  final Set<String> storedSecretKeys;
  final bool testing;
  final ProviderTestResult? testResult;
  final bool saving;
  final String? operationError;
  final bool idReadOnly;
  final ProviderModelDiscovery? discovery;
  final bool loadingModels;
  final VoidCallback? onFetchModels;
  final ValueChanged<String> onIdChanged;
  final ValueChanged<String> onTypeChanged;
  final void Function(String key, String value) onFieldChanged;
  final VoidCallback onTest;
  final VoidCallback onSave;
  final VoidCallback onCancel;

  @override
  State<ProviderEditorView> createState() => _ProviderEditorViewState();
}

class _ProviderEditorViewState extends State<ProviderEditorView> {
  late final TextEditingController _idController = TextEditingController(
    text: widget.draftId,
  );
  final TextEditingController _searchController = TextEditingController();
  final Map<String, TextEditingController> _fieldControllers = {};
  String _query = '';
  String _modelQuery = '';
  bool _showReferenceModels = false;
  bool _advancedOpen = false;

  @override
  void initState() {
    super.initState();
    _syncFieldControllers();
  }

  @override
  void didUpdateWidget(covariant ProviderEditorView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.discovery != oldWidget.discovery &&
        widget.discovery?.succeeded == true) {
      _showReferenceModels = false;
    }
    if (oldWidget.draftId != widget.draftId &&
        _idController.text != widget.draftId) {
      _idController.text = widget.draftId;
    }
    if (oldWidget.typeId != widget.typeId) {
      _advancedOpen = false;
      _modelQuery = '';
      _showReferenceModels = false;
      _syncFieldControllers();
    } else {
      for (final entry in _fieldControllers.entries) {
        final value = widget.fields[entry.key] ?? '';
        if (entry.value.text != value) entry.value.text = value;
      }
    }
  }

  void _syncFieldControllers() {
    for (final controller in _fieldControllers.values) {
      controller.dispose();
    }
    _fieldControllers
      ..clear()
      ..addEntries([
        for (final field in _selected?.fields ?? const <ProviderFieldSpec>[])
          MapEntry(
            field.key,
            TextEditingController(
              text: widget.fields[field.key] ?? field.defaultValue ?? '',
            ),
          ),
      ]);
  }

  ProviderTypeOption? get _selected =>
      findProviderCatalogOption(widget.types, presetId: widget.typeId);

  @override
  void dispose() {
    _idController.dispose();
    _searchController.dispose();
    for (final controller in _fieldControllers.values) {
      controller.dispose();
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final selected = _selected;
    final labels = widget.labels;

    if (selected == null && !widget.idReadOnly) {
      return _buildPresetPicker(context, labels);
    }
    if (selected == null) {
      return AlertDialog(
        title: Text(labels.edit),
        content: Text(labels.validationMissing),
        actions: [
          TextButton(onPressed: widget.onCancel, child: Text(labels.cancel)),
        ],
      );
    }

    final normalFields = selected.fields
        .where((field) => !field.advanced)
        .toList(growable: false);
    final advancedFields = selected.fields
        .where((field) => field.advanced)
        .toList(growable: false);

    return AlertDialog(
      title: Text(
        widget.idReadOnly ? '${labels.edit} · ${selected.label}' : labels.add,
      ),
      content: SizedBox(
        width: 460,
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (!widget.idReadOnly)
                DropdownButtonFormField<String>(
                  initialValue: selected.id,
                  decoration: InputDecoration(labelText: labels.typeLabel),
                  items: [
                    for (final type in widget.types)
                      DropdownMenuItem(value: type.id, child: Text(type.label)),
                  ],
                  onChanged: widget.idReadOnly
                      ? null
                      : (value) {
                          if (value != null) widget.onTypeChanged(value);
                        },
                ),
              const SizedBox(height: 12),
              for (final field in normalFields) ...[
                _field(field, labels),
                const SizedBox(height: 12),
              ],
              ...[
                Align(
                  alignment: Alignment.centerLeft,
                  child: TextButton.icon(
                    onPressed: () =>
                        setState(() => _advancedOpen = !_advancedOpen),
                    icon: Icon(
                      _advancedOpen
                          ? Icons.expand_less_rounded
                          : Icons.expand_more_rounded,
                    ),
                    label: Text(t.settings.providers.advanced),
                  ),
                ),
                if (_advancedOpen) ...[
                  TextField(
                    key: const ValueKey('provider-id'),
                    enabled: !widget.idReadOnly,
                    controller: _idController,
                    decoration: InputDecoration(labelText: labels.idLabel),
                    onChanged: widget.onIdChanged,
                  ),
                  const SizedBox(height: 12),
                  for (final field in advancedFields) ...[
                    _field(field, labels),
                    const SizedBox(height: 12),
                  ],
                ],
              ],
              if (selected.isLlm)
                Text(
                  t.settings.providers.test_model_hint,
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              if (widget.testResult != null)
                StatusMessage(
                  kind: switch (widget.testResult!.status) {
                    ProviderTestStatus.passed => StatusKind.success,
                    ProviderTestStatus.failed => StatusKind.error,
                    ProviderTestStatus.testing => StatusKind.progress,
                    ProviderTestStatus.idle => StatusKind.info,
                  },
                  title: widget.testResult!.status == ProviderTestStatus.passed
                      ? (selected.isLlm
                            ? t.settings.providers.test_model_passed
                            : labels.testPassed)
                      : widget.testResult!.status == ProviderTestStatus.failed
                      ? labels.testFailed
                      : labels.testing,
                  body:
                      widget.testResult!.message ??
                      (widget.testResult!.errorCode == 'validation_missing'
                          ? labels.validationMissing
                          : null),
                ),
              if (widget.operationError != null) ...[
                const SizedBox(height: 12),
                StatusMessage(
                  kind: StatusKind.error,
                  title: widget.operationError!,
                ),
              ],
            ],
          ),
        ),
      ),
      actions: [
        TextButton(onPressed: widget.onCancel, child: Text(labels.cancel)),
        OutlinedButton(
          onPressed: widget.testing ? null : widget.onTest,
          child: Text(
            widget.testing
                ? labels.testing
                : selected.isLlm
                ? t.settings.providers.test_model
                : labels.test,
          ),
        ),
        FilledButton(
          onPressed: widget.saving ? null : widget.onSave,
          child: Text(labels.save),
        ),
      ],
    );
  }

  Widget _field(ProviderFieldSpec field, ProvidersSettingsLabels labels) {
    final input = TextField(
      key: ValueKey('provider-field-${field.key}'),
      obscureText: field.secret,
      controller: _fieldControllers[field.key],
      decoration: InputDecoration(
        labelText: field.label,
        hintText: field.secret && widget.storedSecretKeys.contains(field.key)
            ? labels.secretPlaceholder
            : field.placeholder,
        helperText: field.secret && widget.storedSecretKeys.contains(field.key)
            ? labels.secretStored
            : null,
      ),
      onChanged: (value) => widget.onFieldChanged(field.key, value),
    );
    if (field.key != 'defaultModel' || widget.onFetchModels == null) {
      return input;
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        input,
        const SizedBox(height: 8),
        Align(
          alignment: Alignment.centerLeft,
          child: OutlinedButton.icon(
            onPressed: widget.loadingModels ? null : widget.onFetchModels,
            icon: widget.loadingModels
                ? const SizedBox.square(
                    dimension: 16,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Icon(Icons.refresh_rounded),
            label: Text(t.settings.providers.fetch_models),
          ),
        ),
        _modelList(field),
      ],
    );
  }

  Widget _modelList(ProviderFieldSpec field) {
    final result = widget.discovery;
    final labels = t.settings.providers;
    final live = result?.liveModels ?? const <String>[];
    final references = result?.referenceModels ?? const <String>[];
    final models = _showReferenceModels ? references : live;
    final query = _modelQuery.trim().toLowerCase();
    final filtered = models
        .where((id) => id.toLowerCase().contains(query))
        .toList();
    final error = switch (result?.errorCode) {
      'auth_error' => labels.model_auth_failed,
      'rate_limited' => labels.model_rate_limited,
      'unsupported' => labels.model_unsupported,
      'timeout' => labels.model_timeout,
      'validation_missing' => labels.model_auto_hint,
      null => null,
      _ => labels.model_failed,
    };
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const SizedBox(height: 8),
        Text(
          labels.model_auto_hint,
          style: Theme.of(context).textTheme.bodySmall,
        ),
        if (error != null) ...[
          const SizedBox(height: 8),
          Text(
            error,
            style: TextStyle(color: Theme.of(context).colorScheme.error),
          ),
        ],
        if (references.isNotEmpty)
          CheckboxListTile(
            dense: true,
            contentPadding: EdgeInsets.zero,
            title: Text(labels.model_reference),
            value: _showReferenceModels,
            onChanged: (value) =>
                setState(() => _showReferenceModels = value ?? false),
          ),
        if (result?.succeeded == true || models.isNotEmpty) ...[
          const SizedBox(height: 8),
          Text(
            '${_showReferenceModels ? labels.model_reference : labels.model_live} · ${models.length}',
            style: Theme.of(context).textTheme.labelLarge,
          ),
          if (result?.queriedAt case final DateTime updated)
            Text(
              '${labels.model_updated} ${TimeOfDay.fromDateTime(updated).format(context)}',
              style: Theme.of(context).textTheme.bodySmall,
            ),
          const SizedBox(height: 8),
          TextField(
            key: ValueKey('provider-model-search-${widget.typeId}'),
            decoration: InputDecoration(
              prefixIcon: const Icon(Icons.search_rounded),
              labelText: labels.model_search,
            ),
            onChanged: (value) => setState(() => _modelQuery = value),
          ),
          const SizedBox(height: 8),
          if (filtered.isEmpty)
            Text(labels.model_empty)
          else
            SizedBox(
              height: (filtered.length * 48.0).clamp(48.0, 192.0),
              child: ListView.builder(
                key: const ValueKey('provider-model-list'),
                itemCount: filtered.length,
                itemExtent: 48,
                itemBuilder: (context, index) {
                  final model = filtered[index];
                  final selected = _fieldControllers[field.key]?.text == model;
                  return ListTile(
                    key: ValueKey('provider-model-$model'),
                    dense: true,
                    selected: selected,
                    title: Text(
                      model,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                    trailing: selected
                        ? const Icon(Icons.check_rounded, size: 18)
                        : null,
                    onTap: () {
                      setState(
                        () => _fieldControllers[field.key]?.text = model,
                      );
                      widget.onFieldChanged(field.key, model);
                    },
                  );
                },
              ),
            ),
        ],
      ],
    );
  }

  Widget _buildPresetPicker(
    BuildContext context,
    ProvidersSettingsLabels labels,
  ) {
    final query = _query.trim().toLowerCase();
    final filtered = widget.types
        .where((type) {
          return query.isEmpty ||
              type.label.toLowerCase().contains(query) ||
              type.id.toLowerCase().contains(query) ||
              catalogDescription(type).toLowerCase().contains(query);
        })
        .toList(growable: false);

    return AlertDialog(
      title: Text(labels.add),
      content: SizedBox(
        width: 540,
        height: 520,
        child: Column(
          children: [
            TextField(
              key: const ValueKey('provider-preset-search'),
              controller: _searchController,
              autofocus: true,
              decoration: InputDecoration(
                prefixIcon: const Icon(Icons.search_rounded),
                labelText: t.settings.providers.search,
              ),
              onChanged: (value) => setState(() => _query = value),
            ),
            const SizedBox(height: 12),
            Expanded(
              child: ListView(
                children: [
                  for (final category in kCatalogCategoryOrder)
                    if (filtered.any((type) => type.category == category)) ...[
                      Padding(
                        padding: const EdgeInsets.fromLTRB(12, 12, 12, 4),
                        child: Text(
                          categoryLabel(category),
                          style: Theme.of(context).textTheme.labelLarge,
                        ),
                      ),
                      for (final type in filtered.where(
                        (type) => type.category == category,
                      ))
                        ListTile(
                          key: ValueKey('provider-preset-${type.id}'),
                          title: Text(type.label),
                          subtitle: Text(
                            catalogDescription(type),
                            maxLines: 2,
                            overflow: TextOverflow.ellipsis,
                          ),
                          trailing: type.isExperimental
                              ? Tooltip(
                                  message: stabilityLabel(type.stability),
                                  child: const Icon(
                                    Icons.science_outlined,
                                    size: 18,
                                  ),
                                )
                              : const Icon(Icons.chevron_right_rounded),
                          onTap: () => widget.onTypeChanged(type.id),
                        ),
                    ],
                  if (filtered.isEmpty)
                    Padding(
                      padding: const EdgeInsets.all(24),
                      child: Center(child: Text(labels.empty)),
                    ),
                ],
              ),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(onPressed: widget.onCancel, child: Text(labels.cancel)),
      ],
    );
  }
}
