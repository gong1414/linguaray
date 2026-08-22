import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../features.dart';
import '../../services/app_windows.dart'
    show hideWorkbenchWindow, workbenchWindowController;
import '../../ui/chrome/workbench_shell_view.dart';
import '../../ui/glossary/glossary_screen.dart';
import '../../ui/history/history_screen.dart';
import '../../ui/i18n_labels.dart';
import '../../ui/settings/settings_screens.dart';
import '../../ui/updates/updates_screen.dart';
import '../../ui/vocabulary/vocabulary_screen.dart';
import '../../utils/platform_util.dart';
import 'translation.dart';
import 'welcome.dart';

List<RouteBase> get $appRoutes => <RouteBase>[
  // An indexed-stack shell so each destination keeps its Navigator (and
  // page state — 翻译 keeps its source text and results) while the
  // sidebar switches between them.
  StatefulShellRoute.indexedStack(
    builder: (context, state, navigationShell) =>
        WorkbenchShell(location: state.uri.path, child: navigationShell),
    branches: [
      StatefulShellBranch(
        routes: [
          GoRoute(
            path: '/welcome',
            pageBuilder: (_, state) =>
                _noTransitionPage(state, const WorkbenchWelcomePage()),
          ),
        ],
      ),
      StatefulShellBranch(
        routes: [
          GoRoute(
            path: '/translate',
            pageBuilder: (_, state) =>
                _noTransitionPage(state, const WorkbenchTranslationPage()),
          ),
        ],
      ),
      if (kHistoryFeatureEnabled)
        StatefulShellBranch(
          routes: [
            GoRoute(
              path: '/history',
              pageBuilder: (_, state) =>
                  _noTransitionPage(state, const HistoryScreen()),
            ),
          ],
        ),
      if (kGlossaryFeatureEnabled)
        StatefulShellBranch(
          routes: [
            GoRoute(
              path: '/glossary',
              pageBuilder: (_, state) =>
                  _noTransitionPage(state, const GlossaryScreen()),
            ),
          ],
        ),
      if (kDictionaryFeatureEnabled)
        StatefulShellBranch(
          routes: [
            GoRoute(
              path: '/vocabulary',
              pageBuilder: (_, state) =>
                  _noTransitionPage(state, const VocabularyScreen()),
            ),
          ],
        ),
      StatefulShellBranch(
        routes: [
          ShellRoute(
            pageBuilder: (context, state, child) => _noTransitionPage(
              state,
              SettingsHostScreen(location: state.uri.path, child: child),
            ),
            routes: [
              GoRoute(
                path: '/settings/general',
                pageBuilder: (_, state) =>
                    _noTransitionPage(state, const GeneralSettingsScreen()),
              ),
              GoRoute(
                path: '/settings/services',
                pageBuilder: (_, state) =>
                    _noTransitionPage(state, const ServicesSettingsScreen()),
              ),
              GoRoute(
                path: '/settings/shortcuts',
                pageBuilder: (_, state) =>
                    _noTransitionPage(state, const ShortcutsSettingsScreen()),
              ),
              GoRoute(
                path: '/settings/providers',
                pageBuilder: (_, state) =>
                    _noTransitionPage(state, const ProvidersSettingsScreen()),
              ),
              GoRoute(
                path: '/settings/permissions',
                pageBuilder: (_, state) =>
                    _noTransitionPage(state, const PermissionsSettingsScreen()),
              ),
              GoRoute(
                path: '/settings/about',
                pageBuilder: (_, state) =>
                    _noTransitionPage(state, const AboutSettingsScreen()),
              ),
              if (kAdvancedSettingsFeatureEnabled)
                GoRoute(
                  path: '/settings/advanced',
                  pageBuilder: (_, state) =>
                      _noTransitionPage(state, const AdvancedSettingsScreen()),
                ),
              GoRoute(
                path: '/settings/updates',
                pageBuilder: (_, state) =>
                    _noTransitionPage(state, const UpdatesSettingsScreen()),
              ),
            ],
          ),
        ],
      ),
    ],
  ),
];

Page<void> _noTransitionPage(GoRouterState state, Widget child) {
  return NoTransitionPage<void>(key: state.pageKey, child: child);
}

class WorkbenchShell extends StatefulWidget {
  const WorkbenchShell({
    super.key,
    required this.location,
    required this.child,
  });

  final String location;
  final Widget child;

  @override
  State<WorkbenchShell> createState() => _WorkbenchShellState();
}

class _WorkbenchShellState extends State<WorkbenchShell> {
  WorkbenchDestinationId get _destination {
    final location = widget.location;
    if (location.startsWith('/settings')) {
      return WorkbenchDestinationId.settings;
    }
    if (location.startsWith('/history')) return WorkbenchDestinationId.history;
    if (location.startsWith('/glossary')) {
      return WorkbenchDestinationId.glossary;
    }
    if (location.startsWith('/vocabulary')) {
      return WorkbenchDestinationId.vocabulary;
    }
    return WorkbenchDestinationId.translate;
  }

  @override
  Widget build(BuildContext context) {
    final chrome = (kIsWindows || kIsLinux)
        ? WindowChromeKind.windows
        : WindowChromeKind.macos;
    return WorkbenchShellView(
      labels: workbenchShellLabels(),
      chrome: chrome,
      destination: widget.location == '/welcome'
          ? WorkbenchDestinationId.translate
          : _destination,
      onDestinationSelected: (destination) {
        context.go(switch (destination) {
          WorkbenchDestinationId.translate => '/translate',
          WorkbenchDestinationId.history => '/history',
          WorkbenchDestinationId.glossary => '/glossary',
          WorkbenchDestinationId.vocabulary => '/vocabulary',
          WorkbenchDestinationId.settings => '/settings/general',
        });
      },
      onMinimize: chrome == WindowChromeKind.windows
          ? () => workbenchWindowController.window.minimize()
          : null,
      onToggleMaximize: chrome == WindowChromeKind.windows
          ? () {
              final window = workbenchWindowController.window;
              if (window.isMaximized) {
                window.unmaximize();
              } else {
                window.maximize();
              }
            }
          : null,
      onClose: chrome == WindowChromeKind.windows ? hideWorkbenchWindow : null,
      onDragStart: chrome == WindowChromeKind.windows
          ? () => workbenchWindowController.window.startDragging()
          : null,
      child: widget.child,
    );
  }
}
