use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use linguaray_core::{
    ChatMessage, DetectLanguageRequest, DetectLanguageResponse, LanguageInfo, LanguagePair,
    LookUpRequest, LookUpResponse, RecognizeTextRequest, RecognizeTextResponse, TranslateRequest,
    TranslateResponse,
};
use struct_patch::Patch as ApplyPatch;
use tokio::sync::{broadcast, Mutex as AsyncMutex, RwLock};

use crate::backup;
use crate::domain::engine;
use crate::domain::glossary::{
    check_compliance, GlossaryBook, GlossaryBookInput, GlossaryComplianceIssue, GlossaryEntry,
    GlossaryEntryInput, GlossaryImportReport, GlossaryMatch, GlossaryStore,
};
use crate::domain::glossary_exchange::{self, GlossaryExchangeFormat};
use crate::domain::history::{
    HistoryCounts, HistoryEntry, HistoryEntryInput, HistoryFilter, HistoryStore,
};
use crate::domain::permissions as permission;
use crate::domain::settings::{
    append_translation_service_order, apply_catalog_seed, effective_translation_service_order,
    provider_entry_from_config, remove_provider_from_translation_order,
    remove_translation_service_order, AdvancedSettings, AdvancedSettingsPatch, AppearanceSettings,
    AppearanceSettingsPatch, GeneralSettings, GeneralSettingsPatch, ProviderConfigEntry,
    ServiceConfigEntry, ServiceType, Settings, ShortcutSettings, ShortcutSettingsPatch,
};
use crate::domain::text_extractor;
use crate::domain::vocabulary::{
    VocabularyEntry, VocabularyEntryInput, VocabularyFilter, VocabularyStore,
};
use crate::RuntimeApiServer;
use linguaray_core::TranslationTarget;
use linguaray_engine::prompt::GlossaryTerm;

/// Error type returned by all uniffi-exported Runtime methods.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum RuntimeError {
    #[error("{msg}")]
    Error { msg: String },
}

impl From<String> for RuntimeError {
    fn from(s: String) -> Self {
        RuntimeError::Error { msg: s }
    }
}

/// Identifies which top-level settings section was just modified.
///
/// Emitted by [`SettingsSubscription::next`] every time settings are
/// successfully written through any [`RuntimeSettings`] handle. Consumers
/// (Dart `SettingsStore`, Swift `SettingsViewModel`, etc.) receive these
/// events regardless of which language binding initiated the change, and
/// typically respond by re-fetching the affected section.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, uniffi::Enum)]
pub enum SettingsChange {
    General,
    Appearance,
    Shortcuts,
    Advanced,
    Providers,
    /// A glossary book or term was created, edited or deleted. Glossary data
    /// lives outside `settings.json` but rides the same event channel so
    /// consumers keep a single subscription loop.
    Glossary,
    /// Translation history was created, edited, favorited or deleted.
    History,
    /// A vocabulary book entry was created, edited, favorited or deleted.
    Vocabulary,
}

/// A UI action requested by a loopback API client. The runtime transports
/// intent only; Flutter remains responsible for windows and platform input.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, uniffi::Enum)]
pub enum ExternalActionKind {
    TranslateText,
    TranslateSelection,
    TranslateInput,
    TranslateClipboard,
    CaptureTranslate,
    CaptureOcr,
    ClipboardOcr,
    ShowTranslationWindow,
    ShowOcrWindow,
    OpenSettings,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct ExternalActionRequest {
    pub kind: ExternalActionKind,
    pub text: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Record)]
pub struct BackupSummary {
    pub created_at: u64,
    pub file_count: u32,
    pub includes_secrets: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Record)]
pub struct RestoreSummary {
    pub created_at: u64,
    pub includes_secrets: bool,
}

/// Callback invoked by the Rust runtime as LLM streaming chunks arrive.
///
/// Dart/Swift implement this trait and pass it to
/// [`RuntimeLlm::translate_stream`]. The Rust side calls:
///
/// 1. `on_chunk(content)` — for each token delta (may be called many times)
/// 2. `on_finish(reason)` — when the stream completes (`"stop"`, `"length"`, etc.)
/// 3. `on_error(error)` — if the stream encounters an error
#[uniffi::export(callback_interface)]
pub trait StreamCallback: Send + Sync {
    fn on_chunk(&self, content: String);
    fn on_finish(&self, finish_reason: String);
    fn on_error(&self, error: String);
}

/// Broadcast channel buffer size; settings updates are infrequent so 64
/// is generous. If a subscriber falls more than this many events behind,
/// they receive [`broadcast::error::RecvError::Lagged`] and we transparently
/// resume from the next event.
const EVENT_CHANNEL_CAPACITY: usize = 64;

struct RuntimeState {
    settings: Settings,
    engine: linguaray_engine::Engine,
    /// Credentials hydrated from the OS secure store for this process only.
    /// Persisted settings contain opaque references instead of these values.
    provider_secrets: HashMap<String, HashMap<String, String>>,
}

impl RuntimeState {
    fn new(mut settings: Settings) -> Result<(Self, bool), String> {
        let seeded = apply_catalog_seed(&mut settings);
        let provider_secrets = HashMap::new();
        let engine = engine::build_from_settings_with_secrets(&settings, &provider_secrets)?;
        Ok((
            Self {
                settings,
                engine,
                provider_secrets,
            },
            seeded,
        ))
    }
}

/// Shared, process-wide state behind a [`Runtime`] handle. All [`Runtime`]
/// instances built for the same `data_dir` reference the **same**
/// `RuntimeInner`, so Dart and Swift bindings observe the same in-memory
/// settings/engine state and the same `settings.json` on disk.
struct RuntimeInner {
    settings_file_path: Arc<str>,
    state: RwLock<RuntimeState>,
    /// Terms live in their own files under `data_dir/glossary`, behind their
    /// own lock so editing a book never blocks a translation waiting on
    /// settings (or vice versa).
    glossary: RwLock<GlossaryStore>,
    /// Translation history has its own file and lock, so listing it never
    /// blocks settings or glossary operations.
    history: RwLock<HistoryStore>,
    /// Saved words live beside history, with the same isolation so a corrupt
    /// vocabulary file cannot block translation.
    vocabulary: RwLock<VocabularyStore>,
    /// Broadcasts a [`SettingsChange`] event after every successful
    /// settings write. The sender is kept alive for the lifetime of
    /// `RuntimeInner`, so receivers obtained via `subscribe()` will only
    /// see [`broadcast::error::RecvError::Closed`] once the runtime itself
    /// is dropped (i.e. process shutdown).
    events: broadcast::Sender<SettingsChange>,
    /// Loopback API actions are independent from settings notifications so a
    /// slow settings observer cannot delay a user-visible command.
    action_events: broadcast::Sender<ExternalActionRequest>,
}

/// Process-wide registry mapping a canonical `data_dir` path to the
/// [`RuntimeInner`] currently backing it. The first call to
/// [`Runtime::new`] for a given `data_dir` populates the entry; subsequent
/// calls (regardless of which language binding they originate from) return
/// a cheap [`Runtime`] handle that shares the same `Arc<RuntimeInner>`.
static RUNTIME_REGISTRY: OnceLock<Mutex<HashMap<PathBuf, Arc<RuntimeInner>>>> = OnceLock::new();

fn runtime_registry() -> &'static Mutex<HashMap<PathBuf, Arc<RuntimeInner>>> {
    RUNTIME_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Resolves `data_dir` to a stable canonical path used as the registry key.
/// The directory is created if missing so that paths from different
/// language bindings (`/Users/...` vs `/private/Users/...` symlinks on
/// macOS, trailing slashes, etc.) collapse to the same key.
fn canonical_data_dir(data_dir: &str) -> PathBuf {
    let raw = PathBuf::from(data_dir);
    let _ = std::fs::create_dir_all(&raw);
    std::fs::canonicalize(&raw).unwrap_or(raw)
}

#[derive(Clone, uniffi::Object)]
pub struct Runtime {
    inner: Arc<RuntimeInner>,
}

#[derive(uniffi::Object)]
pub struct RuntimeSettings {
    runtime: Runtime,
}

#[derive(uniffi::Object)]
pub struct RuntimeTranslation {
    runtime: Runtime,
    service_id: String,
}

#[derive(uniffi::Object)]
pub struct RuntimeDictionary {
    runtime: Runtime,
    service_id: String,
}

#[derive(Clone, uniffi::Object)]
pub struct RuntimeLlm {
    runtime: Runtime,
    service_id: String,
}

#[derive(uniffi::Object)]
pub struct RuntimeOcr {
    runtime: Runtime,
    service_id: String,
}

#[derive(uniffi::Object)]
pub struct RuntimeGlossary {
    runtime: Runtime,
}

#[derive(uniffi::Object)]
pub struct RuntimeHistory {
    runtime: Runtime,
}

#[derive(uniffi::Object)]
pub struct RuntimeVocabulary {
    runtime: Runtime,
}

#[derive(uniffi::Object)]
pub struct RuntimeBackup {
    runtime: Runtime,
}

#[derive(uniffi::Object)]
pub struct RuntimePermission;

/// Rust-native screen text extractor.
///
/// Provides clipboard reading, screen selection text extraction,
/// and screen capture with OCR across all supported platforms.
#[derive(uniffi::Object)]
pub struct RuntimeTextExtractor {
    _runtime: Runtime,
}

/// Foreign-language handle for observing [`SettingsChange`] events.
///
/// Obtain one via [`RuntimeSettings::subscribe`] and call
/// [`SettingsSubscription::next`] in a loop:
///
/// * `Some(change)` — a section was modified; reload it if you care.
/// * `None` — the runtime has been torn down and no further events
///   will arrive (terminate the loop).
///
/// Each subscription has its own independent cursor in the broadcast
/// channel; multiple subscribers can coexist and all see the same events.
#[derive(uniffi::Object)]
pub struct SettingsSubscription {
    receiver: AsyncMutex<broadcast::Receiver<SettingsChange>>,
}

#[derive(uniffi::Object)]
pub struct ExternalActionSubscription {
    receiver: AsyncMutex<broadcast::Receiver<ExternalActionRequest>>,
}

impl Runtime {
    fn new_impl(data_dir: String) -> Result<Self, String> {
        let key = canonical_data_dir(&data_dir);

        let mut registry = runtime_registry()
            .lock()
            .map_err(|error| format!("runtime registry mutex poisoned: {error}"))?;

        if let Some(existing) = registry.get(&key) {
            return Ok(Self {
                inner: existing.clone(),
            });
        }

        let settings_file_path = key.join("settings.json");
        let settings = Settings::load(&settings_file_path)?;
        let (state, seeded) = RuntimeState::new(settings)?;
        if seeded {
            state.settings.save(&settings_file_path)?;
        }
        let glossary = GlossaryStore::load(&key)?;
        let history = HistoryStore::load(&key);
        let vocabulary = VocabularyStore::load(&key);
        let (events, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        let (action_events, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        let inner = Arc::new(RuntimeInner {
            settings_file_path: Arc::from(settings_file_path.to_string_lossy().into_owned()),
            state: RwLock::new(state),
            glossary: RwLock::new(glossary),
            history: RwLock::new(history),
            vocabulary: RwLock::new(vocabulary),
            events,
            action_events,
        });
        registry.insert(key, inner.clone());
        Ok(Self { inner })
    }
}

#[uniffi::export]
impl Runtime {
    #[uniffi::constructor]
    pub fn new(data_dir: String) -> Result<Arc<Self>, RuntimeError> {
        Self::new_impl(data_dir).map(Arc::new).map_err(Into::into)
    }

    pub fn settings(self: Arc<Self>) -> Arc<RuntimeSettings> {
        Arc::new(RuntimeSettings {
            runtime: (*self).clone(),
        })
    }

    pub fn translation(
        self: Arc<Self>,
        provider_id: String,
    ) -> Result<Arc<RuntimeTranslation>, RuntimeError> {
        let service_id = validate_service_provider_id(provider_id, "+translation")
            .map_err(RuntimeError::from)?;
        Ok(Arc::new(RuntimeTranslation {
            runtime: (*self).clone(),
            service_id,
        }))
    }

    pub fn dictionary(
        self: Arc<Self>,
        provider_id: String,
    ) -> Result<Arc<RuntimeDictionary>, RuntimeError> {
        let service_id =
            validate_service_provider_id(provider_id, "+dictionary").map_err(RuntimeError::from)?;
        Ok(Arc::new(RuntimeDictionary {
            runtime: (*self).clone(),
            service_id,
        }))
    }

    pub fn ocr(self: Arc<Self>, provider_id: String) -> Result<Arc<RuntimeOcr>, RuntimeError> {
        let service_id =
            validate_service_provider_id(provider_id, "+ocr").map_err(RuntimeError::from)?;
        Ok(Arc::new(RuntimeOcr {
            runtime: (*self).clone(),
            service_id,
        }))
    }

    pub fn llm(self: Arc<Self>, provider_id: String) -> Result<Arc<RuntimeLlm>, RuntimeError> {
        let service_id =
            validate_service_provider_id(provider_id, "+llm").map_err(RuntimeError::from)?;
        Ok(Arc::new(RuntimeLlm {
            runtime: (*self).clone(),
            service_id,
        }))
    }

    pub fn text_extractor(self: Arc<Self>) -> Arc<RuntimeTextExtractor> {
        Arc::new(RuntimeTextExtractor {
            _runtime: (*self).clone(),
        })
    }

    pub fn glossary(self: Arc<Self>) -> Arc<RuntimeGlossary> {
        Arc::new(RuntimeGlossary {
            runtime: (*self).clone(),
        })
    }

    pub fn history(self: Arc<Self>) -> Arc<RuntimeHistory> {
        Arc::new(RuntimeHistory {
            runtime: (*self).clone(),
        })
    }

    pub fn vocabulary(self: Arc<Self>) -> Arc<RuntimeVocabulary> {
        Arc::new(RuntimeVocabulary {
            runtime: (*self).clone(),
        })
    }

    pub fn backup(self: Arc<Self>) -> Arc<RuntimeBackup> {
        Arc::new(RuntimeBackup {
            runtime: (*self).clone(),
        })
    }

    pub fn permission(self: Arc<Self>) -> Arc<RuntimePermission> {
        Arc::new(RuntimePermission)
    }

    pub fn start_api_server(
        self: Arc<Self>,
        host: String,
        port: u16,
    ) -> Result<Arc<RuntimeApiServer>, RuntimeError> {
        RuntimeApiServer::start((*self).clone(), host, port)
    }

    pub fn subscribe_actions(&self) -> Arc<ExternalActionSubscription> {
        Arc::new(ExternalActionSubscription {
            receiver: AsyncMutex::new(self.inner.action_events.subscribe()),
        })
    }

    /// Returns the curated language list supported by the app.
    pub fn list_languages(&self) -> Vec<LanguageInfo> {
        linguaray_engine::all_languages()
    }

    /// Returns languages supported by the app UI.
    pub fn list_app_languages(&self) -> Vec<LanguageInfo> {
        linguaray_engine::app_languages()
    }
}

impl Runtime {
    pub(crate) fn emit_external_action(&self, request: ExternalActionRequest) {
        // No receiver means the UI is still starting or already shutting
        // down. The API request remains safe and does not retain user text.
        let _ = self.inner.action_events.send(request);
    }

    pub(crate) async fn api_translate(
        &self,
        provider_id: String,
        request: TranslateRequest,
    ) -> Result<TranslateResponse, linguaray_api_core::ApiError> {
        let request = linguaray_api_core::translate_request(request)?;
        let provider = {
            let state = self.inner.state.read().await;
            state
                .engine
                .require(&provider_id)
                .map_err(linguaray_api_core::ApiError::from_engine_error)?
                .clone()
        };
        let service = provider.translation().ok_or_else(|| {
            linguaray_api_core::ApiError::from_engine_error(
                linguaray_engine::EngineError::TranslationNotSupported(provider_id.clone()),
            )
        })?;

        service.translate(request).await.map_err(Into::into)
    }

    pub(crate) async fn api_detect_language(
        &self,
        provider_id: String,
        request: DetectLanguageRequest,
    ) -> Result<DetectLanguageResponse, linguaray_api_core::ApiError> {
        let request = linguaray_api_core::detect_language_request(request)?;
        let provider = {
            let state = self.inner.state.read().await;
            state
                .engine
                .require(&provider_id)
                .map_err(linguaray_api_core::ApiError::from_engine_error)?
                .clone()
        };
        let service = provider.translation().ok_or_else(|| {
            linguaray_api_core::ApiError::from_engine_error(
                linguaray_engine::EngineError::TranslationNotSupported(provider_id.clone()),
            )
        })?;

        service.detect_language(request).await.map_err(Into::into)
    }

    pub(crate) async fn api_supported_language_pairs(
        &self,
        provider_id: String,
    ) -> Result<Vec<LanguagePair>, linguaray_api_core::ApiError> {
        let provider = {
            let state = self.inner.state.read().await;
            state
                .engine
                .require(&provider_id)
                .map_err(linguaray_api_core::ApiError::from_engine_error)?
                .clone()
        };
        let service = provider.translation().ok_or_else(|| {
            linguaray_api_core::ApiError::from_engine_error(
                linguaray_engine::EngineError::TranslationNotSupported(provider_id.clone()),
            )
        })?;

        service
            .get_supported_language_pairs()
            .await
            .map_err(Into::into)
    }

    pub(crate) async fn api_lookup(
        &self,
        provider_id: String,
        request: LookUpRequest,
    ) -> Result<LookUpResponse, linguaray_api_core::ApiError> {
        let request = linguaray_api_core::lookup_request(request)?;
        let provider = {
            let state = self.inner.state.read().await;
            state
                .engine
                .require(&provider_id)
                .map_err(linguaray_api_core::ApiError::from_engine_error)?
                .clone()
        };
        let service = provider.dictionary().ok_or_else(|| {
            linguaray_api_core::ApiError::from_engine_error(
                linguaray_engine::EngineError::DictionaryNotSupported(provider_id.clone()),
            )
        })?;

        service.look_up(request).await.map_err(Into::into)
    }

    async fn resolve_service(
        &self,
        service_id: &str,
        expected_type: ServiceType,
    ) -> Result<ResolvedService, String> {
        let state = self.inner.state.read().await;
        if let Some(service) = state.settings.services.get(service_id) {
            if service.r#type != expected_type {
                return Err(format!(
                    "service `{service_id}` is not a {expected_type:?} service"
                ));
            }
            if !state.settings.providers.contains_key(&service.provider_id) {
                return Err(format!(
                    "service `{service_id}` references unknown provider `{}`",
                    service.provider_id
                ));
            }
            let engine_provider = state
                .engine
                .require(&service.provider_id)
                .map_err(|error| error.to_string())?;
            if !provider_supports_service(engine_provider.as_ref(), expected_type)
                || !advertises_system_capability(&service.provider_id, expected_type)
            {
                return Err(format!(
                    "provider `{}` does not support {expected_type:?}",
                    service.provider_id
                ));
            }
            let mut entry = service.clone();
            entry.id = service_id.to_owned();
            return Ok(ResolvedService {
                provider_id: entry.provider_id.clone(),
                entry,
            });
        }

        let provider = state
            .settings
            .providers
            .get(service_id)
            .ok_or_else(|| format!("service or provider `{service_id}` does not exist"))?;
        let engine_provider = state
            .engine
            .require(service_id)
            .map_err(|error| error.to_string())?;
        if !provider_supports_service(engine_provider.as_ref(), expected_type)
            || !advertises_system_capability(service_id, expected_type)
        {
            return Err(format!(
                "provider `{service_id}` does not support {expected_type:?}"
            ));
        }
        Ok(ResolvedService {
            provider_id: service_id.to_owned(),
            entry: service_entry_for_provider_type(
                service_id,
                &normalized_provider_entry(service_id, provider),
                expected_type,
            ),
        })
    }

    async fn resolve_llm_service(&self, service_id: &str) -> Result<ResolvedService, String> {
        match self.resolve_service(service_id, ServiceType::Llm).await {
            Ok(service) => Ok(service),
            Err(_) => {
                let state = self.inner.state.read().await;
                let Some(service) = state.settings.services.get(service_id) else {
                    return Err(format!("service or provider `{service_id}` does not exist"));
                };
                if service.r#type != ServiceType::Translation
                    && service.r#type != ServiceType::Dictionary
                {
                    return Err(format!("service `{service_id}` is not an llm service"));
                }
                if !state.settings.providers.contains_key(&service.provider_id) {
                    return Err(format!(
                        "service `{service_id}` references unknown provider `{}`",
                        service.provider_id
                    ));
                }
                let provider = state
                    .engine
                    .require(&service.provider_id)
                    .map_err(|error| error.to_string())?;
                if provider.llm().is_none() {
                    return Err(format!(
                        "provider `{}` does not support llm",
                        service.provider_id
                    ));
                }
                let mut entry = service.clone();
                entry.id = service_id.to_owned();
                Ok(ResolvedService {
                    provider_id: entry.provider_id.clone(),
                    entry,
                })
            }
        }
    }
}

// Kept in this module via include! so UniFFI export paths remain stable.
include!("runtime/settings_api.rs");

// Kept in this module via include! so UniFFI export paths remain stable.
include!("runtime/backup_api.rs");

// Kept in this module via include! so UniFFI export paths remain stable.
include!("runtime/library_api.rs");

// Kept in this module via include! so UniFFI export paths remain stable.
include!("runtime/translation_api.rs");

// Kept in this module via include! so UniFFI export paths remain stable.
include!("runtime/llm_api.rs");

// Kept in this module via include! so UniFFI export paths remain stable.
include!("runtime/platform_api.rs");

include!("runtime/support.rs");

#[cfg(test)]
mod tests;
