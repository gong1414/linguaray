import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../i18n/i18n.dart';
import '../../shared/i18n_labels.dart';
import 'general_settings_view.dart';
import 'settings_view_model.dart';

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
      pageTitle: t.settings.navigation.general_settings,
      preferences: preferences,
      languages: state.languages,
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
    );
  }
}
