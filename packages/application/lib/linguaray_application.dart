/// Pure Dart application contracts and use cases for LinguaRay.
///
/// This package deliberately has no Flutter, platform-plugin, networking, or
/// FFI dependency. Desktop adapters implement its ports; views consume the
/// immutable output models through a view model.
library;

export 'src/settings/models.dart';
export 'src/settings/ports.dart';
export 'src/translation/load_translation_catalog.dart';
export 'src/translation/models.dart';
export 'src/translation/ports.dart';
export 'src/translation/translate_text.dart';
