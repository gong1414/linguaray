enum SpeechStatus { idle, speaking, interrupted, unavailable, failed }

final class SpeechState {
  const SpeechState({required this.status, this.errorCode});

  const SpeechState.idle() : status = SpeechStatus.idle, errorCode = null;

  final SpeechStatus status;
  final String? errorCode;

  bool get isSpeaking => status == SpeechStatus.speaking;
  bool get isAvailable => status != SpeechStatus.unavailable;
}

enum SpeechUtteranceKind { source, translation, headword }
