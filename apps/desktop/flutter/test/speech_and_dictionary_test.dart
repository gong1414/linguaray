import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:linguaray_application/linguaray_application.dart';
import 'package:linguaray_desktop/src/data/channel_speech_service.dart';
import 'package:linguaray_desktop/src/ui/translation/widgets/dictionary_lookup_dialog.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('speech service publishes native completion state', () async {
    const channel = MethodChannel('linguaray/test-speech');
    final messenger =
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
    messenger.setMockMethodCallHandler(channel, (call) async {
      if (call.method == 'isAvailable') return true;
      return null;
    });
    addTearDown(() => messenger.setMockMethodCallHandler(channel, null));

    final service = ChannelSpeechService(channel: channel);
    addTearDown(service.dispose);
    final states = <SpeechStatus>[];
    final subscription = service.states.listen(
      (state) => states.add(state.status),
    );
    addTearDown(subscription.cancel);

    final started = await service.speak(
      text: 'Hello',
      kind: SpeechUtteranceKind.source,
      language: 'en',
    );
    expect(started.status, SpeechStatus.speaking);

    await _sendNativeMethodCall(
      messenger,
      channel.name,
      const MethodCall('stateChanged', 'idle'),
    );
    expect(states, [SpeechStatus.speaking, SpeechStatus.idle]);
  });

  testWidgets('dictionary result can be saved to vocabulary', (tester) async {
    var saved = false;
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: DictionaryLookupDialog(
            labels: const DictionaryLookupDialogLabels(
              title: 'Dictionary',
              pronunciation: 'Pronunciation',
              definitions: 'Definitions',
              save: 'Save',
              saved: 'Saved',
              close: 'Close',
              empty: 'No result',
              lookupFailed: 'Lookup failed',
              saveFailed: 'Save failed',
            ),
            lookup: Future.value(
              const DictionaryEntry(
                word: 'apple',
                providerName: 'ECDICT',
                serviceId: 'ecdict+dictionary',
                translations: ['苹果'],
                pronunciations: [
                  DictionaryPronunciation(text: '/ˈæpəl/', accent: 'US'),
                ],
                definitions: [
                  DictionaryDefinition(
                    partOfSpeech: 'noun',
                    values: ['a round fruit'],
                  ),
                ],
              ),
            ),
            onSave: (_) async => saved = true,
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('apple'), findsOneWidget);
    expect(find.text('苹果'), findsOneWidget);
    expect(find.textContaining('a round fruit'), findsOneWidget);
    await tester.tap(find.text('Save'));
    await tester.pumpAndSettle();
    expect(saved, isTrue);
    expect(find.text('Saved'), findsOneWidget);
  });
}

Future<void> _sendNativeMethodCall(
  TestDefaultBinaryMessenger messenger,
  String channel,
  MethodCall call,
) {
  final completed = Completer<void>();
  messenger.handlePlatformMessage(
    channel,
    const StandardMethodCodec().encodeMethodCall(call),
    (ByteData? _) => completed.complete(),
  );
  return completed.future;
}
