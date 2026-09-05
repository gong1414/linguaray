import 'package:flutter/foundation.dart' show kDebugMode;
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:go_router/go_router.dart';
import 'package:linguaray_ui/linguaray_ui.dart' show LinguaRayMaterialTheme;

import '../i18n/i18n.dart';
import '../services/app_windows.dart';
import '../services/settings_store.dart';
import '../ui/ocr/ocr_screen.dart';
import '../widgets/toast_host.dart';
import 'app_routes.dart';

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
    settingsStore.addListener(_onSettingsChanged);
  }

  @override
  void dispose() {
    detachSettingsRouter(_router);
    settingsStore.removeListener(_onSettingsChanged);
    super.dispose();
  }

  void _onSettingsChanged() => setState(() {});

  @override
  Widget build(BuildContext context) {
    return MaterialApp.router(
      debugShowCheckedModeBanner: false,
      title: kSettingsWindowTitle,
      theme: LinguaRayMaterialTheme.light(),
      darkTheme: LinguaRayMaterialTheme.dark(),
      themeMode: settingsStore.themeMode,
      builder: (context, child) {
        final mac = Theme.of(context).platform == TargetPlatform.macOS;
        return CallbackShortcuts(
          bindings: {
            SingleActivator(LogicalKeyboardKey.keyW, meta: mac, control: !mac):
                hideSettingsWindow,
            const SingleActivator(LogicalKeyboardKey.escape):
                hideSettingsWindow,
          },
          child: ToastHost(child: child!),
        );
      },
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

  void _onSettingsChanged() => setState(() {});

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

class OcrApp extends StatefulWidget {
  const OcrApp({super.key});

  @override
  State<OcrApp> createState() => _OcrAppState();
}

class _OcrAppState extends State<OcrApp> {
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

  void _onSettingsChanged() => setState(() {});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      title: kOcrWindowTitle,
      theme: LinguaRayMaterialTheme.light(),
      darkTheme: LinguaRayMaterialTheme.dark(),
      themeMode: settingsStore.themeMode,
      localizationsDelegates: context.localizationDelegates,
      supportedLocales: context.supportedLocales,
      locale: context.locale,
      home: const OcrScreen(),
    );
  }
}
