/// Stable export surface for the settings routes.
///
/// Each settings domain lives in its own library so changes to one area do not
/// pull every settings implementation into a single oversized module.
library;

export '../../features/integrations/system_settings_screens.dart';
export '../../features/preferences/general_settings_screen.dart';
export '../../features/services/service_settings_screens.dart';
export '../../features/shortcuts/shortcut_settings_screens.dart';
export 'settings_host_screen.dart';
