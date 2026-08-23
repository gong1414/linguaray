import 'dart:async';
import 'dart:io';

import 'package:flutter/foundation.dart' show kDebugMode;
import 'package:flutter/material.dart' hide Image;
import 'package:go_router/go_router.dart';
import 'package:linguaray_ui/linguaray_ui.dart' show LinguaRayMaterialTheme;
import 'package:nativeapi/nativeapi.dart';

import '../i18n/i18n.dart';
import '../platform/menu_accelerator.dart';
import '../platform/permission_controller.dart';
import '../platform/platform_types.dart';
import '../platform/protocol_controller.dart';
import '../platform/trigger_controller.dart';
import '../services/app_windows.dart';
import '../services/dock_icon_controller.dart';
import '../services/mac_app_presentation.dart';
import '../services/runtime.dart' show runtimeDataDirectory;
import '../services/settings_store.dart';
import '../services/shortcut_service/shortcut_service.dart';
import '../ui/quick_translate/quick_translate_screen.dart';
import '../utils/env.dart';
import '../utils/language_util.dart';
import '../widgets/toast_host.dart';
import '__root.dart';
import 'debug/runtime.dart' as debug_runtime_route;
import 'settings/index.dart' as settings_route;

const _debugInitialRoute = String.fromEnvironment('LINGUARAY_INITIAL_ROUTE');

SettingsDestination? get _debugInitialDestination {
  if (!kDebugMode || _debugInitialRoute.trim().isEmpty) return null;
  final route = _debugInitialRoute.trim();
  for (final destination in SettingsDestination.values) {
    if (destination.location == route) return destination;
  }
  return null;
}

// ──────────────────────────────────────────────────────────────────────────────
// Routers
// ──────────────────────────────────────────────────────────────────────────────

/// Assembles the main application's route graph from modular route files.
///
/// Modular route organization:
/// - each route lives in its own module/file
/// - this file is the composition root for router setup
GoRouter createSettingsAppRouter({String? initialLocation}) {
  return GoRouter(
    routes: <RouteBase>[
      ...$appRoutes,
      ...debug_runtime_route.$appRoutes,
      ...settings_route.$appRoutes,
    ],
    initialLocation: initialLocation ?? pendingSettingsLocation,
    debugLogDiagnostics: false,
  );
}

/// Assembles the mini-translator window's route graph.
GoRouter createMiniTranslatorAppRouter() {
  return GoRouter(
    routes: [
      GoRoute(
        path: '/',
        builder: (context, state) => const QuickTranslateScreen(),
      ),
    ],
    debugLogDiagnostics: false,
  );
}

// ──────────────────────────────────────────────────────────────────────────────
// App widgets
// ──────────────────────────────────────────────────────────────────────────────

class SettingsApp extends StatefulWidget {
  const SettingsApp({super.key});

  @override
  State<SettingsApp> createState() => _SettingsAppState();
}

class _SettingsAppState extends State<SettingsApp> {
  late final GoRouter _router = createSettingsAppRouter(
    initialLocation: kDebugMode && _debugInitialRoute.trim().isNotEmpty
        ? _debugInitialRoute.trim()
        : null,
  );

  @override
  void initState() {
    super.initState();
    attachSettingsRouter(_router);
    settingsStore.addListener(_onSettingsChanged);
  }

  @override
  void dispose() {
    detachSettingsRouter(_router);
    settingsStore.removeListener(_onSettingsChanged);
    super.dispose();
  }

  void _onSettingsChanged() {
    setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp.router(
      debugShowCheckedModeBanner: false,
      title: kSettingsWindowTitle,
      theme: LinguaRayMaterialTheme.light(),
      darkTheme: LinguaRayMaterialTheme.dark(),
      themeMode: settingsStore.themeMode,
      builder: (context, child) => ToastHost(child: child!),
      routerConfig: _router,
      localizationsDelegates: context.localizationDelegates,
      supportedLocales: context.supportedLocales,
      locale: context.locale,
    );
  }
}

class MiniTranslatorApp extends StatefulWidget {
  const MiniTranslatorApp({super.key});

  @override
  State<MiniTranslatorApp> createState() => _MiniTranslatorAppState();
}

class _MiniTranslatorAppState extends State<MiniTranslatorApp> {
  late final GoRouter _router = createMiniTranslatorAppRouter();

  @override
  void initState() {
    super.initState();
    settingsStore.addListener(_onSettingsChanged);
  }

  @override
  void dispose() {
    settingsStore.removeListener(_onSettingsChanged);
    super.dispose();
  }

  void _onSettingsChanged() {
    setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp.router(
      debugShowCheckedModeBanner: false,
      title: kMiniTranslatorWindowTitle,
      theme: LinguaRayMaterialTheme.light(),
      darkTheme: LinguaRayMaterialTheme.dark(),
      themeMode: settingsStore.themeMode,
      builder: (context, child) => ToastHost(child: child!),
      routerConfig: _router,
      localizationsDelegates: context.localizationDelegates,
      supportedLocales: context.supportedLocales,
      locale: context.locale,
    );
  }
}

class RootView extends StatelessWidget {
  const RootView({super.key});

  @override
  Widget build(BuildContext context) {
    // The scope sits above the window manager so one language switch reaches
    // every window — both settings and the mini translator hang off it.
    return TranslationProvider(
      child: const LocaleRebuildScope(child: _RootBodyView()),
    );
  }
}

class _RootBodyView extends StatefulWidget {
  const _RootBodyView();

  @override
  State<_RootBodyView> createState() => _RootBodyViewState();
}

class _RootBodyViewState extends State<_RootBodyView>
    with WidgetsBindingObserver {
  late final TrayIcon _trayIcon;
  late bool _showInMenuBar;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    _showInMenuBar = settingsStore.general.showInMenuBar;
    settingsStore.addListener(_handleChanged);
    _setupTrayIcon();
    MacAppPresentation.setHandlers(
      onReopen: showSettingsWindow,
      onOpenSettings: showSettingsWindow,
    );
    unawaited(
      ShortcutService.instance.start(onAction: triggerController.trigger),
    );
    protocolController.onTranslate = (text) {
      triggerController.quickWindowRequest.value = QuickWindowRequest(
        text: text,
        submit: true,
        clearExisting: true,
      );
      unawaited(
        showMiniTranslatorWindow(position: miniTranslatorPositionNearCursor()),
      );
    };
    protocolController.onOpenSettings = showSettingsWindow;
    protocolController.start();
    unawaited(permissionController.refresh());
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_debugInitialDestination == null) {
        initializeResidentApp();
      } else {
        showSettingsWindow(destination: _debugInitialDestination);
      }
    });
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    settingsStore.removeListener(_handleChanged);
    unawaited(ShortcutService.instance.stop());
    _trayIcon.dispose();
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      unawaited(permissionController.refresh());
    }
  }

  Future<void> _handleChanged() async {
    // Handle language change
    final oldLocale = context.locale;
    final newLocale = languageToLocale(settingsStore.appLanguage);
    if (newLocale != oldLocale) {
      await context.setLocale(newLocale);
    }

    // Handle show in menu bar toggle
    final newShowInMenuBar = settingsStore.general.showInMenuBar;
    if (newShowInMenuBar != _showInMenuBar) {
      _showInMenuBar = newShowInMenuBar;
      _trayIcon.isVisible = newShowInMenuBar;
      // Dropping the tray icon would leave the app with no visible entry
      // point, so the Dock icon takes over.
      dockIconController.setTrayIconVisible(newShowInMenuBar);
    }

    // Rebuild so shortcut edits immediately update the native accelerator
    // column as well as the registered global shortcuts.
    _trayIcon.contextMenu = _buildContextMenu();
  }

  // ────────────────────────────────────────────────────────────────────────────
  // Tray icon
  // ────────────────────────────────────────────────────────────────────────────

  void _setupTrayIcon() {
    _trayIcon = TrayIcon();
    final icon = Image.fromAsset('resources/images/tray_icon.png');
    if (icon != null) _trayIcon.icon = icon;
    _trayIcon.isVisible = _showInMenuBar;
    dockIconController.setTrayIconVisible(_showInMenuBar);
    _trayIcon.contextMenu = _buildContextMenu();
    _trayIcon.contextMenuTrigger = ContextMenuTrigger.none;
    _trayIcon.on<TrayIconClickedEvent>((_) => _openTrayMenuAfterEvent());
    _trayIcon.on<TrayIconRightClickedEvent>((_) => _openTrayMenuAfterEvent());
  }

  void _openTrayMenuAfterEvent() {
    scheduleMicrotask(_trayIcon.openContextMenu);
  }

  Menu _buildContextMenu() {
    final menu = Menu();
    final labels = t.app.tray.context_menu;
    final shortcuts = settingsStore.shortcuts;

    _addTrayAction(
      menu,
      labels.selection_translation,
      () => triggerController.trigger(TriggerAction.translateSelection),
      shortcut: shortcuts.extractTextFromScreenSelection,
    );
    _addTrayAction(
      menu,
      labels.capture_translation,
      () => triggerController.trigger(TriggerAction.captureAndTranslate),
      shortcut: shortcuts.extractTextFromScreenCapture,
    );
    _addTrayAction(
      menu,
      labels.input_translation,
      () => triggerController.openInputWindow(trayBounds: _trayIcon.bounds),
      shortcut: shortcuts.translateInputContent,
    );
    _addTrayAction(
      menu,
      labels.clipboard_translation,
      () => triggerController.trigger(TriggerAction.translateInput),
      shortcut: shortcuts.extractTextFromClipboard,
    );
    _addTrayAction(
      menu,
      labels.show_translation_window,
      () =>
          triggerController.showTranslationWindow(trayBounds: _trayIcon.bounds),
      shortcut: shortcuts.toggleMiniTranslator,
    );

    menu.addSeparator();
    _addTrayAction(
      menu,
      labels.capture_ocr,
      () => triggerController.trigger(TriggerAction.captureOcr),
      shortcut: shortcuts.captureOcr,
    );

    menu.addSeparator();
    _addTrayAction(
      menu,
      labels.preferences,
      () async => showSettingsWindow(),
      shortcut: Platform.isMacOS ? 'Command+,' : 'Ctrl+,',
    );
    _addTrayAction(
      menu,
      labels.about,
      () async =>
          showSettingsWindow(destination: SettingsDestination.settingsAbout),
    );
    menu.addItem(
      MenuItem(
        'LinguaRay ${Env.instance.appVersion} '
        '[${Env.instance.appBuildNumber}]',
      )..enabled = false,
    );

    if (kDebugMode) {
      final devToolsSubmenu = Menu();
      devToolsSubmenu.addItem(
        MenuItem(labels.dev_tools.open_data_directory)
          ..on<MenuItemClickedEvent>((_) async {
            UrlOpener.instance.open('file://${runtimeDataDirectory.path}');
          }),
      );

      final devToolsItem = MenuItem(
        labels.dev_tools.title,
        MenuItemType.submenu,
      );
      devToolsItem.submenu = devToolsSubmenu;
      menu.addItem(devToolsItem);
    }

    menu.addSeparator();
    final quitItem = MenuItem(labels.quit)
      ..on<MenuItemClickedEvent>((_) {
        exit(0);
      });
    setNativeMenuAccelerator(
      quitItem,
      Platform.isMacOS ? 'Command+Q' : 'Alt+F4',
    );
    menu.addItem(quitItem);

    return menu;
  }

  void _addTrayAction(
    Menu menu,
    String label,
    Future<void> Function() action, {
    String? shortcut,
  }) {
    final item = MenuItem(label)
      ..on<MenuItemClickedEvent>((_) => unawaited(action()));
    if (shortcut != null && shortcut.trim().isNotEmpty) {
      setNativeMenuAccelerator(item, shortcut);
    }
    menu.addItem(item);
  }

  @override
  Widget build(BuildContext context) {
    return ValueListenableBuilder<AppSurface>(
      valueListenable: appSurface,
      builder: (context, surface, _) => switch (surface) {
        AppSurface.settings => const SettingsApp(
          key: ValueKey(AppSurface.settings),
        ),
        AppSurface.miniTranslator => const MiniTranslatorApp(
          key: ValueKey(AppSurface.miniTranslator),
        ),
      },
    );
  }
}
