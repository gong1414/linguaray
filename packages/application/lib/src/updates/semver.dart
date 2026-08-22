/// Compares two dotted semantic versions, ignoring a leading `v`.
///
/// Pre-release suffixes after `-` or `+` are stripped so `1.2.3-beta` compares
/// as `1.2.3`. Returns true when [candidate] is strictly newer than [current].
bool isNewerVersion(String candidate, String current) {
  final left = _parts(candidate);
  final right = _parts(current);
  final length = left.length > right.length ? left.length : right.length;
  for (var i = 0; i < length; i++) {
    final a = i < left.length ? left[i] : 0;
    final b = i < right.length ? right[i] : 0;
    if (a > b) return true;
    if (a < b) return false;
  }
  return false;
}

List<int> _parts(String version) {
  var value = version.trim();
  if (value.toLowerCase().startsWith('v')) {
    value = value.substring(1);
  }
  final cut = value.split(RegExp(r'[-+]')).first;
  return [for (final part in cut.split('.')) int.tryParse(part) ?? 0];
}
