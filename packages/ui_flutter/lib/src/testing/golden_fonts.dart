import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/services.dart' show FontLoader;

/// Native faces used for deterministic desktop goldens.
///
/// Production uses the platform UI font instead of shipping a private font
/// bundle. Tests register host font files under the production theme family names so
/// macOS and Windows can each maintain an intentional baseline.
Map<String, String> get goldenHostFaces {
  if (Platform.isWindows) {
    final windows = Platform.environment['WINDIR'] ?? r'C:\Windows';
    return {
      'Segoe UI': '$windows\\Fonts\\segoeui.ttf',
      'Microsoft YaHei UI': '$windows\\Fonts\\msyh.ttc',
      'sans-serif': '$windows\\Fonts\\seguisym.ttf',
    };
  }
  return const {
    'CupertinoSystemText': '/System/Library/Fonts/SFNS.ttf',
    'CupertinoSystemDisplay': '/System/Library/Fonts/SFNS.ttf',
    'PingFang SC': '/System/Library/Fonts/STHeiti Medium.ttc',
    'sans-serif': '/System/Library/Fonts/Apple Symbols.ttf',
  };
}

Future<void> _load(String family, Uint8List bytes) async {
  final loader = FontLoader(family)
    ..addFont(Future.value(bytes.buffer.asByteData()));
  await loader.load();
}

/// Registers the native text faces used by desktop visual regression tests.
Future<void> loadGoldenFonts() async {
  for (final entry in goldenHostFaces.entries) {
    await _load(entry.key, File(entry.value).readAsBytesSync());
  }
}
