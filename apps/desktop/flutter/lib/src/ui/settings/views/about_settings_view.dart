import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_ui/linguaray_ui.dart' show BrandLogo;

import '../settings_labels.dart';

class AboutSettingsView extends StatelessWidget {
  const AboutSettingsView({
    required this.labels,
    required this.info,
    required this.copied,
    required this.onCopyVersion,
    required this.onOpenWebsite,
    required this.onOpenChangelog,
    required this.onOpenIssues,
    required this.onOpenLicense,
    super.key,
  });

  final AboutSettingsLabels labels;
  final AboutInfo info;
  final bool copied;
  final VoidCallback onCopyVersion;
  final VoidCallback onOpenWebsite;
  final VoidCallback onOpenChangelog;
  final VoidCallback onOpenIssues;
  final VoidCallback onOpenLicense;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return ListView(
      padding: const EdgeInsets.fromLTRB(32, 36, 32, 36),
      children: [
        const Center(child: BrandLogo(size: 48)),
        const SizedBox(height: 16),
        Text(
          info.appName,
          textAlign: TextAlign.center,
          style: theme.textTheme.headlineMedium,
        ),
        const SizedBox(height: 6),
        Text(
          'v${info.version} (${info.buildNumber}) · ${info.platformLabel}',
          textAlign: TextAlign.center,
          style: theme.textTheme.bodyMedium?.copyWith(
            color: theme.colorScheme.onSurfaceVariant,
          ),
        ),
        const SizedBox(height: 8),
        Text(
          labels.copyright,
          textAlign: TextAlign.center,
          style: theme.textTheme.bodyMedium,
        ),
        const SizedBox(height: 16),
        Center(
          child: OutlinedButton(
            onPressed: onCopyVersion,
            child: Text(copied ? labels.copied : labels.copyVersion),
          ),
        ),
        const SizedBox(height: 24),
        ListTile(
          title: Text(labels.website),
          trailing: const Icon(Icons.open_in_new_rounded),
          onTap: onOpenWebsite,
        ),
        ListTile(
          title: Text(labels.changelog),
          trailing: const Icon(Icons.open_in_new_rounded),
          onTap: onOpenChangelog,
        ),
        ListTile(
          title: Text(labels.issues),
          trailing: const Icon(Icons.open_in_new_rounded),
          onTap: onOpenIssues,
        ),
        ListTile(
          title: Text(labels.license),
          subtitle: Text(info.license),
          trailing: const Icon(Icons.open_in_new_rounded),
          onTap: onOpenLicense,
        ),
      ],
    );
  }
}
