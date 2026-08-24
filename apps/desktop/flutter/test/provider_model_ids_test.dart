import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_desktop/src/data/provider_model_ids.dart';

void main() {
  test('merges saved, live, and snapshot models in stable priority order', () {
    expect(
      mergeProviderModelIds(
        saved: ' saved-model ',
        live: const ['live-a', 'saved-model', 'live-b'],
        snapshot: const ['live-a', 'snapshot-a'],
      ),
      const ['saved-model', 'live-a', 'live-b', 'snapshot-a'],
    );
  });

  test('omits an empty saved model without disturbing fallbacks', () {
    expect(
      mergeProviderModelIds(
        saved: '  ',
        snapshot: const ['snapshot-a', 'snapshot-a'],
      ),
      const ['snapshot-a'],
    );
  });
}
