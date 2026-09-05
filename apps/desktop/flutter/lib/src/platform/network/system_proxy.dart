import 'dart:io';

import 'package:flutter/services.dart';

import 'proxy_bypass.dart';

class SystemProxySnapshot {
  const SystemProxySnapshot({this.http, this.https, this.bypass = const []});

  final String? http;
  final String? https;
  final List<String> bypass;

  String resolve(Uri uri) {
    if (bypassesProxy(uri.host, bypass)) return 'DIRECT';
    final proxy = uri.scheme == 'https' ? https ?? http : http;
    return proxy == null || proxy.isEmpty ? 'DIRECT' : 'PROXY $proxy; DIRECT';
  }
}

const _channel = MethodChannel('linguaray/system_proxy');
SystemProxySnapshot? _snapshot;

Future<void> initializeSystemProxy() async {
  if (!Platform.isMacOS && !Platform.isWindows) return;
  try {
    final raw = await _channel.invokeMapMethod<String, Object?>('read');
    if (raw == null) return;
    _snapshot = SystemProxySnapshot(
      http: raw['http'] as String?,
      https: raw['https'] as String?,
      bypass: (raw['bypass'] as List<Object?>? ?? const [])
          .whereType<String>()
          .toList(growable: false),
    );
  } on PlatformException {
    _snapshot = null;
  }
}

String findSystemProxy(Uri uri) =>
    _snapshot?.resolve(uri) ?? HttpClient.findProxyFromEnvironment(uri);
