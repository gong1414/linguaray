import 'dart:async';

import 'package:flutter/material.dart';

import '../i18n/i18n.dart';
import '../platform/external_action_controller.dart';
import '../platform/permission_controller.dart';
import '../platform/protocol_controller.dart';
import '../platform/startup_update_controller.dart';
import '../platform/trigger_controller.dart';
import '../services/app_windows.dart';
import '../services/mac_app_presentation.dart';
import '../services/settings_store.dart';
import '../services/shortcut_service/shortcut_service.dart';
import '../utils/language_util.dart';
import '../utils/platform_util.dart';
import 'app_routes.dart';
import 'app_tray_controller.dart';
import 'surface_apps.dart';

class RootView extends StatelessWidget {
  const RootView({super.key});

  @override
  Widget build(BuildContext context) {
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
  final AppTrayController _tray = AppTrayController();
  late bool _showInMenuBar;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    _showInMenuBar = _effectiveTrayVisibility;
    settingsStore.addListener(_handleChanged);
    _tray.initialize(visible: _showInMenuBar);
    MacAppPresentation.setHandlers(
      onReopen: showSettingsWindow,
      onOpenSettings: showSettingsWindow,
    );
    unawaited(
      ShortcutService.instance.start(onAction: triggerController.trigger),
    );
    protocolController.onCommand = externalActionController.dispatchProtocol;
    protocolController.start();
    externalActionController.start();
    unawaited(permissionController.refresh());
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (debugInitialDestination == null) {
        initializeResidentApp();
        startupUpdateController.start();
      } else {
        showSettingsWindow(destination: debugInitialDestination);
      }
    });
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    settingsStore.removeListener(_handleChanged);
    unawaited(ShortcutService.instance.stop());
    startupUpdateController.stop();
    _tray.dispose();
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      unawaited(permissionController.refresh());
      unawaited(startupUpdateController.check());
    }
  }

  Future<void> _handleChanged() async {
    final newLocale = languageToLocale(settingsStore.appLanguage);
    if (newLocale != context.locale) await context.setLocale(newLocale);

    final visible = _effectiveTrayVisibility;
    if (visible != _showInMenuBar) {
      _showInMenuBar = visible;
      _tray.setVisible(visible);
    }
    unawaited(startupUpdateController.check());
    _tray.rebuildMenu();
  }

  bool get _effectiveTrayVisibility =>
      kIsWindows || settingsStore.general.showInMenuBar;

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
        AppSurface.ocr => const OcrApp(key: ValueKey(AppSurface.ocr)),
      },
    );
  }
}
