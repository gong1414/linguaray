/// Pure Dart application contracts and use cases for LinguaRay.
///
/// This package deliberately has no Flutter, platform-plugin, networking, or
/// FFI dependency. Desktop adapters implement its ports; views consume the
/// immutable output models through a view model.
library;

export 'src/dictionary/look_up_word.dart';
export 'src/dictionary/models.dart';
export 'src/dictionary/ports.dart';
export 'src/errors/mapping.dart';
export 'src/errors/models.dart';
export 'src/glossary/models.dart';
export 'src/glossary/ports.dart';
export 'src/history/models.dart';
export 'src/history/ports.dart';
export 'src/history/record_completed_translation.dart';
export 'src/protocol/models.dart';
export 'src/protocol/parse.dart';
export 'src/settings/models.dart';
export 'src/settings/ports.dart';
export 'src/speech/models.dart';
export 'src/speech/ports.dart';
export 'src/translation/load_translation_catalog.dart';
export 'src/translation/models.dart';
export 'src/translation/ports.dart';
export 'src/translation/translate_text.dart';
export 'src/updates/check_for_update.dart';
export 'src/updates/models.dart';
export 'src/updates/ports.dart';
export 'src/updates/semver.dart';
export 'src/vocabulary/models.dart';
export 'src/vocabulary/ports.dart';
