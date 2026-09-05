import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:linguaray_application/linguaray_application.dart';

import '../features/backup/backup_port.dart';
import '../features/backup/data/file_selector_backup_picker.dart';
import '../features/backup/data/runtime_backup_archive.dart';
import '../features/integrations/data/system_settings_adapter.dart';
import '../features/library/glossary/data/runtime_glossary_repository.dart';
import '../features/library/glossary/glossary_exchange_controller.dart';
import '../features/library/history/data/runtime_history_repository.dart';
import '../features/library/vocabulary/data/runtime_vocabulary_repository.dart';
import '../features/ocr/ocr_controller.dart';
import '../features/preferences/data/general_settings_adapter.dart';
import '../features/providers/data/provider_settings_adapter.dart';
import '../features/services/data/service_settings_adapter.dart';
import '../features/translation/data/runtime_dictionary_repository.dart';
import '../features/translation/data/runtime_translation_repository.dart';
import '../features/updates/data/github_update_repository.dart';
import '../platform/credentials/secret_store.dart';
import '../platform/files/text_file_dialogs.dart';
import '../platform/network/network_proxy.dart';
import '../platform/network/system_proxy.dart';
import '../platform/permissions/permission_repository.dart';
import '../platform/shortcuts/shortcut_repository.dart';
import '../platform/speech/channel_speech_service.dart';
import 'env.dart';
import 'settings/settings_section.dart';
import 'settings/settings_store.dart';

final settingsStoreProvider = Provider<SettingsStore>((ref) => settingsStore);

final ocrControllerProvider = Provider<OcrController>((ref) => ocrController);

final translationRepositoryProvider = Provider<TranslationRepository>(
  (ref) =>
      RuntimeTranslationRepository(store: ref.watch(settingsStoreProvider)),
);

final loadTranslationCatalogProvider = Provider<LoadTranslationCatalog>(
  (ref) => LoadTranslationCatalog(ref.watch(translationRepositoryProvider)),
);

final translateTextProvider = Provider<TranslateText>(
  (ref) => TranslateText(ref.watch(translationRepositoryProvider)),
);

final preferencesRepositoryProvider = Provider<PreferencesRepository>(
  (ref) => RuntimeGeneralSettingsAdapter(ref.watch(settingsStoreProvider)),
);
final translationPreferencesRepositoryProvider =
    Provider<TranslationPreferencesRepository>(
      (ref) => RuntimeGeneralSettingsAdapter(ref.watch(settingsStoreProvider)),
    );
final appInfoRepositoryProvider = Provider<AppInfoRepository>(
  (ref) => RuntimeSystemSettingsAdapter(ref.watch(settingsStoreProvider)),
);
final integrationSettingsRepositoryProvider =
    Provider<IntegrationSettingsRepository>(
      (ref) => RuntimeSystemSettingsAdapter(ref.watch(settingsStoreProvider)),
    );
final providerSettingsRepositoryProvider = Provider<ProviderSettingsRepository>(
  (ref) => RuntimeProviderSettingsAdapter(
    ref.watch(settingsStoreProvider),
    providerCredentialsController,
  ),
);
final serviceSettingsRepositoryProvider = Provider<ServiceSettingsRepository>(
  (ref) => RuntimeServiceSettingsAdapter(
    ref.watch(settingsStoreProvider),
    ref.watch(appInfoRepositoryProvider).loadCapabilities,
  ),
);

final permissionRepositoryProvider = Provider<PermissionRepository>(
  (ref) => ControllerPermissionRepository(),
);

final shortcutRepositoryProvider = Provider<ShortcutRepository>(
  (ref) => RuntimeShortcutRepository(store: ref.watch(settingsStoreProvider)),
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
  (ref) => RuntimeDictionaryRepository(store: ref.watch(settingsStoreProvider)),
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
  final store = ref.watch(settingsStoreProvider);
  final repository = GitHubUpdateRepository(
    client: createNetworkHttpClient(readAdvanced: () => store.advanced),
  );
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
  final store = ref.watch(settingsStoreProvider);
  final listenable = store.listenablesFor(const [
    SettingsSection.general,
    SettingsSection.appearance,
  ]);
  void changed() => ref.invalidateSelf();
  listenable.addListener(changed);
  ref.onDispose(() => listenable.removeListener(changed));
  return RuntimeGeneralSettingsAdapter(store).currentPreferences;
});

final glossaryExchangeControllerProvider = Provider<GlossaryExchangeController>(
  (ref) => GlossaryExchangeController(
    ref.watch(glossaryRepositoryProvider),
    const TextFileDialogs(),
  ),
);

final backupArchiveProvider = Provider<BackupArchive>(
  (ref) => const RuntimeBackupArchive(),
);

final backupFilePickerProvider = Provider<BackupFilePicker>(
  (ref) => const FileSelectorBackupPicker(),
);

typedef BackupRestoreEffects = Future<void> Function();

final backupRestoreEffectsProvider = Provider<BackupRestoreEffects>((ref) {
  final store = ref.watch(settingsStoreProvider);
  return () async {
    await providerCredentialsController.hydrateAll();
    await Future.wait([
      store.reloadAppearance(),
      store.reloadGeneral(),
      store.reloadShortcuts(),
      store.reloadAdvanced(),
      store.reloadProviders(),
      store.reloadServices(),
    ]);
    await initializeSystemProxy();
  };
});
