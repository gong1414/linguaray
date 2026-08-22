import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../features.dart';
import '../../i18n/i18n.dart';
import '../../services/app_windows.dart'
    show hideWorkbenchWindow, workbenchWindowController;
import '../../utils/platform_util.dart';
import '../../utils/utils.dart';
import '../../widgets/navigation_item.dart';
import '../../widgets/ui.dart'
    show DesignThemeContext, DesignTypographyStyles, SidebarCard, SidebarGroup;
import '../../widgets/workbench.dart';
import '../settings/about.dart';
import '../settings/advanced.dart';
import '../settings/general.dart';
import '../settings/index.dart';
import '../settings/providers.dart';
import '../settings/services.dart';
import '../settings/shortcuts.dart';
import 'glossary.dart';
import 'library.dart';
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
                      _noTransitionPage(state, const WorkbenchLibraryPage()),
                ),
              ],
            ),
          if (kGlossaryFeatureEnabled)
            StatefulShellBranch(
              routes: [
                GoRoute(
                  path: '/glossary',
                  pageBuilder: (_, state) =>
                      _noTransitionPage(state, const WorkbenchGlossaryPage()),
                ),
              ],
            ),
          StatefulShellBranch(
            routes: [
              ShellRoute(
                pageBuilder: (context, state, child) => _noTransitionPage(
                  state,
                  SettingsTabsShell(location: state.uri.path, child: child),
                ),
                routes: [
                  GoRoute(
                    path: '/settings/general',
                    pageBuilder: (_, state) =>
                        _noTransitionPage(state, const GeneralSettingsPage()),
                  ),
                  GoRoute(
                    path: '/settings/services',
                    pageBuilder: (_, state) =>
                        _noTransitionPage(state, const ServicesSettingsPage()),
                  ),
                  GoRoute(
                    path: '/settings/shortcuts',
                    pageBuilder: (_, state) =>
                        _noTransitionPage(state, const ShortcutsSettingsPage()),
                  ),
                  GoRoute(
                    path: '/settings/providers',
                    pageBuilder: (_, state) =>
                        _noTransitionPage(state, const ProvidersSettingsPage()),
                  ),
                  if (kAdvancedSettingsFeatureEnabled)
                    GoRoute(
                      path: '/settings/advanced',
                      pageBuilder: (_, state) => _noTransitionPage(
                        state,
                        const AdvancedSettingsPage(),
                      ),
                    ),
                  GoRoute(
                    path: '/settings/about',
                    pageBuilder: (_, state) =>
                        _noTransitionPage(state, const AboutSettingsPage()),
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
  bool _collapsed = false;

  /// Kept here rather than in the sidebar: collapsing unmounts that column, so
  /// a width held inside it would go back to the token on every re-open.
  double? _sidebarWidth;

  bool _selected(String path) =>
      widget.location == path || widget.location.startsWith('$path/');

  /// The shell draws its own window buttons on Windows and Linux, so they get
  /// the real verbs. Close hides rather than destroys — the app lives on in
  /// the tray, the same answer the window delegate gives the native close.
  WorkbenchWindowActions? get _windowActions {
    if (!kIsWindows && !kIsLinux) return null;
    return WorkbenchWindowActions(
      onMinimize: () => workbenchWindowController.window.minimize(),
      onToggleMaximize: () {
        final window = workbenchWindowController.window;
        if (window.isMaximized) {
          window.unmaximize();
        } else {
          window.maximize();
        }
      },
      onClose: hideWorkbenchWindow,
      onDragStart: () => workbenchWindowController.window.startDragging(),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Workbench(
        collapsed: _collapsed,
        onToggleCollapsed: () => setState(() => _collapsed = !_collapsed),
        sidebarWidth: _sidebarWidth,
        onSidebarWidthChange: (width) => setState(() => _sidebarWidth = width),
        windowActions: _windowActions,
        sidebarFooter:
            widget.location == '/welcome' ? null : const _SidebarVersion(),
        sidebar: widget.location == '/welcome'
            ? const []
            : [
                SidebarGroup(
                  first: true,
                  label: Text(t.workbench.workspace),
                  children: [
                    NavigationItem(
                      label: t.workbench.translate,
                      icon: FluentIcons.translate_20_regular,
                      selected: _selected('/translate'),
                      onTap: () => context.go('/translate'),
                    ),
                    if (kGlossaryFeatureEnabled)
                      NavigationItem(
                        label: t.workbench.glossary,
                        icon: FluentIcons.book_20_regular,
                        selected: _selected('/glossary'),
                        onTap: () => context.go('/glossary'),
                      ),
                    if (kHistoryFeatureEnabled)
                      NavigationItem(
                        label: t.workbench.history,
                        icon: FluentIcons.history_20_regular,
                        selected: _selected('/history'),
                        onTap: () => context.go('/history'),
                      ),
                    NavigationItem(
                      label: t.settings.layout.title,
                      icon: FluentIcons.settings_20_regular,
                      selected: _selected('/settings'),
                      onTap: () => context.go('/settings/general'),
                    ),
                  ],
                ),
              ],
        child: widget.child,
      ),
    );
  }
}

/// A passive version marker. Automatic updates are intentionally outside the
/// first-release scope, so this never pretends to run an updater.
class _SidebarVersion extends StatelessWidget {
  const _SidebarVersion();

  @override
  Widget build(BuildContext context) {
    final tokens = context.tokens;
    final colors = tokens.colors;

    return SidebarCard(
      gap: 6,
      children: [
        Text(
          'LinguaRay · ${sharedEnv.appVersion}',
          style: tokens.typography.sansStyle(
            fontSize: 12,
            fontWeight: FontWeight.w600,
            height: 1,
            color: colors.fg,
          ),
        ),
      ],
    );
  }
}
