import 'dart:ui' show Locale;

import '../i18n/i18n.dart';
import '../services/runtime.dart' show runtime;

/// Sentinel value for auto-detected source language.
const kAutoSource = 'auto';

List<String>? _appLanguages;
List<String>? _supportedLanguages;

List<String> get appLanguages {
  _appLanguages ??= runtime
      .listAppLanguages()
      .map((lang) => lang.code)
      .toList();
  return _appLanguages!;
}

List<String> get supportedLanguages {
  _supportedLanguages ??= runtime
      .listLanguages()
      .map((lang) => lang.code)
      .toList();
  return _supportedLanguages!;
}

String defaultTargetLanguageForAppLanguage(String languageTag) {
  final normalized = languageTag.trim().toLowerCase().replaceAll('_', '-');
  final code = switch (normalized) {
    'en' || 'en-us' || 'en-gb' => 'en',
    'ja' || 'ja-jp' => 'ja',
    'ko' || 'ko-kr' => 'ko',
    'zh-hant' || 'zh-tw' || 'zh-hk' => 'zh-Hant',
    'zh' || 'zh-hans' || 'zh-cn' || 'zh-sg' => 'zh-Hans',
    _ => 'zh-Hans',
  };
  return _preferredLanguage(code);
}

String _preferredLanguage(String code) {
  final languages = supportedLanguages;
  if (languages.contains(code)) return code;
  return languages.isNotEmpty ? languages.first : code;
}

/// Returns the language name in the current interface locale.
String getLanguageName(String language) =>
    _languageNameFromT(language) ?? language;

/// Looks up the translated name for a language code via the i18n system.
String? _languageNameFromT(String language) {
  final value =
      t['common.language.${language.toLowerCase().replaceAll('-', '_')}'];
  return value is String ? value : null;
}

/// Returns the default set of common language codes.
/// Always includes a broad set of widely-used languages, filtered to only
/// those actually supported by the current app build.
List<String> defaultCommonLanguages() {
  const base = <String>[
    'en',
    'zh-Hans',
    'zh-Hant',
    'ja',
    'ko',
    'fr',
    'de',
    'es',
    'ru',
    'pt',
    'ar',
    'it',
  ];
  // Keep only languages that are actually in the supported list.
  return base.where((code) => supportedLanguages.contains(code)).toList();
}

Locale languageToLocale(String language) {
  final parts = language.split('-');
  if (parts.length >= 2 && parts[1].length == 4) {
    return Locale.fromSubtags(languageCode: parts[0], scriptCode: parts[1]);
  }
  if (parts.length >= 2) {
    return Locale(parts[0], parts[1]);
  }
  return Locale(parts[0]);
}
