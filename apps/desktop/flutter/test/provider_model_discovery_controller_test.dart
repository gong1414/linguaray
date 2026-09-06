import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/features/providers/provider_model_discovery_controller.dart';

void main() {
  const a = ProviderDraft(id: 'a', typeId: 'openai_compatible', fields: {});
  const b = ProviderDraft(id: 'b', typeId: 'openai_compatible', fields: {});
  testWidgets(
    'edits debounce, configuration switches discard older responses',
    (tester) async {
      final calls = <String>[];
      final first = Completer<ProviderModelDiscovery>();
      final second = Completer<ProviderModelDiscovery>();
      final controller = ProviderModelDiscoveryController((draft) {
        calls.add(draft.id);
        return draft.id == 'a' ? first.future : second.future;
      });
      addTearDown(controller.dispose);
      controller.schedule(b);
      await tester.pump(const Duration(milliseconds: 300));
      controller.schedule(a);
      await tester.pump(const Duration(milliseconds: 701));
      expect(calls, ['a']);
      controller.schedule(b, immediately: true);
      second.complete(const ProviderModelDiscovery(liveModels: ['b/model']));
      await tester.pump();
      first.complete(const ProviderModelDiscovery(liveModels: ['a/model']));
      await tester.pump();
      expect(controller.result!.liveModels, ['b/model']);
      expect(controller.loading, isFalse);
      controller.schedule(null);
      expect(controller.result, isNull);
    },
  );

  testWidgets('errors stay errors and completing after dispose is safe', (
    tester,
  ) async {
    final controller = ProviderModelDiscoveryController(
      (_) async => const ProviderModelDiscovery(
        referenceModels: ['offline/model'],
        errorCode: 'auth_error',
      ),
    );
    controller.schedule(a, immediately: true);
    await tester.pump();
    expect(controller.result!.succeeded, isFalse);
    expect(controller.result!.liveModels, isEmpty);
    expect(controller.result!.errorCode, 'auth_error');
    controller.dispose();
    final pending = Completer<ProviderModelDiscovery>();
    final disposed = ProviderModelDiscoveryController((_) => pending.future);
    disposed.schedule(a, immediately: true);
    disposed.dispose();
    pending.complete(const ProviderModelDiscovery());
    await tester.pump();
    expect(tester.takeException(), isNull);
  });
}
