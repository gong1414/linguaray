import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart'
    show UpdateState;

import '../features/updates/update_coordinator.dart';
import '../i18n/i18n.dart';
import '../platform/permissions/permission_controller.dart';
import '../platform/platform_util.dart';
import '../platform/protocol/protocol_controller.dart';
import '../platform/shortcuts/shortcut_service.dart';
import '../platform/windows/mac_app_presentation.dart';
import '../shared/language_util.dart';
import 'app_tray_controller.dart';
import 'commands/external_action_controller.dart';
import 'commands/trigger_controller.dart';
import 'navigation/app_routes.dart';
import 'settings/settings_section.dart';
import 'settings/settings_store.dart';
import 'updates/automatic_update_schedule.dart';
import 'windows/app_windows.dart';
import 'windows/surface_apps.dart';

class RootView extends StatelessWidget {
  const RootView({super.key});

  @override
  Widget build(BuildContext context) {
    return TranslationProvider(
      child: const LocaleRebuildScope(child: _RootBodyView()),
    );
  }
}

class _RootBodyView extends ConsumerStatefulWidget {
  const _RootBodyView();

  @override
  ConsumerState<_RootBodyView> createState() => _RootBodyViewState();
}

class _RootBodyViewState extends ConsumerState<_RootBodyView>
    with WidgetsBindingObserver {
  late final AppTrayController _tray;
  late final ProviderSubscription<UpdateState> _updateSubscription;
  late bool _showInMenuBar;
  late final AutomaticUpdateSchedule _updates;
  late final Listenable _hostSettings;

  @override
  void initState() {
    super.initState();
    _updates = AutomaticUpdateSchedule(
      enabled: () => settingsStore.advanced.checkUpdatesOnLaunch,
      runCheck: () => ref.read(updateCoordinatorProvider.notifier).check(),
    );
    WidgetsBinding.instance.addObserver(this);
    _showInMenuBar = _effectiveTrayVisibility;
    _hostSettings = settingsStore.listenablesFor(const [
      SettingsSection.general,
      SettingsSection.appearance,
      SettingsSection.shortcuts,
      SettingsSection.advanced,
    ]);
    _hostSettings.addListener(_handleChanged);
    _tray = AppTrayController(
      readUpdate: () => ref.read(updateCoordinatorProvider),
    );
    _tray.initialize(visible: _showInMenuBar);
    _updateSubscription = ref.listenManual(
      updateCoordinatorProvider,
      (_, _) => _tray.rebuildMenu(),
    );
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
        _updates.start();
      } else {
        showSettingsWindow(destination: debugInitialDestination);
      }
    });
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    _hostSettings.removeListener(_handleChanged);
    unawaited(ShortcutService.instance.stop());
    _updates.dispose();
    _updateSubscription.close();
    _tray.dispose();
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      unawaited(permissionController.refresh());
      unawaited(_updates.check());
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
    unawaited(_updates.check());
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
