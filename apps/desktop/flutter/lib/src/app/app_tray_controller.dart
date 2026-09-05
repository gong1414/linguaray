import 'dart:async';
import 'dart:io';

import 'package:flutter/foundation.dart' show kDebugMode;
import 'package:linguaray_application/linguaray_application.dart'
    show UpdateState, UpdateStatus;
import 'package:nativeapi/nativeapi.dart';

import '../i18n/i18n.dart';
import '../platform/platform_types.dart';
import '../platform/shortcuts/menu_accelerator.dart';
import '../platform/windows/dock_icon_controller.dart';
import 'commands/trigger_controller.dart';
import 'env.dart';
import 'runtime.dart' show runtimeDataDirectory;
import 'settings/settings_store.dart';
import 'windows/app_windows.dart';

final class AppTrayController {
  AppTrayController({required this._readUpdate, SettingsStore? store})
    : _store = store ?? settingsStore;
  final UpdateState Function() _readUpdate;
  final SettingsStore _store;
  late final TrayIcon _trayIcon;

  void initialize({required bool visible}) {
    _trayIcon = TrayIcon();
    final icon = Image.fromAsset('resources/images/tray_icon.png');
    if (icon != null) _trayIcon.icon = icon;
    _trayIcon.isVisible = visible;
    dockIconController.setTrayIconVisible(visible);
    rebuildMenu();
    _trayIcon.contextMenuTrigger = ContextMenuTrigger.none;
    _trayIcon.on<TrayIconClickedEvent>((_) => _openMenuAfterEvent());
    _trayIcon.on<TrayIconRightClickedEvent>((_) => _openMenuAfterEvent());
  }

  void setVisible(bool visible) {
    _trayIcon.isVisible = visible;
    dockIconController.setTrayIconVisible(visible);
  }

  void rebuildMenu() {
    _trayIcon.contextMenu = _buildContextMenu();
  }

  void dispose() {
    _trayIcon.dispose();
  }

  void _openMenuAfterEvent() {
    scheduleMicrotask(_trayIcon.openContextMenu);
  }

  Menu _buildContextMenu() {
    final menu = Menu();
    final labels = t.app.tray.context_menu;
    final shortcuts = _store.shortcuts;

    _addAction(
      menu,
      labels.selection_translation,
      () => triggerController.trigger(TriggerAction.translateSelection),
      shortcut: shortcuts.extractTextFromScreenSelection,
    );
    _addAction(
      menu,
      labels.capture_translation,
      () => triggerController.trigger(TriggerAction.captureAndTranslate),
      shortcut: shortcuts.extractTextFromScreenCapture,
    );
    _addAction(
      menu,
      labels.input_translation,
      () => triggerController.openInputWindow(trayBounds: _trayIcon.bounds),
      shortcut: shortcuts.translateInputContent,
    );
    _addAction(
      menu,
      labels.clipboard_translation,
      () => triggerController.trigger(TriggerAction.translateInput),
      shortcut: shortcuts.extractTextFromClipboard,
    );
    _addAction(
      menu,
      labels.show_translation_window,
      () =>
          triggerController.showTranslationWindow(trayBounds: _trayIcon.bounds),
      shortcut: shortcuts.toggleMiniTranslator,
    );

    menu.addSeparator();
    _addAction(
      menu,
      labels.capture_ocr,
      () => triggerController.trigger(TriggerAction.captureOcr),
      shortcut: shortcuts.captureOcr,
    );
    _addAction(
      menu,
      labels.silent_capture_ocr,
      () => triggerController.trigger(TriggerAction.silentCaptureOcr),
      shortcut: shortcuts.silentCaptureOcr,
    );
    _addAction(
      menu,
      labels.file_ocr,
      () => triggerController.trigger(TriggerAction.fileOcr),
      shortcut: shortcuts.fileOcr,
    );
    _addAction(
      menu,
      labels.clipboard_ocr,
      () => triggerController.trigger(TriggerAction.clipboardOcr),
      shortcut: shortcuts.clipboardOcr,
    );
    _addAction(
      menu,
      labels.show_ocr_window,
      () => triggerController.trigger(TriggerAction.showOcrWindow),
      shortcut: shortcuts.showOcrWindow,
    );

    menu.addSeparator();
    final update = _readUpdate();
    if (update.status == UpdateStatus.available) {
      _addAction(
        menu,
        t.ui.updates.available(version: update.manifest?.version ?? ''),
        () async => showSettingsWindow(
          destination: SettingsDestination.settingsUpdates,
        ),
      );
    }
    _addAction(
      menu,
      labels.preferences,
      () async => showSettingsWindow(),
      shortcut: Platform.isMacOS ? 'Command+,' : 'Ctrl+,',
    );
    _addAction(
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
      )..submenu = devToolsSubmenu;
      menu.addItem(devToolsItem);
    }

    menu.addSeparator();
    final quitItem = MenuItem(labels.quit)
      ..on<MenuItemClickedEvent>((_) => exit(0));
    setNativeMenuAccelerator(
      quitItem,
      Platform.isMacOS ? 'Command+Q' : 'Alt+F4',
    );
    menu.addItem(quitItem);
    return menu;
  }

  void _addAction(
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
}
