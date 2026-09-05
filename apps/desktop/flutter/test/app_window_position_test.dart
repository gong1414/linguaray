import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/src/platform/windows/window_positioning.dart';

void main() {
  test('wide reading window shrinks to a small secondary work area', () {
    const workArea = Rect.fromLTWH(-600, 24, 600, 456);
    final fitted = fitPopoverToWorkArea(
      position: const Offset(-200, 300),
      desiredSize: const Size(720, 800),
      workArea: workArea,
    );
    expect(fitted, workArea);
  });

  test('growing result moves upward while preserving its reading width', () {
    const workArea = Rect.fromLTWH(0, 24, 1440, 876);
    final fitted = fitPopoverToWorkArea(
      position: const Offset(700, 600),
      desiredSize: const Size(720, 500),
      workArea: workArea,
    );
    expect(fitted, const Rect.fromLTWH(700, 400, 720, 500));
  });

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
      expect(workArea.contains(position), isTrue);
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
