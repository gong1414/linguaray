import 'dart:io';

import 'settings_store.dart';
import 'system_proxy.dart';

HttpClient createNetworkHttpClient() {
  final client = HttpClient();
  client.findProxy = findLinguaRayProxy;
  return client;
}

String findLinguaRayProxy(Uri uri) {
  final advanced = settingsStore.advanced;
  switch (advanced.proxyMode) {
    case 'direct':
      return 'DIRECT';
    case 'custom':
      if (_bypassesProxy(uri.host, advanced.proxyBypass)) return 'DIRECT';
      final proxy = Uri.tryParse(advanced.proxyUrl.trim());
      if (proxy == null ||
          (proxy.scheme != 'http' && proxy.scheme != 'https') ||
          proxy.host.isEmpty ||
          proxy.userInfo.isNotEmpty) {
        return 'DIRECT';
      }
      final port = proxy.port;
      if (port == 0) return 'DIRECT';
      return 'PROXY ${proxy.host}:$port; DIRECT';
    default:
      return findSystemProxy(uri);
  }
}

bool _bypassesProxy(String host, String rawRules) {
  final normalizedHost = host.toLowerCase();
  for (final rawRule in rawRules.split(',')) {
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
