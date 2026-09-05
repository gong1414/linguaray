import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_runtime/linguaray_runtime.dart' as linguaray_runtime;

import 'src/app/env.dart';
import 'src/app/navigation/app_router.dart';
import 'src/app/runtime.dart' show initRuntime;
import 'src/app/settings/settings_store.dart';
import 'src/i18n/i18n.dart';
import 'src/platform/credentials/secret_store.dart';
import 'src/platform/network/system_proxy.dart';
import 'src/shared/language_util.dart';

Future<void> _ensureInitialized() async {
  WidgetsFlutterBinding.ensureInitialized();
  // Fail fast if the committed Dart bindings do not match the bundled
  // native library (API version / checksum mismatch).
  linguaray_runtime.ensureInitialized();
  await initRuntime();
  debugPrint('[LinguaRay] Rust runtime initialized.');

  await initEnv();
  debugPrint('[LinguaRay] Environment initialized.');
  initProviderCredentialsController();
  try {
    await providerCredentialsController.hydrateAll();
  } catch (_) {
    // A locked or unavailable system vault must not prevent the settings and
    // permission UI from opening. No provider secret or platform error detail
    // is written to logs.
    debugPrint('[LinguaRay] Provider credentials are currently unavailable.');
  }
  debugPrint('[LinguaRay] Provider credentials hydrated.');
  await settingsStore.init();
  debugPrint('[LinguaRay] Settings initialized.');
  await initializeSystemProxy();
  debugPrint('[LinguaRay] Network policy initialized.');
}

Future<void> main() async {
  await _ensureInitialized();

  await LocaleSettings.setLocaleRaw(
    languageToLocale(settingsStore.appLanguage).toLanguageTag(),
  );
  debugPrint('[LinguaRay] Locale initialized.');

  runApp(const ProviderScope(child: RootView()));
  debugPrint('[LinguaRay] Root widget mounted.');
}
