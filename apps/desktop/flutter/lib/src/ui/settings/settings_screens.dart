/// Stable export surface for the settings routes.
///
/// Each settings domain lives in its own library so changes to one area do not
/// pull every settings implementation into a single oversized module.
library;

export 'general_settings_screen.dart';
export 'service_settings_screens.dart';
export 'settings_host_screen.dart';
export 'shortcut_settings_screens.dart';
export 'system_settings_screens.dart';
