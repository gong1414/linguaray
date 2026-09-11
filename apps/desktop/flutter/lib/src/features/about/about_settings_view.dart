import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_ui/linguaray_ui.dart' show BrandLogo;

import '../../shared/settings_labels.dart';
import '../../shared/settings_page.dart';

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
    return SettingsPage(
      title: labels.title,
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(vertical: 24),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const BrandLogo(size: 64),
              const SizedBox(width: 24),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(info.appName, style: theme.textTheme.headlineMedium),
                    const SizedBox(height: 8),
                    Text(
                      'v${info.version} (${info.buildNumber}) · ${info.platformLabel}',
                      style: theme.textTheme.bodySmall,
                    ),
                    const SizedBox(height: 16),
                    OutlinedButton.icon(
                      onPressed: onCopyVersion,
                      icon: Icon(
                        copied ? Icons.check_rounded : Icons.copy_rounded,
                        size: 16,
                      ),
                      label: Text(copied ? labels.copied : labels.copyVersion),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
        const Divider(),
        const SizedBox(height: 12),
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
        const SizedBox(height: 24),
        Text(labels.copyright, style: theme.textTheme.bodySmall),
      ],
    );
  }
}
