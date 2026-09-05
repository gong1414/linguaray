import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../shared/settings_labels.dart';
import '../../shared/settings_page.dart';
import '../../shared/status_message.dart';

class ServicesSettingsView extends StatelessWidget {
  const ServicesSettingsView({
    required this.labels,
    required this.pageTitle,
    required this.services,
    required this.serviceKind,
    required this.loading,
    required this.onEnabledChanged,
    required this.onMakeDefault,
    required this.onConfigureProviders,
    super.key,
    this.onAdd,
    this.onDelete,
    this.onReorderTranslation,
    this.errorCode,
    this.onRetry,
  });

  final ServicesSettingsLabels labels;
  final String pageTitle;
  final List<ServiceRecord> services;
  final String serviceKind;
  final bool loading;
  final void Function(String id, bool enabled) onEnabledChanged;
  final ValueChanged<String> onMakeDefault;
  final ValueChanged<String>? onDelete;
  final VoidCallback onConfigureProviders;
  final VoidCallback? onAdd;
  final void Function(int oldIndex, int newIndex)? onReorderTranslation;
  final String? errorCode;
  final VoidCallback? onRetry;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final visible = services
        .where((service) => service.kind == serviceKind)
        .toList();
    final dictionaries = serviceKind == 'translation'
        ? services.where((service) => service.kind == 'dictionary').toList()
        : const <ServiceRecord>[];
    final cardColor = theme.colorScheme.surfaceContainerLowest;
    final cardShape = theme.cardTheme.shape;

    return SettingsPage(
      title: pageTitle,
      children: [
        Row(
          children: [
            Expanded(
              child: Text(
                serviceKind == 'ocr' ? labels.ocr : labels.translation,
                style: theme.textTheme.titleMedium?.copyWith(
                  fontWeight: FontWeight.w600,
                ),
              ),
            ),
            TextButton(
              onPressed: onConfigureProviders,
              child: Text(labels.configureProviders),
            ),
            if (onAdd != null)
              IconButton(
                tooltip: labels.title,
                onPressed: onAdd,
                icon: const Icon(Icons.add_rounded),
              ),
          ],
        ),
        const SizedBox(height: 8),
        if (errorCode != null)
          Padding(
            padding: const EdgeInsets.only(bottom: 12),
            child: StatusMessage(
              kind: StatusKind.error,
              title: labels.errorMessage?.call(errorCode) ?? labels.loading,
              action: onRetry == null
                  ? null
                  : OutlinedButton(
                      onPressed: onRetry,
                      child: Text(labels.loading),
                    ),
            ),
          ),
        if (loading)
          const SizedBox(
            height: 180,
            child: Center(child: CircularProgressIndicator()),
          )
        else if (visible.isEmpty)
          Material(
            color: cardColor,
            shape: cardShape,
            child: Padding(
              padding: const EdgeInsets.all(24),
              child: StatusMessage(
                kind: StatusKind.info,
                title: labels.empty,
                action: FilledButton.icon(
                  onPressed: onAdd ?? onConfigureProviders,
                  icon: const Icon(Icons.add_rounded),
                  label: Text(labels.title),
                ),
              ),
            ),
          )
        else
          Material(
            color: cardColor,
            shape: cardShape,
            clipBehavior: Clip.antiAlias,
            child: onReorderTranslation == null || serviceKind != 'translation'
                ? Column(
                    children: [
                      for (final (index, service) in visible.indexed) ...[
                        if (index > 0) const Divider(height: 1, indent: 12),
                        _ServiceTile(
                          key: ValueKey(service.id),
                          labels: labels,
                          service: service,
                          onEnabledChanged: onEnabledChanged,
                          onMakeDefault: onMakeDefault,
                          onDelete: onDelete,
                        ),
                      ],
                    ],
                  )
                : ReorderableListView.builder(
                    shrinkWrap: true,
                    physics: const NeverScrollableScrollPhysics(),
                    buildDefaultDragHandles: false,
                    itemCount: visible.length,
                    onReorderItem: onReorderTranslation!,
                    itemBuilder: (context, index) => _ServiceTile(
                      key: ValueKey(visible[index].id),
                      labels: labels,
                      service: visible[index],
                      reorderIndex: index,
                      onEnabledChanged: onEnabledChanged,
                      onMakeDefault: onMakeDefault,
                      onDelete: onDelete,
                    ),
                  ),
          ),
        if (!loading && dictionaries.isNotEmpty) ...[
          const SizedBox(height: 24),
          Text(
            labels.dictionary,
            style: theme.textTheme.titleMedium?.copyWith(
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(height: 8),
          Material(
            color: cardColor,
            shape: cardShape,
            clipBehavior: Clip.antiAlias,
            child: Column(
              children: [
                for (final (index, service) in dictionaries.indexed) ...[
                  if (index > 0) const Divider(height: 1, indent: 12),
                  _ServiceTile(
                    key: ValueKey(service.id),
                    labels: labels,
                    service: service,
                    onEnabledChanged: onEnabledChanged,
                    onMakeDefault: onMakeDefault,
                    onDelete: onDelete,
                  ),
                ],
              ],
            ),
          ),
        ],
      ],
    );
  }
}

class _ServiceTile extends StatelessWidget {
  const _ServiceTile({
    super.key,
    required this.labels,
    required this.service,
    required this.onEnabledChanged,
    required this.onMakeDefault,
    this.onDelete,
    this.reorderIndex,
  });

  final ServicesSettingsLabels labels;
  final ServiceRecord service;
  final void Function(String id, bool enabled) onEnabledChanged;
  final ValueChanged<String> onMakeDefault;
  final ValueChanged<String>? onDelete;
  final int? reorderIndex;

  String get _displayName {
    final name = service.name.trim();
    if (name.isEmpty ||
        name == service.id ||
        name == service.providerId ||
        name.contains('+')) {
      final kind = switch (service.kind) {
        'dictionary' => labels.dictionary,
        'ocr' => labels.ocr,
        _ => labels.translation,
      };
      return '${service.providerName} $kind';
    }
    return name;
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(8, 5, 10, 5),
      child: Row(
        children: [
          if (reorderIndex != null)
            ReorderableDragStartListener(
              index: reorderIndex!,
              child: const Padding(
                padding: EdgeInsets.all(8),
                child: Icon(Icons.drag_indicator_rounded, size: 19),
              ),
            )
          else
            const SizedBox(width: 8),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  _displayName,
                  style: Theme.of(context).textTheme.bodyMedium
                      ?.copyWith(fontWeight: FontWeight.w600),
                ),
                const SizedBox(height: 2),
                Text(
                  service.isDefault
                      ? '${service.providerName} · ${labels.isDefault}'
                      : service.providerName,
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                  ),
                ),
              ],
            ),
          ),
          if (!service.isDefault)
            IconButton(
              tooltip: labels.makeDefault,
              onPressed: () => onMakeDefault(service.id),
              icon: const Icon(Icons.star_outline_rounded, size: 19),
            ),
          if (onDelete != null && !service.synthesized)
            IconButton(
              tooltip: labels.delete,
              onPressed: () => onDelete!(service.id),
              icon: const Icon(Icons.remove_circle_outline_rounded, size: 18),
            ),
          Switch(
            value: service.enabled,
            onChanged: (value) => onEnabledChanged(service.id, value),
          ),
        ],
      ),
    );
  }
}
