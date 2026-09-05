import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../features/integrations/data/system_settings_adapter.dart';
import '../features/library/glossary/data/runtime_glossary_repository.dart';
import '../features/library/glossary/glossary_exchange_controller.dart';
import '../features/library/history/data/runtime_history_repository.dart';
import '../features/library/vocabulary/data/runtime_vocabulary_repository.dart';
import '../features/preferences/data/general_settings_adapter.dart';
import '../features/providers/data/provider_settings_adapter.dart';
import '../features/services/data/service_settings_adapter.dart';
import '../features/translation/data/runtime_dictionary_repository.dart';
import '../features/translation/data/runtime_translation_repository.dart';
import '../features/updates/data/github_update_repository.dart';
import '../platform/credentials/secret_store.dart';
import '../platform/files/text_file_dialogs.dart';
import '../platform/network/network_proxy.dart';
import '../platform/permissions/permission_repository.dart';
import '../platform/shortcuts/shortcut_repository.dart';
import '../platform/speech/channel_speech_service.dart';
import 'env.dart';
import 'settings/settings_store.dart';

final translationRepositoryProvider = Provider<TranslationRepository>(
  (ref) => RuntimeTranslationRepository(),
);

final loadTranslationCatalogProvider = Provider<LoadTranslationCatalog>(
  (ref) => LoadTranslationCatalog(ref.watch(translationRepositoryProvider)),
);

final translateTextProvider = Provider<TranslateText>(
  (ref) => TranslateText(ref.watch(translationRepositoryProvider)),
);

final preferencesRepositoryProvider = Provider<PreferencesRepository>(
  (ref) => RuntimeGeneralSettingsAdapter(settingsStore),
);
final translationPreferencesRepositoryProvider =
    Provider<TranslationPreferencesRepository>(
      (ref) => RuntimeGeneralSettingsAdapter(settingsStore),
    );
final appInfoRepositoryProvider = Provider<AppInfoRepository>(
  (ref) => RuntimeSystemSettingsAdapter(settingsStore),
);
final integrationSettingsRepositoryProvider =
    Provider<IntegrationSettingsRepository>(
      (ref) => RuntimeSystemSettingsAdapter(settingsStore),
    );
final providerSettingsRepositoryProvider = Provider<ProviderSettingsRepository>(
  (ref) => RuntimeProviderSettingsAdapter(
    settingsStore,
    providerCredentialsController,
  ),
);
final serviceSettingsRepositoryProvider = Provider<ServiceSettingsRepository>(
  (ref) => RuntimeServiceSettingsAdapter(
    settingsStore,
    ref.watch(appInfoRepositoryProvider).loadCapabilities,
  ),
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

final updateRepositoryProvider = Provider<UpdateRepository>((ref) {
  final repository = GitHubUpdateRepository(client: createNetworkHttpClient());
  ref.onDispose(repository.close);
  return repository;
});

final updateInstallerProvider = Provider<UpdateInstaller>(
  (ref) =>
      DesktopUpdateInstaller(repository: ref.watch(updateRepositoryProvider)),
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

final updateCurrentVersionProvider = Provider<String>(
  (ref) => Env.instance.appVersion,
);

// Expose an application snapshot to interactive views without leaking the
// runtime settings object or running OS synchronization during build.
final translationInteractionPreferencesProvider = Provider<GeneralPreferences>((
  ref,
) {
  void changed() => ref.invalidateSelf();
  settingsStore.addListener(changed);
  ref.onDispose(() => settingsStore.removeListener(changed));
  return RuntimeGeneralSettingsAdapter(settingsStore).currentPreferences;
});

final glossaryExchangeControllerProvider = Provider<GlossaryExchangeController>(
  (ref) => GlossaryExchangeController(
    ref.watch(glossaryRepositoryProvider),
    const TextFileDialogs(),
  ),
);
