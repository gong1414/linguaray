import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../../i18n/i18n.dart';
import '../../../routes/settings/provider_catalog.dart';
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
    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(20, 16, 20, 8),
          child: Row(
            children: [
              Expanded(
                child: Text(
                  labels.title,
                  style: Theme.of(context).textTheme.titleLarge,
                ),
              ),
              FilledButton.icon(
                onPressed: onAdd,
                icon: const Icon(Icons.add_rounded),
                label: Text(labels.add),
              ),
            ],
          ),
        ),
        Expanded(
          child: loading
              ? const Center(child: CircularProgressIndicator())
              : providers.isEmpty
              ? Padding(
                  padding: const EdgeInsets.all(24),
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
                  padding: const EdgeInsets.fromLTRB(12, 0, 12, 16),
                  itemCount: providers.length,
                  separatorBuilder: (_, _) => const Divider(),
                  itemBuilder: (context, index) {
                    final provider = providers[index];
                    return ListTile(
                      title: Text(provider.displayName),
                      subtitle: Text(
                        provider.hasStoredSecret
                            ? '${provider.typeId} · ${labels.secretStored}'
                            : provider.typeId,
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
        ),
      ],
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
    this.models = const [],
    this.loadingModels = false,
    this.modelsError,
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
  final List<String> models;
  final bool loadingModels;
  final String? modelsError;
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
  bool _advancedOpen = false;

  @override
  void initState() {
    super.initState();
    _syncFieldControllers();
  }

  @override
  void didUpdateWidget(covariant ProviderEditorView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.draftId != widget.draftId &&
        _idController.text != widget.draftId) {
      _idController.text = widget.draftId;
    }
    if (oldWidget.typeId != widget.typeId) {
      _advancedOpen = false;
      _syncFieldControllers();
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
      title: Text(widget.idReadOnly ? labels.edit : labels.add),
      content: SizedBox(
        width: 460,
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              TextField(
                key: const ValueKey('provider-id'),
                enabled: !widget.idReadOnly,
                controller: _idController,
                decoration: InputDecoration(labelText: labels.idLabel),
                onChanged: widget.onIdChanged,
              ),
              const SizedBox(height: 12),
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
              if (advancedFields.isNotEmpty) ...[
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
                if (_advancedOpen)
                  for (final field in advancedFields) ...[
                    _field(field, labels),
                    const SizedBox(height: 12),
                  ],
              ],
              if (widget.testResult != null)
                StatusMessage(
                  kind: switch (widget.testResult!.status) {
                    ProviderTestStatus.passed => StatusKind.success,
                    ProviderTestStatus.failed => StatusKind.error,
                    ProviderTestStatus.testing => StatusKind.progress,
                    ProviderTestStatus.idle => StatusKind.info,
                  },
                  title: widget.testResult!.status == ProviderTestStatus.passed
                      ? labels.testPassed
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
          child: Text(widget.testing ? labels.testing : labels.test),
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
        if (widget.modelsError != null) ...[
          const SizedBox(height: 8),
          Text(
            widget.modelsError!,
            style: TextStyle(color: Theme.of(context).colorScheme.error),
          ),
        ],
        if (widget.models.isNotEmpty) ...[
          const SizedBox(height: 8),
          Wrap(
            spacing: 6,
            runSpacing: 6,
            children: [
              for (final model in widget.models.take(16))
                ActionChip(
                  label: Text(model),
                  onPressed: () {
                    _fieldControllers[field.key]?.text = model;
                    widget.onFieldChanged(field.key, model);
                  },
                ),
            ],
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
