int _sequence = 0;

/// Returns a process- and restart-unique ID for one translation request.
///
/// History rows use this value as an upsert key while a request streams. A
/// timestamp alone is not sufficient when two windows submit in the same
/// microsecond, so a process-local sequence is appended as well.
String newTranslationSessionId() {
  final timestamp = DateTime.now().microsecondsSinceEpoch.toRadixString(36);
  final sequence = (_sequence++).toRadixString(36);
  return '$timestamp-$sequence';
}
