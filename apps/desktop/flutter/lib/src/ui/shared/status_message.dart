import 'package:flutter/material.dart';

enum StatusKind { info, progress, success, warning, error }

class StatusMessage extends StatelessWidget {
  const StatusMessage({
    required this.title,
    super.key,
    this.body,
    this.kind = StatusKind.info,
    this.action,
  });

  final String title;
  final String? body;
  final StatusKind kind;
  final Widget? action;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final (icon, color) = switch (kind) {
      StatusKind.info => (Icons.info_outline_rounded, scheme.onSurfaceVariant),
      StatusKind.progress => (Icons.hourglass_top_rounded, scheme.primary),
      StatusKind.success => (
        Icons.check_circle_outline_rounded,
        scheme.primary,
      ),
      StatusKind.warning => (Icons.warning_amber_rounded, scheme.tertiary),
      StatusKind.error => (Icons.error_outline_rounded, scheme.error),
    };

    return Semantics(
      liveRegion: kind == StatusKind.error || kind == StatusKind.warning,
      child: DecoratedBox(
        decoration: BoxDecoration(
          color: color.withValues(alpha: 0.08),
          borderRadius: BorderRadius.circular(10),
          border: Border.all(color: color.withValues(alpha: 0.28)),
        ),
        child: Padding(
          padding: const EdgeInsets.fromLTRB(14, 12, 14, 12),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Icon(icon, size: 20, color: color),
                  const SizedBox(width: 10),
                  Expanded(
                    child: Text(
                      title,
                      style: Theme.of(context).textTheme.titleMedium
                          ?.copyWith(color: scheme.onSurface),
                    ),
                  ),
                ],
              ),
              if (body != null && body!.trim().isNotEmpty) ...[
                const SizedBox(height: 6),
                Padding(
                  padding: const EdgeInsets.only(left: 30),
                  child: Text(
                    body!,
                    style: Theme.of(context).textTheme.bodyMedium
                        ?.copyWith(color: scheme.onSurfaceVariant),
                  ),
                ),
              ],
              if (action != null) ...[
                const SizedBox(height: 12),
                Padding(
                  padding: const EdgeInsets.only(left: 30),
                  child: action!,
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}
