import 'dart:io';

import 'package:flutter_test/flutter_test.dart';

void main() {
  final dartSpeech = File('lib/src/platform/speech/channel_speech_service.dart')
      .readAsStringSync();
  final dartProtocol = File(
    'lib/src/platform/protocol/protocol_controller.dart',
  ).readAsStringSync();
  final dartProxy = File('lib/src/platform/network/system_proxy.dart')
      .readAsStringSync();
  final macHost = Directory('macos/Runner/Plugins')
      .listSync()
      .whereType<File>()
      .where((file) => file.path.endsWith('.swift'))
      .map((file) => file.readAsStringSync())
      .join('\n');
  final windowsHost = Directory('windows/runner')
      .listSync()
      .whereType<File>()
      .where((file) => file.path.endsWith('.cpp') || file.path.endsWith('.h'))
      .map((file) => file.readAsStringSync())
      .join('\n');

  test('Dart and both desktop hosts agree on shared channel names', () {
    final contracts = <String, List<String>>{
      'linguaray/speech': [dartSpeech, macHost, windowsHost],
      'linguaray/protocol': [dartProtocol, macHost, windowsHost],
      'linguaray/system_proxy': [dartProxy, macHost, windowsHost],
    };

    for (final MapEntry(key: channel, value: sources) in contracts.entries) {
      for (final source in sources) {
        expect(source, contains(channel), reason: 'Missing $channel contract');
      }
    }
  });

  test('removed native channels are absent from the macOS host', () {
    final mainWindow = File('macos/Runner/MainFlutterWindow.swift')
        .readAsStringSync();
    expect(mainWindow, isNot(contains('MacWindowAppearancePlugin')));
    expect(mainWindow, isNot(contains('NativeTextFieldPlugin')));
    expect(mainWindow, isNot(contains('NativeTextPlugin')));
    expect(macHost, isNot(contains('linguaray/mac_window_appearance')));
  });
}
