import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:hotkey_manager/hotkey_manager.dart';

import '../../app/settings/settings_store.dart';
import '../platform_types.dart';

typedef TriggerActionHandler = Future<void> Function(TriggerAction action);

class ShortcutService extends ChangeNotifier {
  ShortcutService._();

  static final ShortcutService instance = ShortcutService._();

  TriggerActionHandler? _handler;
  bool _started = false;
  bool _registrationSuspended = false;
  int _registrationGeneration = 0;
  List<ShortcutBinding> _bindings = const [];

  List<ShortcutBinding> get bindings => List.unmodifiable(_bindings);

  Future<void> start({required TriggerActionHandler onAction}) async {
    _handler = onAction;
    if (!_started) {
      _started = true;
      settingsStore.addListener(_settingsChanged);
    }
    await _registerCurrentSettings();
  }

  Future<void> stop() async {
    if (!_started) return;
    _started = false;
    _handler = null;
    _registrationSuspended = false;
    _registrationGeneration++;
    settingsStore.removeListener(_settingsChanged);
    await hotKeyManager.unregisterAll();
    _bindings = const [];
    notifyListeners();
  }

  void _settingsChanged() {
    if (_started && !_registrationSuspended) {
      unawaited(_registerCurrentSettings());
    }
  }

  /// Releases global bindings while the settings UI records a replacement.
  /// Otherwise the operating system consumes an existing shortcut before the
  /// focused Flutter recorder can observe it.
  Future<void> suspendRegistration() async {
    if (!_started || _registrationSuspended) return;
    _registrationSuspended = true;
    _registrationGeneration++;
    await hotKeyManager.unregisterAll();
  }

  Future<void> resumeRegistration() async {
    if (!_registrationSuspended) return;
    _registrationSuspended = false;
    if (_started) await _registerCurrentSettings();
  }

  Future<void> _registerCurrentSettings() async {
    final generation = ++_registrationGeneration;
    await hotKeyManager.unregisterAll();
    if (!_started ||
        _registrationSuspended ||
        generation != _registrationGeneration) {
      return;
    }

    final shortcuts = settingsStore.shortcuts;
    final requested = <TriggerAction, String>{
      TriggerAction.toggleQuickWindow: shortcuts.toggleMiniTranslator,
      TriggerAction.translateSelection:
          shortcuts.extractTextFromScreenSelection,
      TriggerAction.captureAndTranslate: shortcuts.extractTextFromScreenCapture,
      TriggerAction.captureOcr: shortcuts.captureOcr,
      TriggerAction.silentCaptureOcr: shortcuts.silentCaptureOcr,
      TriggerAction.fileOcr: shortcuts.fileOcr,
      TriggerAction.clipboardOcr: shortcuts.clipboardOcr,
      TriggerAction.showOcrWindow: shortcuts.showOcrWindow,
      TriggerAction.translateInput: shortcuts.extractTextFromClipboard,
      TriggerAction.openInputWindow: shortcuts.translateInputContent,
    };
    final duplicateCounts = <String, int>{};
    for (final accelerator in requested.values) {
      final canonical = _canonical(accelerator);
      if (canonical.isNotEmpty) {
        duplicateCounts[canonical] = (duplicateCounts[canonical] ?? 0) + 1;
      }
    }

    final results = <ShortcutBinding>[];
    for (final entry in requested.entries) {
      if (!_started ||
          _registrationSuspended ||
          generation != _registrationGeneration) {
        return;
      }
      final accelerator = entry.value.trim();
      if (accelerator.isEmpty) {
        results.add(
          ShortcutBinding(
            action: entry.key,
            accelerator: accelerator,
            state: ShortcutRegistrationState.unregistered,
          ),
        );
        continue;
      }
      if ((duplicateCounts[_canonical(accelerator)] ?? 0) > 1) {
        results.add(
          ShortcutBinding(
            action: entry.key,
            accelerator: accelerator,
            state: ShortcutRegistrationState.conflict,
            conflictReason: 'The shortcut is assigned to more than one action.',
          ),
        );
        continue;
      }

      final hotKey = parseStoredShortcut(accelerator);
      if (hotKey == null) {
        results.add(
          ShortcutBinding(
            action: entry.key,
            accelerator: accelerator,
            state: ShortcutRegistrationState.invalid,
            conflictReason: 'The shortcut cannot be parsed.',
          ),
        );
        continue;
      }

      try {
        await hotKeyManager.register(
          hotKey,
          keyDownHandler: (_) {
            final handler = _handler;
            if (handler != null) unawaited(handler(entry.key));
          },
        );
        results.add(
          ShortcutBinding(
            action: entry.key,
            accelerator: accelerator,
            state: ShortcutRegistrationState.registered,
          ),
        );
      } catch (error) {
        results.add(
          ShortcutBinding(
            action: entry.key,
            accelerator: accelerator,
            state: ShortcutRegistrationState.conflict,
            conflictReason: _registrationError(error),
          ),
        );
      }
    }

    if (!_started ||
        _registrationSuspended ||
        generation != _registrationGeneration) {
      return;
    }
    _bindings = results;
    notifyListeners();
  }

  String _registrationError(Object error) {
    if (error is PlatformException &&
        error.message?.trim().isNotEmpty == true) {
      return error.message!.trim();
    }
    return 'The operating system rejected this shortcut.';
  }
}

String _canonical(String stored) => stored
    .split('+')
    .map((part) => part.trim().toLowerCase())
    .where((part) => part.isNotEmpty)
    .join('+');

@visibleForTesting
HotKey? parseStoredShortcut(String stored) {
  final parts = stored
      .split('+')
      .map((part) => part.trim())
      .where((part) => part.isNotEmpty)
      .toList();
  if (parts.isEmpty) return null;

  final modifiers = <HotKeyModifier>[];
  PhysicalKeyboardKey? key;
  for (final part in parts) {
    switch (part.toLowerCase()) {
      case 'option':
      case 'alt':
        modifiers.add(HotKeyModifier.alt);
        break;
      case 'control':
      case 'ctrl':
        modifiers.add(HotKeyModifier.control);
        break;
      case 'shift':
        modifiers.add(HotKeyModifier.shift);
        break;
      case 'command':
      case 'cmd':
      case 'meta':
        modifiers.add(HotKeyModifier.meta);
        break;
      default:
        if (key != null) return null;
        key = _physicalKey(part);
    }
  }
  if (key == null || modifiers.isEmpty) return null;
  return HotKey(
    key: key,
    modifiers: modifiers.toSet().toList(growable: false),
    scope: HotKeyScope.system,
  );
}

PhysicalKeyboardKey? _physicalKey(String value) {
  final normalized = value.trim().toUpperCase();
  if (normalized.length == 1) {
    final code = normalized.codeUnitAt(0);
    if (code >= 65 && code <= 90) {
      return PhysicalKeyboardKey(
        PhysicalKeyboardKey.keyA.usbHidUsage + code - 65,
      );
    }
    if (code >= 49 && code <= 57) {
      return PhysicalKeyboardKey(
        PhysicalKeyboardKey.digit1.usbHidUsage + code - 49,
      );
    }
    if (normalized == '0') return PhysicalKeyboardKey.digit0;
  }
  if (normalized.startsWith('F')) {
    final number = int.tryParse(normalized.substring(1));
    if (number != null && number >= 1 && number <= 12) {
      return PhysicalKeyboardKey(
        PhysicalKeyboardKey.f1.usbHidUsage + number - 1,
      );
    }
  }
  return switch (normalized) {
    'SPACE' => PhysicalKeyboardKey.space,
    'ENTER' || 'RETURN' => PhysicalKeyboardKey.enter,
    'TAB' => PhysicalKeyboardKey.tab,
    'ESC' || 'ESCAPE' => PhysicalKeyboardKey.escape,
    'BACKSPACE' => PhysicalKeyboardKey.backspace,
    'DELETE' => PhysicalKeyboardKey.delete,
    'UP' => PhysicalKeyboardKey.arrowUp,
    'DOWN' => PhysicalKeyboardKey.arrowDown,
    'LEFT' => PhysicalKeyboardKey.arrowLeft,
    'RIGHT' => PhysicalKeyboardKey.arrowRight,
    'HOME' => PhysicalKeyboardKey.home,
    'END' => PhysicalKeyboardKey.end,
    'PAGEUP' => PhysicalKeyboardKey.pageUp,
    'PAGEDOWN' => PhysicalKeyboardKey.pageDown,
    _ => null,
  };
}
