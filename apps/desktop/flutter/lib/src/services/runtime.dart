import 'dart:io';

import 'package:linguaray_runtime/linguaray_runtime.dart';
import 'package:path/path.dart' as path;
import 'package:path_provider/path_provider.dart';

// Re-export uniffi-generated types so that files importing this service do
// not need a separate import of linguaray_runtime.
export 'package:linguaray_runtime/linguaray_runtime.dart'
    show
        // Settings types
        AdvancedSettings,
        AdvancedSettingsPatch,
        ApiServerInfo,
        AppearanceSettings,
        AppearanceSettingsPatch,
        BackupSummary,
        ExternalActionKind,
        ExternalActionRequest,
        ExternalActionSubscription,
        GeneralSettings,
        GeneralSettingsPatch,
        InputSubmitMode,
        ProviderConfigEntry,
        ServiceConfigEntry,
        ServiceType,
        ShortcutSettings,
        ShortcutSettingsPatch,
        TranslationTarget,
        // Glossary types
        GlossaryBook,
        GlossaryBookInput,
        GlossaryComplianceIssue,
        GlossaryEntry,
        GlossaryEntryInput,
        GlossaryIssueKind,
        GlossaryMatch,
        RuntimeGlossary,
        // History types
        HistoryCounts,
        HistoryEntry,
        HistoryEntryInput,
        HistoryFilter,
        RuntimeHistory,
        RuntimeVocabulary,
        VocabularyEntry,
        VocabularyEntryInput,
        VocabularyFilter,
        // Translation / look-up types
        DetectLanguageRequest,
        DetectLanguageResponse,
        LookUpRequest,
        LookUpResponse,
        TextTranslation,
        TranslateRequest,
        TranslateResponse,
        WordDefinition,
        WordEtymology,
        WordImage,
        WordPhrase,
        WordPronunciation,
        WordSentence,
        WordSynonym,
        WordTag,
        WordTense,
        ProviderType,
        RecognizeTextRequest,
        RecognizeTextResponse,
        RuntimeApiServer,
        RuntimeBackup,
        RuntimeOcr,
        RuntimeTextExtractor,
        RestoreSummary,
        SelectionExtraction;

/// Singleton [Runtime] handle, backed by the Rust native library.
///
/// The Rust side keeps a single shared instance per `data_dir`, so this
/// handle references the **same** in-memory state as the [Runtime] used
/// by the native macOS Settings UI (Swift). Writes from either side are
/// immediately visible to the other on the next read.
///
/// Call [initRuntime] during app startup (before [settingsStore.init]) to
/// populate this variable.
late final Runtime runtime;
late final Directory runtimeDataDirectory;
RuntimeApiServer? _apiServer;
ApiServerInfo? _apiServerInfo;

const _runtimeDataDirectoryOverride = String.fromEnvironment(
  'LINGUARAY_RUNTIME_DATA_DIR',
);

ApiServerInfo? get apiServerInfo => _apiServerInfo;

/// Initialises the Rust runtime with the platform's application-support
/// directory as the data directory.
///
/// Must be called before any code that accesses [runtime].
Future<void> initRuntime() async {
  if (_runtimeDataDirectoryOverride.trim().isNotEmpty) {
    runtimeDataDirectory = Directory(_runtimeDataDirectoryOverride.trim());
  } else {
    final supportDirectory = await getApplicationSupportDirectory();
    runtimeDataDirectory = Directory(path.join(supportDirectory.path, 'v2'));
  }
  await runtimeDataDirectory.create(recursive: true);
  runtime = Runtime(dataDir: runtimeDataDirectory.path);
}

Future<ApiServerInfo?> applyApiServerSettings(AdvancedSettings settings) async {
  if (!settings.apiServerEnabled) {
    stopApiServer();
    return null;
  }

  final host = settings.apiServerHost.trim().isEmpty
      ? '127.0.0.1'
      : settings.apiServerHost.trim();
  final port = settings.apiServerPort;
  final current = _apiServerInfo;
  if (_apiServer != null &&
      current != null &&
      current.host == host &&
      (port == 0 || current.port == port)) {
    return current;
  }

  stopApiServer();
  final server = runtime.startApiServer(host: host, port: port);
  _apiServer = server;
  _apiServerInfo = server.info();
  return _apiServerInfo;
}

void stopApiServer() {
  _apiServer?.stop();
  _apiServer = null;
  _apiServerInfo = null;
}

/// A simple error class used to record translation / dictionary lookup
/// failures in [TranslationResultRecord].
class TranslationError {
  final String message;

  const TranslationError({required this.message});

  factory TranslationError.fromJson(Map<String, dynamic> json) =>
      TranslationError(message: json['message'] ?? '');

  Map<String, dynamic> toJson() => {'message': message};
}
