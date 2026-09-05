import 'dart:convert';
import 'dart:ffi';

import 'package:cnativeapi/cnativeapi.dart';
import 'package:ffi/ffi.dart';
import 'package:nativeapi/nativeapi.dart';

/// Applies a configured LinguaRay shortcut to a native menu item.
///
/// `nativeapi` 0.1.4 exposes menu items but not the accelerator property in
/// its Dart wrapper. The underlying stable C API does expose it, so this small
/// adapter keeps the status-menu shortcut column native on macOS and Windows.
void setNativeMenuAccelerator(MenuItem item, String shortcut) {
  final accelerator = NativeMenuAccelerator.tryParse(shortcut);
  if (accelerator == null) return;

  final native = calloc<native_keyboard_accelerator_t>();
  try {
    native.ref.modifiers = accelerator.modifiers;
    final keyBytes = utf8.encode(accelerator.key);
    if (keyBytes.length >= 64) return;
    for (var index = 0; index < keyBytes.length; index++) {
      native.ref.key[index] = keyBytes[index];
    }
    native.ref.key[keyBytes.length] = 0;
    cnativeApiBindings.native_menu_item_set_accelerator(
      item.nativeHandle,
      native,
    );
  } finally {
    calloc.free(native);
  }
}

class NativeMenuAccelerator {
  const NativeMenuAccelerator({required this.modifiers, required this.key});

  final int modifiers;
  final String key;

  static NativeMenuAccelerator? tryParse(String shortcut) {
    final parts = shortcut
        .split('+')
        .map((part) => part.trim())
        .where((part) => part.isNotEmpty)
        .toList(growable: false);
    if (parts.isEmpty) return null;

    var modifiers = 0;
    for (final part in parts.take(parts.length - 1)) {
      switch (part.toLowerCase()) {
        case 'control':
        case 'ctrl':
          modifiers |= native_accelerator_modifier_t
              .NATIVE_ACCELERATOR_MODIFIER_CTRL
              .value;
        case 'option':
        case 'alt':
          modifiers |= native_accelerator_modifier_t
              .NATIVE_ACCELERATOR_MODIFIER_ALT
              .value;
        case 'shift':
          modifiers |= native_accelerator_modifier_t
              .NATIVE_ACCELERATOR_MODIFIER_SHIFT
              .value;
        case 'command':
        case 'cmd':
        case 'meta':
        case 'super':
          modifiers |= native_accelerator_modifier_t
              .NATIVE_ACCELERATOR_MODIFIER_META
              .value;
        default:
          return null;
      }
    }

    final rawKey = parts.last;
    const specialKeys = <String>{
      'F1',
      'F2',
      'F3',
      'F4',
      'F5',
      'F6',
      'F7',
      'F8',
      'F9',
      'F10',
      'F11',
      'F12',
      'Enter',
      'Return',
      'Tab',
      'Space',
      'Escape',
      'Delete',
      'Backspace',
      'ArrowUp',
      'ArrowDown',
      'ArrowLeft',
      'ArrowRight',
    };
    if (rawKey.runes.length != 1 && !specialKeys.contains(rawKey)) return null;
    return NativeMenuAccelerator(
      modifiers: modifiers,
      key: rawKey.runes.length == 1 ? rawKey.toLowerCase() : rawKey,
    );
  }
}
