import 'package:beyondtranslate_runtime/beyondtranslate_runtime.dart'
    as beyondtranslate_runtime;
import 'package:flutter/material.dart';

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
  _smokeTestBeyondtranslateRuntime();
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

void _smokeTestBeyondtranslateRuntime() {
  beyondtranslate_runtime.ensureInitialized();
  debugPrint(
    '[beyondtranslate_runtime] Dart version() = '
    '${beyondtranslate_runtime.version()}',
  );
  debugPrint(
    '[beyondtranslate_runtime] Dart add(a: 2, b: 3) = '
    '${beyondtranslate_runtime.add(a: 2, b: 3)}',
  );
  debugPrint(
    '[beyondtranslate_runtime] Dart greet(name: "main.dart") = '
    '${beyondtranslate_runtime.greet(name: 'main.dart')}',
  );
}

Future<void> main() async {
  await _ensureInitialized();

  await LocaleSettings.setLocaleRaw(
    languageToLocale(settingsStore.appLanguage).toLanguageTag(),
  );
  debugPrint('[LinguaRay] Locale initialized.');

  runApp(const RootView());
  debugPrint('[LinguaRay] Root widget mounted.');
}
