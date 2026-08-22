import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../shared/status_message.dart';
import '../settings_labels.dart';

class ServicesSettingsView extends StatelessWidget {
  const ServicesSettingsView({
    required this.labels,
    required this.services,
    required this.loading,
    required this.onEnabledChanged,
    required this.onMakeDefault,
    required this.onConfigureProviders,
    super.key,
    this.onAdd,
    this.onReorderTranslation,
    this.errorCode,
    this.onRetry,
  });

  final ServicesSettingsLabels labels;
  final List<ServiceRecord> services;
  final bool loading;
  final void Function(String id, bool enabled) onEnabledChanged;
  final ValueChanged<String> onMakeDefault;
  final VoidCallback onConfigureProviders;
  final VoidCallback? onAdd;
  final void Function(int oldIndex, int newIndex)? onReorderTranslation;
  final String? errorCode;
  final VoidCallback? onRetry;

  @override
  Widget build(BuildContext context) {
    if (loading) {
      return const Center(child: CircularProgressIndicator());
    }

    final translation = services
        .where((service) => service.kind == 'translation')
        .toList();
    final ocr = services.where((service) => service.kind == 'ocr').toList();

    if (services.isEmpty) {
      return Padding(
        padding: const EdgeInsets.all(24),
        child: StatusMessage(
          kind: StatusKind.warning,
          title: labels.empty,
          action: OutlinedButton(
            onPressed: onConfigureProviders,
            child: Text(labels.configureProviders),
          ),
        ),
      );
    }

    return ListView(
      padding: const EdgeInsets.fromLTRB(20, 16, 24, 24),
      children: [
        if (errorCode != null)
          StatusMessage(
            kind: StatusKind.error,
            title: labels.errorMessage?.call(errorCode) ?? labels.loading,
            action: onRetry == null
                ? null
                : OutlinedButton(
                    onPressed: onRetry,
                    child: Text(labels.loading),
                  ),
          ),
        Row(
          children: [
            Expanded(
              child: Text(
                labels.translation,
                style: Theme.of(context).textTheme.titleMedium,
              ),
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
        if (onReorderTranslation == null)
          for (final service in translation)
            _ServiceTile(
              key: ValueKey(service.id),
              labels: labels,
              service: service,
              onEnabledChanged: onEnabledChanged,
              onMakeDefault: onMakeDefault,
            )
        else
          ReorderableListView.builder(
            shrinkWrap: true,
            physics: const NeverScrollableScrollPhysics(),
            buildDefaultDragHandles: false,
            itemCount: translation.length,
            onReorderItem: onReorderTranslation!,
            itemBuilder: (context, index) => _ServiceTile(
              key: ValueKey(translation[index].id),
              labels: labels,
              service: translation[index],
              reorderIndex: index,
              onEnabledChanged: onEnabledChanged,
              onMakeDefault: onMakeDefault,
            ),
          ),
        if (ocr.isNotEmpty) ...[
          const SizedBox(height: 20),
          Text(labels.ocr, style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 8),
          for (final service in ocr)
            _ServiceTile(
              labels: labels,
              service: service,
              onEnabledChanged: onEnabledChanged,
              onMakeDefault: onMakeDefault,
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
    this.reorderIndex,
  });

  final ServicesSettingsLabels labels;
  final ServiceRecord service;
  final void Function(String id, bool enabled) onEnabledChanged;
  final ValueChanged<String> onMakeDefault;
  final int? reorderIndex;

  @override
  Widget build(BuildContext context) {
    return SwitchListTile(
      contentPadding: EdgeInsets.zero,
      title: Row(
        children: [
          Expanded(child: Text(service.name)),
          if (!service.isDefault)
            IconButton(
              tooltip: labels.makeDefault,
              onPressed: () => onMakeDefault(service.id),
              icon: const Icon(Icons.star_outline_rounded),
            ),
        ],
      ),
      subtitle: Text(
        service.isDefault
            ? '${service.providerName} · ${labels.isDefault}'
            : service.providerName,
      ),
      value: service.enabled,
      onChanged: (value) => onEnabledChanged(service.id, value),
      secondary: reorderIndex == null
          ? null
          : ReorderableDragStartListener(
              index: reorderIndex!,
              child: const Padding(
                padding: EdgeInsets.all(12),
                child: Icon(Icons.drag_indicator_rounded),
              ),
            ),
    );
  }
}
