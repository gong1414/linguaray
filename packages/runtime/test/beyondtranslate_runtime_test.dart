import 'package:flutter_test/flutter_test.dart';
import 'package:beyondtranslate_runtime/beyondtranslate_runtime.dart' as rp;

void main() {
  // NOTE: these tests resolve `package:beyondtranslate_runtime/uniffi:beyondtranslate_runtime` via
  // Dart's native-assets system. Run them with native assets enabled:
  //
  //     flutter config --enable-native-assets
  //     flutter test
  //
  // The first invocation triggers `hook/build.dart`, which runs
  // `cargo build --release --target <triple>` for the host.

  test('core model bindings round trip through the native library', () {
    final request = rp.TranslateRequest(
      sourceLanguage: 'zh',
      targetLanguage: 'en',
      text: '你好',
    );
    final echoedRequest = rp.echoTranslateRequest(request: request);
    expect(echoedRequest.sourceLanguage, 'zh');
    expect(echoedRequest.targetLanguage, 'en');
    expect(echoedRequest.text, '你好');

    final response = rp.LookUpResponse(
      translations: [
        rp.TextTranslation(detectedSourceLanguage: 'zh', text: 'hello'),
      ],
      word: 'hello',
      tags: [rp.WordTag(name: 'noun')],
      definitions: [
        rp.WordDefinition(type: 'n', name: 'noun', values: ['hello']),
      ],
      etymology: [
        rp.WordEtymology(origin: 'Middle English', root: ['hal'])
      ],
    );
    final echoedResponse = rp.echoLookUpResponse(response: response);
    expect(echoedResponse.translations.single.text, 'hello');
    expect(echoedResponse.tags!.single.name, 'noun');
    expect(echoedResponse.definitions!.single.type, 'n');
    expect(echoedResponse.etymology!.single.origin, 'Middle English');
  });

  test('committed bindings match the native library checksums', () {
    // Fails fast when the generated Dart bindings drift from the cdylib.
    rp.ensureInitialized();
  });
}
