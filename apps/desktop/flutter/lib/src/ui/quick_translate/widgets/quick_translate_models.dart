final class QuickTranslateLabels {
  const QuickTranslateLabels({
    required this.title,
    required this.inputHint,
    required this.translate,
    required this.clear,
    required this.copy,
    required this.copied,
    required this.pin,
    required this.unpin,
    required this.capture,
    required this.clipboard,
    required this.openSettings,
    required this.autoDetect,
    required this.autoMatch,
    required this.swapLanguages,
    required this.translating,
    required this.empty,
    required this.retry,
    required this.configureServices,
    required this.permissionDenied,
    required this.permissionNext,
    required this.captureCancelled,
    required this.serviceError,
    required this.noServices,
    required this.failureMessage,
    this.sourceLabel = '',
    this.resultLabel = '',
    this.resultPlaceholder = '',
    this.captureFailed = '',
    this.ocrNotConfigured = '',
    this.ocrEmpty = '',
    this.emptySelection = '',
    this.clipboardUnavailable = '',
    this.clipboardRestoreFailed = '',
    this.recheck = '',
    this.speakSource = '',
    this.speakResult = '',
    this.stopSpeaking = '',
    this.lookup = '',
    this.saveVocabulary = '',
    this.vocabularySaved = '',
    this.favorite = '',
    this.unfavorite = '',
    this.glossaryMatches = '',
    this.glossaryWarnings = '',
  });

  final String sourceLabel;
  final String resultLabel;
  final String resultPlaceholder;
  final String title;
  final String inputHint;
  final String translate;
  final String clear;
  final String copy;
  final String copied;
  final String pin;
  final String unpin;
  final String capture;
  final String clipboard;
  final String openSettings;
  final String autoDetect;
  final String autoMatch;
  final String swapLanguages;
  final String translating;
  final String empty;
  final String retry;
  final String configureServices;
  final String permissionDenied;
  final String permissionNext;
  final String captureCancelled;
  final String serviceError;
  final String noServices;
  final String Function(String? code) failureMessage;
  final String captureFailed;
  final String ocrNotConfigured;
  final String ocrEmpty;
  final String emptySelection;
  final String clipboardUnavailable;
  final String clipboardRestoreFailed;
  final String recheck;
  final String speakSource;
  final String speakResult;
  final String stopSpeaking;
  final String lookup;
  final String saveVocabulary;
  final String vocabularySaved;
  final String favorite;
  final String unfavorite;
  final String glossaryMatches;
  final String glossaryWarnings;
}

enum QuickTranslateNotice {
  none,
  permissionDenied,
  captureCancelled,
  captureFailed,
  ocrNotConfigured,
  ocrEmpty,
  emptySelection,
  clipboardUnavailable,
  clipboardRestoreFailed,
}
