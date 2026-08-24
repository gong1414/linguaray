import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../models/settings_navigation.dart';
import '../i18n_labels.dart';
import 'settings_shell_view.dart';

class SettingsHostScreen extends StatelessWidget {
  const SettingsHostScreen({
    required this.location,
    required this.child,
    super.key,
  });

  final String location;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    final section = settingsDestinationForLocation(location).section;
    return SettingsShellView(
      labels: settingsShellLabels(),
      section: section,
      onSectionSelected: (section) => context.go(section.destination.location),
      child: child,
    );
  }
}
