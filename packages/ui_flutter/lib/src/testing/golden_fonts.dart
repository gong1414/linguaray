import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/services.dart' show FontLoader;

import 'package:linguaray_ui/src/theme/tokens.dart';

/// Golden-test support: the host font faces and type roles the design tokens
/// resolve to, shared by every suite that renders goldens (the kit's own
/// golden test and the app's business-widget twin).
///
/// Import it from `package:linguaray_ui/testing.dart`, never from the main
/// barrel — nothing in production code should touch these.

/// Where pub put a dependency, read off the package config rather than the
/// asset bundle: the bundle's root depends on which directory `flutter test`
/// was invoked from, and these goldens have to render the same either way.
Directory _packageRoot(String package) {
  for (var dir = Directory.current; ; dir = dir.parent) {
    final config = File('${dir.path}/.dart_tool/package_config.json');
    if (config.existsSync()) {
      final packages =
          (jsonDecode(config.readAsStringSync()) as Map)['packages'] as List;
      for (final entry in packages.cast<Map<String, dynamic>>()) {
        if (entry['name'] != package) continue;
        return Directory.fromUri(
          config.uri.resolve(entry['rootUri'] as String),
        );
      }
    }
    if (dir.parent.path == dir.path) {
      throw StateError(
        '$package is not in any package_config.json above ${Directory.current.path}',
      );
    }
  }
}

/// Native faces used for deterministic desktop goldens.
///
/// Production uses the platform UI font instead of shipping a private font
/// bundle. Tests register the host font files under stable family aliases so
/// macOS and Windows can each maintain an intentional baseline.
Map<String, String> get goldenHostFaces {
  if (Platform.isWindows) {
    final windows = Platform.environment['WINDIR'] ?? r'C:\Windows';
    return {
      'Golden UI': '$windows\\Fonts\\segoeui.ttf',
      'Golden CJK': '$windows\\Fonts\\msyh.ttc',
      'Golden Mono': '$windows\\Fonts\\consola.ttf',
      'Golden Symbols': '$windows\\Fonts\\seguisym.ttf',
    };
  }
  return const {
    'Golden UI': '/System/Library/Fonts/SFNS.ttf',
    'Golden CJK': '/System/Library/Fonts/STHeiti Medium.ttc',
    'Golden Mono': '/System/Library/Fonts/Menlo.ttc',
    'Golden Symbols': '/System/Library/Fonts/Apple Symbols.ttf',
  };
}

/// The type roles bound to those faces. `flutter test` does not resolve
/// `family: null` to the platform UI font the way the running app does, so the
/// roles have to name the families that were just registered.
const goldenTypography = DesignTypography(
  display: DesignFont(
    family: 'Golden UI',
    fallback: ['Golden CJK', 'Golden Symbols'],
  ),
  sans: DesignFont(
    family: 'Golden UI',
    fallback: ['Golden CJK', 'Golden Symbols'],
  ),
  label: DesignFont(
    family: 'Golden UI',
    fallback: ['Golden CJK', 'Golden Symbols'],
  ),
  cjk: DesignFont(
    family: 'Golden CJK',
    fallback: ['Golden UI', 'Golden Symbols'],
  ),
  mono: DesignFont(
    family: 'Golden Mono',
    fallback: ['Golden CJK', 'Golden Symbols'],
  ),
);

/// Host face names whose font file is missing on this machine — a non-empty
/// list means the suite must skip rather than report false diffs.
List<String> missingGoldenHostFaces() {
  return [
    for (final entry in goldenHostFaces.entries)
      if (!File(entry.value).existsSync()) entry.key,
  ];
}

Future<void> _load(String family, Uint8List bytes) async {
  final loader = FontLoader(family)
    ..addFont(Future.value(bytes.buffer.asByteData()));
  await loader.load();
}

/// Registers every face a golden render needs: the macOS system faces the
/// tokens name, plus the Fluent icon font pair from
/// `fluentui_system_icons`. Call once in `setUpAll`.
Future<void> loadGoldenFonts() async {
  for (final entry in goldenHostFaces.entries) {
    await _load(entry.key, File(entry.value).readAsBytesSync());
  }
  final icons = _packageRoot('fluentui_system_icons');
  for (final font in const ['Regular', 'Filled']) {
    final file = File('${icons.path}/lib/fonts/FluentSystemIcons-$font.ttf');
    if (!file.existsSync()) {
      throw StateError('missing icon font: ${file.path}');
    }
    await _load(
      'packages/fluentui_system_icons/FluentSystemIcons-$font',
      file.readAsBytesSync(),
    );
  }
}
