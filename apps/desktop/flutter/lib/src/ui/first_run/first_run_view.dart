import 'package:flutter/material.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_ui/linguaray_ui.dart' show BrandLogo;

final class FirstRunLabels {
  const FirstRunLabels({
    required this.title,
    required this.subtitle,
    required this.permissionsTitle,
    required this.permissionsBody,
    required this.accessibility,
    required this.screenRecording,
    required this.shortcutsTitle,
    required this.shortcutsBody,
    required this.servicesTitle,
    required this.servicesBody,
    required this.granted,
    required this.denied,
    required this.notRequired,
    required this.unknown,
    required this.checking,
    required this.conflict,
    required this.noProvider,
    required this.ready,
    required this.grant,
    required this.recheck,
    required this.configureServices,
    required this.start,
    required this.skip,
  });

  final String title;
  final String subtitle;
  final String permissionsTitle;
  final String permissionsBody;
  final String accessibility;
  final String screenRecording;
  final String shortcutsTitle;
  final String shortcutsBody;
  final String servicesTitle;
  final String servicesBody;
  final String granted;
  final String denied;
  final String notRequired;
  final String unknown;
  final String checking;
  final String conflict;
  final String noProvider;
  final String ready;
  final String grant;
  final String recheck;
  final String configureServices;
  final String start;
  final String skip;
}

class FirstRunView extends StatelessWidget {
  const FirstRunView({
    required this.labels,
    required this.permissions,
    required this.shortcutsReady,
    required this.shortcutConflict,
    required this.hasServices,
    required this.checkingPermissions,
    required this.onGrantAccessibility,
    required this.onGrantScreenRecording,
    required this.onRecheck,
    required this.onConfigureServices,
    required this.onStart,
    required this.onSkip,
    super.key,
  });

  final FirstRunLabels labels;
  final AccessSnapshot permissions;
  final bool shortcutsReady;
  final bool shortcutConflict;
  final bool hasServices;
  final bool checkingPermissions;
  final VoidCallback onGrantAccessibility;
  final VoidCallback onGrantScreenRecording;
  final VoidCallback onRecheck;
  final VoidCallback onConfigureServices;
  final VoidCallback onStart;
  final VoidCallback onSkip;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Material(
      color: theme.scaffoldBackgroundColor,
      child: LayoutBuilder(
        builder: (context, constraints) {
          return SingleChildScrollView(
            padding: const EdgeInsets.fromLTRB(32, 28, 32, 28),
            child: Center(
              child: ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: 560),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    const Align(
                      alignment: Alignment.centerLeft,
                      child: BrandLogo(size: 40),
                    ),
                    const SizedBox(height: 20),
                    Text(labels.title, style: theme.textTheme.headlineMedium),
                    const SizedBox(height: 8),
                    Text(
                      labels.subtitle,
                      style: theme.textTheme.bodyMedium?.copyWith(
                        color: theme.colorScheme.onSurfaceVariant,
                      ),
                    ),
                    const SizedBox(height: 24),
                    _StepCard(
                      index: '1',
                      title: labels.permissionsTitle,
                      body: labels.permissionsBody,
                      trailing: checkingPermissions
                          ? const SizedBox.square(
                              dimension: 18,
                              child: CircularProgressIndicator(strokeWidth: 2),
                            )
                          : null,
                      child: Column(
                        children: [
                          _AccessRow(
                            title: labels.accessibility,
                            state: permissions.accessibility,
                            labels: labels,
                            onGrant: onGrantAccessibility,
                          ),
                          _AccessRow(
                            title: labels.screenRecording,
                            state: permissions.screenRecording,
                            labels: labels,
                            onGrant: onGrantScreenRecording,
                          ),
                        ],
                      ),
                    ),
                    const SizedBox(height: 12),
                    _StepCard(
                      index: '2',
                      title: labels.shortcutsTitle,
                      body: shortcutConflict
                          ? labels.conflict
                          : labels.shortcutsBody,
                      statusColor: shortcutConflict
                          ? theme.colorScheme.error
                          : shortcutsReady
                          ? theme.colorScheme.primary
                          : null,
                    ),
                    const SizedBox(height: 12),
                    _StepCard(
                      index: '3',
                      title: labels.servicesTitle,
                      body: hasServices ? labels.ready : labels.noProvider,
                      action: hasServices
                          ? null
                          : OutlinedButton(
                              onPressed: onConfigureServices,
                              child: Text(labels.configureServices),
                            ),
                    ),
                    const SizedBox(height: 28),
                    Wrap(
                      spacing: 12,
                      runSpacing: 8,
                      children: [
                        FilledButton(
                          onPressed: onStart,
                          child: Text(labels.start),
                        ),
                        TextButton(onPressed: onSkip, child: Text(labels.skip)),
                        OutlinedButton(
                          onPressed: onRecheck,
                          child: Text(labels.recheck),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            ),
          );
        },
      ),
    );
  }
}

class _StepCard extends StatelessWidget {
  const _StepCard({
    required this.index,
    required this.title,
    required this.body,
    this.child,
    this.action,
    this.trailing,
    this.statusColor,
  });

  final String index;
  final String title;
  final String body;
  final Widget? child;
  final Widget? action;
  final Widget? trailing;
  final Color? statusColor;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return DecoratedBox(
      decoration: BoxDecoration(
        color: theme.colorScheme.surfaceContainerLowest,
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: theme.colorScheme.outlineVariant),
      ),
      child: Padding(
        padding: const EdgeInsets.fromLTRB(16, 14, 16, 14),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Row(
              children: [
                CircleAvatar(
                  radius: 12,
                  backgroundColor: (statusColor ?? theme.colorScheme.primary)
                      .withValues(alpha: 0.16),
                  child: Text(
                    index,
                    style: theme.textTheme.labelMedium?.copyWith(
                      color: statusColor ?? theme.colorScheme.primary,
                    ),
                  ),
                ),
                const SizedBox(width: 10),
                Expanded(
                  child: Text(title, style: theme.textTheme.titleMedium),
                ),
                if (trailing != null) trailing!,
              ],
            ),
            const SizedBox(height: 8),
            Text(
              body,
              style: theme.textTheme.bodyMedium?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
            if (child != null) ...[const SizedBox(height: 8), child!],
            if (action != null) ...[const SizedBox(height: 12), action!],
          ],
        ),
      ),
    );
  }
}

class _AccessRow extends StatelessWidget {
  const _AccessRow({
    required this.title,
    required this.state,
    required this.labels,
    required this.onGrant,
  });

  final String title;
  final AccessState state;
  final FirstRunLabels labels;
  final VoidCallback onGrant;

  @override
  Widget build(BuildContext context) {
    final (label, color, canGrant) = switch (state) {
      AccessState.granted => (
        labels.granted,
        Theme.of(context).colorScheme.primary,
        false,
      ),
      AccessState.denied => (
        labels.denied,
        Theme.of(context).colorScheme.error,
        true,
      ),
      AccessState.notRequired => (
        labels.notRequired,
        Theme.of(context).colorScheme.onSurfaceVariant,
        false,
      ),
      AccessState.checking => (
        labels.checking,
        Theme.of(context).colorScheme.primary,
        false,
      ),
      AccessState.unknown => (
        labels.unknown,
        Theme.of(context).colorScheme.onSurfaceVariant,
        true,
      ),
    };

    return ListTile(
      contentPadding: EdgeInsets.zero,
      title: Text(title),
      subtitle: Text(label, style: TextStyle(color: color)),
      trailing: canGrant
          ? TextButton(onPressed: onGrant, child: Text(labels.grant))
          : Icon(
              state == AccessState.granted || state == AccessState.notRequired
                  ? Icons.check_circle_outline_rounded
                  : Icons.radio_button_unchecked,
              color: color,
            ),
    );
  }
}
