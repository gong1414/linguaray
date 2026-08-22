import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../platform/onboarding_controller.dart';
import '../i18n_labels.dart';
import '../settings/view_models/permissions_view_model.dart';
import '../settings/view_models/settings_view_model.dart';
import '../settings/view_models/shortcuts_view_model.dart';
import 'first_run_view.dart';

class FirstRunScreen extends ConsumerWidget {
  const FirstRunScreen({super.key});

  void _complete(BuildContext context) {
    onboardingController.complete();
    context.go('/translate');
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final permissions = ref.watch(permissionsViewModelProvider);
    final shortcuts = ref.watch(shortcutsViewModelProvider);
    final services = ref.watch(servicesSettingsViewModelProvider);

    return FirstRunView(
      labels: firstRunLabels(),
      permissions: permissions,
      shortcutsReady: !shortcuts.hasConflict && !shortcuts.loading,
      shortcutConflict: shortcuts.hasConflict,
      hasServices: services.services.any((item) => item.enabled),
      checkingPermissions: permissions.accessibility.name == 'unknown',
      onGrantAccessibility: () => ref
          .read(permissionsViewModelProvider.notifier)
          .requestAccessibility(),
      onGrantScreenRecording: () => ref
          .read(permissionsViewModelProvider.notifier)
          .requestScreenRecording(),
      onRecheck: () =>
          ref.read(permissionsViewModelProvider.notifier).refresh(),
      onConfigureServices: () => context.go('/settings/providers'),
      onStart: () => _complete(context),
      onSkip: () => _complete(context),
    );
  }
}
