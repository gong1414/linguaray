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

  test('windows capabilities hide system translation and dictionary', () {
    const windows = PlatformCapabilities.windows();
    const macos = PlatformCapabilities.macos();

    expect(windows.systemOcr, isTrue);
    expect(windows.systemTranslation, isFalse);
    expect(windows.systemLanguageDetection, isFalse);
    expect(windows.systemDictionary, isFalse);
    expect(macos.systemTranslation, isTrue);
    expect(macos.systemDictionary, isTrue);
  });

  test('general preferences keep new workflow fields', () {
    const preferences = GeneralPreferences(
      launchAtLogin: false,
      showInMenuBar: true,
      language: 'zh-Hans',
      themeMode: ThemePreference.system,
      commonLanguages: ['en', 'zh-Hans'],
      inputSubmitMode: InputSubmitMode.commandEnter,
      autoCopyDetectedText: false,
      doubleClickCopyResult: true,
    );

    expect(preferences.commonLanguages, ['en', 'zh-Hans']);
    expect(preferences.inputSubmitMode, InputSubmitMode.commandEnter);
    expect(preferences.autoCopyDetectedText, isFalse);
  });
}
