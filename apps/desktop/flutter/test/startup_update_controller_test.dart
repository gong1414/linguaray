import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/platform/startup_update_controller.dart';

void main() {
  test(
    'resident checks honor interval, preference changes, resume, and disposal',
    () {
      fakeAsync((time) {
        var enabled = true;
        var calls = 0;
        final clock = time.getClock(DateTime(2026));
        final controller = StartupUpdateController(
          enabled: () => enabled,
          now: clock.now,
          runCheck: () async {
            calls++;
            return const UpdateState.idle('0.6.1');
          },
        );
        controller.start();
        time.flushMicrotasks();
        expect(calls, 1);
        unawaited(controller.check()); // Resume before the interval.
        time.flushMicrotasks();
        expect(calls, 1);
        time.elapse(const Duration(hours: 6));
        expect(calls, 2);
        enabled = false;
        time.elapse(const Duration(hours: 12));
        expect(calls, 2);
        enabled = true;
        unawaited(controller.check()); // Settings changed or app resumed.
        time.flushMicrotasks();
        expect(calls, 3);
        controller.dispose();
        time.elapse(const Duration(days: 1));
        expect(calls, 3);
      });
    },
  );

  test('concurrent automatic checks share one request', () async {
    final pending = Completer<UpdateState>();
    var calls = 0;
    final controller = StartupUpdateController(
      enabled: () => true,
      runCheck: () {
        calls++;
        return pending.future;
      },
    );
    final first = controller.check();
    final second = controller.check();
    expect(identical(first, second), isTrue);
    expect(calls, 1);
    controller.dispose();
    pending.complete(const UpdateState.idle('0.6.1'));
    await first; // Completion after disposal must not notify a dead listener.
  });
}
