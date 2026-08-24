import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../../models/settings_navigation.dart';
import '../../services/app_windows.dart'
    show hideSettingsWindow, settingsWindowController;
import '../../ui/history/history_screen.dart';
import '../../ui/settings/data_transfer_settings_screen.dart';
import '../../ui/settings/library_settings_screens.dart';
import '../../ui/settings/settings_screens.dart';
import '../../ui/updates/updates_screen.dart';
import '../../utils/platform_util.dart';

/// Production navigation contains only preferences and management pages.
/// Translation itself is presented by the tray/shortcut-driven quick panel.
List<RouteBase> get $appRoutes => <RouteBase>[
  ShellRoute(
    builder: (context, state, child) => _SettingsWindowFrame(
      child: SettingsHostScreen(location: state.uri.path, child: child),
    ),
    routes: [
      for (final destination in SettingsDestination.values)
        GoRoute(
          path: destination.location,
          pageBuilder: (_, state) =>
              _noTransitionPage(state, _settingsPage(destination)),
        ),
    ],
  ),
];

Widget _settingsPage(SettingsDestination destination) => switch (destination) {
  SettingsDestination.settingsTranslation => const TranslationSettingsScreen(),
  SettingsDestination.settingsTranslationServices =>
    const ServicesSettingsScreen(serviceKind: 'translation'),
  SettingsDestination.settingsFavorites => const HistoryScreen(
    initialFilter: HistoryFilter.favorites,
    lockFilter: true,
  ),
  SettingsDestination.settingsHistory => const HistoryScreen(
    initialFilter: HistoryFilter.all,
  ),
  SettingsDestination.settingsGlossary => const GlossarySettingsScreen(),
  SettingsDestination.settingsVocabulary => const VocabularySettingsScreen(),
  SettingsDestination.settingsOcr => const OcrSettingsScreen(),
  SettingsDestination.settingsOcrServices => const ServicesSettingsScreen(
    serviceKind: 'ocr',
  ),
  SettingsDestination.settingsGeneral => const GeneralSettingsScreen(),
  SettingsDestination.settingsPermissions => const PermissionsSettingsScreen(),
  SettingsDestination.settingsDataTransfer =>
    const DataTransferSettingsScreen(),
  SettingsDestination.settingsIntegration => const AdvancedSettingsScreen(),
  SettingsDestination.settingsUpdates => const UpdatesSettingsScreen(),
  SettingsDestination.settingsAbout => const AboutSettingsScreen(),
};

Page<void> _noTransitionPage(GoRouterState state, Widget child) =>
    NoTransitionPage<void>(key: state.pageKey, child: child);

class _SettingsWindowFrame extends StatelessWidget {
  const _SettingsWindowFrame({required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    if (!kIsWindows) return child;
    final theme = Theme.of(context);
    return Column(
      children: [
        GestureDetector(
          behavior: HitTestBehavior.opaque,
          onPanStart: (_) => settingsWindowController.window.startDragging(),
          child: Container(
            height: 38,
            color: theme.colorScheme.surface,
            child: Row(
              children: [
                const SizedBox(width: 14),
                Text('LinguaRay', style: theme.textTheme.labelLarge),
                const Spacer(),
                _WindowButton(
                  icon: Icons.remove_rounded,
                  onPressed: () => settingsWindowController.window.minimize(),
                ),
                _WindowButton(
                  icon: Icons.crop_square_rounded,
                  onPressed: () {
                    final window = settingsWindowController.window;
                    window.isMaximized
                        ? window.unmaximize()
                        : window.maximize();
                  },
                ),
                const _WindowButton(
                  icon: Icons.close_rounded,
                  onPressed: hideSettingsWindow,
                ),
              ],
            ),
          ),
        ),
        Expanded(child: child),
      ],
    );
  }
}

class _WindowButton extends StatelessWidget {
  const _WindowButton({required this.icon, required this.onPressed});

  final IconData icon;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) => SizedBox(
    width: 46,
    height: 38,
    child: IconButton(
      padding: EdgeInsets.zero,
      iconSize: 16,
      onPressed: onPressed,
      icon: Icon(icon),
    ),
  );
}
