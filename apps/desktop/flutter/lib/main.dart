import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_runtime/linguaray_runtime.dart' as linguaray_runtime;

import 'src/i18n/i18n.dart';
import 'src/platform/onboarding_controller.dart';
import 'src/platform/secret_store.dart';
import 'src/routes/app_router.dart';
import 'src/services/runtime.dart' show initRuntime;
import 'src/services/settings_store.dart';
import 'src/utils/env.dart';
import 'src/utils/language_util.dart';

Future<void> _ensureInitialized() async {
  WidgetsFlutterBinding.ensureInitialized();
  // Fail fast if the committed Dart bindings do not match the bundled
  // native library (API version / checksum mismatch).
  linguaray_runtime.ensureInitialized();
  await initRuntime();
  debugPrint('[LinguaRay] Rust runtime initialized.');

  await initEnv();
  debugPrint('[LinguaRay] Environment initialized.');
  initOnboardingController();
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
