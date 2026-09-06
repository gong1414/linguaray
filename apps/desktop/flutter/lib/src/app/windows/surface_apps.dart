import 'package:flutter/foundation.dart' show kDebugMode;
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:linguaray_ui/linguaray_ui.dart' show LinguaRayMaterialTheme;

import '../../features/ocr/ocr_screen.dart';
import '../../i18n/i18n.dart';
import '../dependencies.dart';
import '../navigation/app_routes.dart';
import '../settings/settings_section.dart';
import 'app_windows.dart';

class SettingsApp extends StatefulWidget {
  const SettingsApp({super.key});

  @override
  State<SettingsApp> createState() => _SettingsAppState();
}

class _SettingsAppState extends State<SettingsApp> {
  late final GoRouter _router = createSettingsAppRouter(
    initialLocation: kDebugMode && debugInitialRoute.trim().isNotEmpty
        ? debugInitialRoute.trim()
        : null,
  );

  @override
  void initState() {
    super.initState();
    attachSettingsRouter(_router);
  }

  @override
  void dispose() {
    detachSettingsRouter(_router);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return _AppearanceThemedApp(
      builder: (context, themeMode) => MaterialApp.router(
        debugShowCheckedModeBanner: false,
        title: kSettingsWindowTitle,
        theme: LinguaRayMaterialTheme.light(),
        darkTheme: LinguaRayMaterialTheme.dark(),
        themeMode: themeMode,
        builder: (context, child) {
          final mac = Theme.of(context).platform == TargetPlatform.macOS;
          return CallbackShortcuts(
            bindings: {
              SingleActivator(
                LogicalKeyboardKey.keyW,
                meta: mac,
                control: !mac,
              ): hideSettingsWindow,
              const SingleActivator(LogicalKeyboardKey.escape):
                  hideSettingsWindow,
            },
            child: child!,
          );
        },
        routerConfig: _router,
        localizationsDelegates: context.localizationDelegates,
        supportedLocales: context.supportedLocales,
        locale: context.locale,
      ),
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
  Widget build(BuildContext context) {
    return _AppearanceThemedApp(
      builder: (context, themeMode) => MaterialApp.router(
        debugShowCheckedModeBanner: false,
        title: kMiniTranslatorWindowTitle,
        theme: LinguaRayMaterialTheme.light(),
        darkTheme: LinguaRayMaterialTheme.dark(),
        themeMode: themeMode,
        routerConfig: _router,
        localizationsDelegates: context.localizationDelegates,
        supportedLocales: context.supportedLocales,
        locale: context.locale,
      ),
    );
  }
}

class OcrApp extends StatelessWidget {
  const OcrApp({super.key});

  @override
  Widget build(BuildContext context) {
    return _AppearanceThemedApp(
      builder: (context, themeMode) => MaterialApp(
        debugShowCheckedModeBanner: false,
        title: kOcrWindowTitle,
        theme: LinguaRayMaterialTheme.light(),
        darkTheme: LinguaRayMaterialTheme.dark(),
        themeMode: themeMode,
        localizationsDelegates: context.localizationDelegates,
        supportedLocales: context.supportedLocales,
        locale: context.locale,
        home: const OcrScreen(),
      ),
    );
  }
}

class _AppearanceThemedApp extends ConsumerWidget {
  const _AppearanceThemedApp({required this.builder});

  final Widget Function(BuildContext context, ThemeMode themeMode) builder;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final store = ref.watch(settingsStoreProvider);
    return ListenableBuilder(
      listenable: store.listenableFor(SettingsSection.appearance),
      builder: (context, _) => builder(context, store.themeMode),
    );
  }
}
