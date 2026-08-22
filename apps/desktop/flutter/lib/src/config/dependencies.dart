import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../data/permission_repository.dart';
import '../data/runtime_translation_repository.dart';
import '../data/shortcut_repository.dart';
import '../data/workspace_settings_repository.dart';

final translationRepositoryProvider = Provider<TranslationRepository>(
  (ref) => RuntimeTranslationRepository(),
);

final loadTranslationCatalogProvider = Provider<LoadTranslationCatalog>(
  (ref) => LoadTranslationCatalog(ref.watch(translationRepositoryProvider)),
);

final translateTextProvider = Provider<TranslateText>(
  (ref) => TranslateText(ref.watch(translationRepositoryProvider)),
);

final workspaceSettingsRepositoryProvider =
    Provider<WorkspaceSettingsRepository>(
      (ref) => RuntimeWorkspaceSettingsRepository(),
    );

final permissionRepositoryProvider = Provider<PermissionRepository>(
  (ref) => ControllerPermissionRepository(),
);

final shortcutRepositoryProvider = Provider<ShortcutRepository>(
  (ref) => RuntimeShortcutRepository(),
);
