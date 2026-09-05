import 'package:pub_semver/pub_semver.dart';

/// Stable releases outrank their prereleases. Build metadata does not affect
/// precedence, and malformed versions must never trigger an update.
bool isNewerVersion(String candidate, String current) {
  Version parse(String value) {
    value = value.trim().replaceFirst(RegExp(r'^[vV]'), '');
    return Version.parse(value.split('+').first);
  }

  try {
    return parse(candidate) > parse(current);
  } on FormatException {
    return false;
  }
}
