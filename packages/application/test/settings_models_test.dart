import 'package:linguaray_application/linguaray_application.dart';
import 'package:test/test.dart';

void main() {
  test('access snapshot reports when a grant is still needed', () {
    const denied = AccessSnapshot(
      accessibility: AccessState.denied,
      screenRecording: AccessState.granted,
    );
    const windows = AccessSnapshot.notRequired();

    expect(denied.needsAttention, isTrue);
    expect(windows.needsAttention, isFalse);
    expect(windows.accessibility, AccessState.notRequired);
  });

  test('provider records never need a plaintext secret field', () {
    const provider = ProviderRecord(
      id: 'deepl',
      typeId: 'deepl',
      displayName: 'DeepL',
      publicFields: {'baseUrl': 'https://api.deepl.com'},
      storedSecretKeys: {'authKey'},
    );

    expect(provider.hasStoredSecret, isTrue);
    expect(provider.publicFields.containsKey('authKey'), isFalse);
  });
}
