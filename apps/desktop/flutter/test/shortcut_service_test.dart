import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hotkey_manager/hotkey_manager.dart';
import 'package:linguaray_desktop/src/services/shortcut_service/shortcut_service.dart';

void main() {
  test('parses stored desktop accelerators', () {
    final shortcut = parseStoredShortcut('Command + Shift + A');

    expect(shortcut, isNotNull);
    expect(shortcut!.key, PhysicalKeyboardKey.keyA);
    expect(shortcut.modifiers,
        containsAll([HotKeyModifier.meta, HotKeyModifier.shift]));
    expect(shortcut.scope, HotKeyScope.system);
  });

  test('accepts aliases and function keys', () {
    final shortcut = parseStoredShortcut('Ctrl+Alt+F12');

    expect(shortcut, isNotNull);
    expect(shortcut!.key, PhysicalKeyboardKey.f12);
    expect(
      shortcut.modifiers,
      containsAll([HotKeyModifier.control, HotKeyModifier.alt]),
    );
  });

  test('rejects missing modifiers and multiple primary keys', () {
    expect(parseStoredShortcut('Space'), isNull);
    expect(parseStoredShortcut('Command+A+B'), isNull);
    expect(parseStoredShortcut('Command+DefinitelyNotAKey'), isNull);
  });
}
