import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../data/channel_speech_service.dart';
import '../data/github_update_repository.dart';
import '../data/permission_repository.dart';
import '../data/runtime_dictionary_repository.dart';
import '../data/runtime_glossary_repository.dart';
import '../data/runtime_history_repository.dart';
import '../data/runtime_translation_repository.dart';
import '../data/runtime_vocabulary_repository.dart';
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

final historyRepositoryProvider = Provider<HistoryRepository>(
  (ref) => RuntimeHistoryRepository(),
);

final recordCompletedTranslationProvider = Provider<RecordCompletedTranslation>(
  (ref) => RecordCompletedTranslation(ref.watch(historyRepositoryProvider)),
);

final glossaryRepositoryProvider = Provider<GlossaryRepository>(
  (ref) => RuntimeGlossaryRepository(),
);

final dictionaryRepositoryProvider = Provider<DictionaryRepository>(
  (ref) => RuntimeDictionaryRepository(),
);

final lookUpWordProvider = Provider<LookUpWord>(
  (ref) => LookUpWord(ref.watch(dictionaryRepositoryProvider)),
);

final vocabularyRepositoryProvider = Provider<VocabularyRepository>(
  (ref) => RuntimeVocabularyRepository(),
);

final speechServiceProvider = Provider<SpeechService>((ref) {
  final service = ChannelSpeechService();
  ref.onDispose(service.dispose);
  return service;
});

final updateRepositoryProvider = Provider<UpdateRepository>(
  (ref) => GitHubUpdateRepository(),
);

final updateInstallerProvider = Provider<UpdateInstaller>(
  (ref) => DesktopUpdateInstaller(),
);

final checkForUpdateProvider = Provider<CheckForUpdate>(
  (ref) => CheckForUpdate(ref.watch(updateRepositoryProvider)),
);

final downloadVerifiedUpdateProvider = Provider<DownloadVerifiedUpdate>(
  (ref) => DownloadVerifiedUpdate(ref.watch(updateRepositoryProvider)),
);

final parseProtocolLinkProvider = Provider<ParseProtocolLink>(
  (ref) => const ParseProtocolLink(),
);
