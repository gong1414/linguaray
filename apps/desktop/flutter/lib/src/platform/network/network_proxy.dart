import 'dart:io';

import '../../app/runtime.dart' show AdvancedSettings;
import 'proxy_bypass.dart';
import 'system_proxy.dart';

HttpClient createNetworkHttpClient({
  required AdvancedSettings Function() readAdvanced,
}) {
  final client = HttpClient();
  client.findProxy = (uri) => findLinguaRayProxy(uri, advanced: readAdvanced());
  return client;
}

String findLinguaRayProxy(Uri uri, {required AdvancedSettings advanced}) {
  switch (advanced.proxyMode) {
    case 'direct':
      return 'DIRECT';
    case 'custom':
      if (bypassesProxy(uri.host, advanced.proxyBypass.split(','))) {
        return 'DIRECT';
      }
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
