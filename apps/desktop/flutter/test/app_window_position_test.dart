import 'package:beyondtranslate_desktop/src/services/app_windows.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('quick window placement', () {
    const workArea = Rect.fromLTWH(100, 50, 1200, 800);
    const size = Size(396, 420);

    test('prefers below and right of the pointer', () {
      expect(
        positionPopoverNearPoint(
          point: const Offset(300, 200),
          windowSize: size,
          workArea: workArea,
        ),
        const Offset(312, 212),
      );
    });

    test('flips above and left near the bottom-right edge', () {
      final position = positionPopoverNearPoint(
        point: const Offset(1250, 800),
        windowSize: size,
        workArea: workArea,
      );
      expect(position, const Offset(842, 368));
      expect(
        workArea.contains(position),
        isTrue,
      );
      expect(position.dx + size.width, lessThanOrEqualTo(workArea.right));
      expect(position.dy + size.height, lessThanOrEqualTo(workArea.bottom));
    });

    test('clamps an oversized popover to the work-area origin', () {
      expect(
        positionPopoverNearPoint(
          point: const Offset(120, 70),
          windowSize: const Size(1400, 900),
          workArea: workArea,
        ),
        workArea.topLeft,
      );
    });
  });
}
