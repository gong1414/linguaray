import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../app/dependencies.dart';
import '../../i18n/i18n.dart';
import '../../shared/i18n_labels.dart';
import '../providers/providers_settings_screen.dart';
import 'services_settings_view.dart';
import 'services_settings_view_model.dart';

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
      onConfigureProviders: () => unawaited(showProviderManager(context)),
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
      .read(providerSettingsRepositoryProvider)
      .listProviders();
  if (providers.isEmpty && context.mounted) {
    await showProviderManager(context);
    providers = await ref
        .read(providerSettingsRepositoryProvider)
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
