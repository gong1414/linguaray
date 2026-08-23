import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/src/platform/menu_accelerator.dart';

void main() {
  test('parses configured shortcuts for the native menu column', () {
    final accelerator = NativeMenuAccelerator.tryParse('Option+Shift+W');

    expect(accelerator, isNotNull);
    expect(accelerator!.modifiers, 2 | 4);
    expect(accelerator.key, 'w');
  });

  test('rejects empty, multi-key and unknown shortcuts', () {
    expect(NativeMenuAccelerator.tryParse(''), isNull);
    expect(NativeMenuAccelerator.tryParse('Option+PageUp'), isNull);
    expect(NativeMenuAccelerator.tryParse('Hyper+Q'), isNull);
  });

  test('keeps native special-key names intact', () {
    expect(NativeMenuAccelerator.tryParse('Command+Space')?.key, 'Space');
    expect(NativeMenuAccelerator.tryParse('Ctrl+F12')?.key, 'F12');
  });
}
