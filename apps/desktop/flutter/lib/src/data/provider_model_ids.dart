List<String> mergeProviderModelIds({
  String? saved,
  Iterable<String> live = const [],
  Iterable<String> snapshot = const [],
}) {
  final merged = <String>[];
  final savedId = saved?.trim();
  if (savedId != null && savedId.isNotEmpty) merged.add(savedId);

  for (final id in [...live, ...snapshot]) {
    if (!merged.contains(id)) merged.add(id);
  }
  return merged;
}
