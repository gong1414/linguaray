bool bypassesProxy(String host, Iterable<String> rawRules) {
  final normalizedHost = host.toLowerCase();
  for (final rawRule in rawRules) {
    final rule = rawRule.trim().toLowerCase();
    if (rule.isEmpty) continue;
    if (rule == '*') return true;
    if (rule == '<local>' && !normalizedHost.contains('.')) return true;

    final withoutWildcard = rule.startsWith('*.') ? rule.substring(2) : rule;
    final domain = withoutWildcard.startsWith('.')
        ? withoutWildcard.substring(1)
        : withoutWildcard;
    if (normalizedHost == domain || normalizedHost.endsWith('.$domain')) {
      return true;
    }
  }
  return false;
}
