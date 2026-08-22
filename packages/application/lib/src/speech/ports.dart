import 'package:linguaray_application/src/speech/models.dart';

abstract interface class SpeechService {
  Stream<SpeechState> get states;

  Future<bool> isAvailable();

  Future<SpeechState> speak({
    required String text,
    required SpeechUtteranceKind kind,
    String? language,
  });

  Future<SpeechState> stop();

  Future<void> dispose();
}
