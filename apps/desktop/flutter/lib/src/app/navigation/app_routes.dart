import 'package:flutter/foundation.dart' show kDebugMode;
import 'package:go_router/go_router.dart';

import '../../features/translation/quick_translate/quick_translate_screen.dart';
import '../windows/app_windows.dart';
import 'settings_routes.dart' as settings_route;

const debugInitialRoute = String.fromEnvironment('LINGUARAY_INITIAL_ROUTE');

SettingsDestination? get debugInitialDestination {
  if (!kDebugMode || debugInitialRoute.trim().isEmpty) return null;
  final route = debugInitialRoute.trim();
  for (final destination in SettingsDestination.values) {
    if (destination.location == route) return destination;
  }
  return null;
}

GoRouter createSettingsAppRouter({String? initialLocation}) {
  return GoRouter(
    routes: <RouteBase>[
      GoRoute(
        path: '/',
        redirect: (_, _) => SettingsDestination.settingsTranslation.location,
      ),
      ...settings_route.$appRoutes,
    ],
    initialLocation: initialLocation ?? pendingSettingsLocation,
    debugLogDiagnostics: false,
  );
}

GoRouter createMiniTranslatorAppRouter() {
  return GoRouter(
    routes: [
      GoRoute(
        path: '/',
        builder: (context, state) => const QuickTranslateScreen(),
      ),
    ],
    debugLogDiagnostics: false,
  );
}
