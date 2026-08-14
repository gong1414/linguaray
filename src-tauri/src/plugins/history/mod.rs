//! Encrypted-history service surface.

pub mod crypto;
pub mod export;
pub mod search;

pub use crate::db::history::{
    cleanup_expired, cleanup_expired_now, clear_all, privacy_status, set_enabled,
    set_enabled_preference, set_retention, HistoryPrivacyStatus, HistoryServiceError,
    RETENTION_30_DAYS, RETENTION_90_DAYS,
};

use crate::db::{Database, DbError};
use crate::keystore::{Keystore, KeystoreError};
use crate::service::TranslationOutcome;
use crypto::{encrypt_field, EncryptedField, HistoryCryptoError, HistoryField};
use rusqlite::params;
use thiserror::Error;
use zeroize::Zeroizing;

#[derive(Debug, Error)]
pub enum HistoryPersistError {
    #[error(transparent)]
    Db(#[from] DbError),
    #[error(transparent)]
    Keystore(#[from] KeystoreError),
    #[error(transparent)]
    Crypto(#[from] HistoryCryptoError),
    #[error("history consent is enabled but its key is missing")]
    MissingKey,
    #[error("system clock precedes Unix epoch")]
    Clock,
}

struct PreparedResult {
    uuid: String,
    provider_uuid: String,
    engine_id: String,
    outcome_tag: &'static str,
    result: Option<EncryptedField>,
    error_kind: Option<&'static str>,
    error: Option<EncryptedField>,
}

/// Persist one complete translation session after an explicit consent check.
///
/// The function intentionally performs the work in three non-overlapping lock
/// phases: DB consent read, keystore key read, then one DB transaction that
/// rechecks consent and inserts every encrypted row. A concurrent disable wins
/// before the final transaction and therefore produces no history row.
///
/// 8 个参数是既定签名（`run_translate_session` 位置调用），收敛成 ctx struct
/// 会让唯一调用点更绕；与 `translate_parallel` 的扁平签名保持一致更易读。
/// clippy 的 7 参数阈值是经验值，这里故意放宽。
#[allow(clippy::too_many_arguments)]
pub fn persist_translation_session(
    db: &Database,
    keystore: &Keystore,
    trigger_source: &str,
    source_text: &str,
    detected_language: Option<&str>,
    target_language: &str,
    outcomes: &[TranslationOutcome],
    elapsed_ms: u64,
) -> Result<bool, HistoryPersistError> {
    if !db.with_conn(crate::db::history::privacy_status)?.enabled {
        return Ok(false);
    }

    let key = Zeroizing::new(
        keystore
            .get_history_key()?
            .ok_or(HistoryPersistError::MissingKey)?
            .0,
    );
    let session_uuid = uuid::Uuid::new_v4().to_string();
    let encrypted_source = encrypt_field(
        &key,
        &HistoryField::SessionSource {
            uuid: &session_uuid,
        },
        source_text.as_bytes(),
    )?;

    let mut prepared = Vec::with_capacity(outcomes.len());
    for outcome in outcomes {
        let result_uuid = uuid::Uuid::new_v4().to_string();
        match &outcome.result {
            Ok(translation) => prepared.push(PreparedResult {
                uuid: result_uuid.clone(),
                provider_uuid: outcome.uuid.clone(),
                engine_id: translation.engine.clone(),
                outcome_tag: "success",
                result: Some(encrypt_field(
                    &key,
                    &HistoryField::ResultText { uuid: &result_uuid },
                    translation.text.as_bytes(),
                )?),
                error_kind: None,
                error: None,
            }),
            Err(error) => prepared.push(PreparedResult {
                uuid: result_uuid.clone(),
                provider_uuid: outcome.uuid.clone(),
                engine_id: outcome.uuid.clone(),
                outcome_tag: "failure",
                result: None,
                error_kind: Some(history_error_kind(error)),
                error: Some(encrypt_field(
                    &key,
                    &HistoryField::ResultError { uuid: &result_uuid },
                    error.to_string().as_bytes(),
                )?),
            }),
        }
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| HistoryPersistError::Clock)?;
    let timestamp = i64::try_from(timestamp.as_secs()).map_err(|_| HistoryPersistError::Clock)?;
    let elapsed_ms = i64::try_from(elapsed_ms).unwrap_or(i64::MAX);

    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        let consent: i64 = tx.query_row(
            "SELECT history_enabled FROM preferences WHERE id=1",
            [],
            |row| row.get(0),
        )?;
        if consent != 1 {
            tx.rollback()?;
            return Ok(false);
        }
        tx.execute(
            "INSERT INTO history_sessions
             (session_uuid, timestamp, trigger_source, detected_language,
              target_language, is_favorite, source_text_encrypted,
              source_text_nonce, crypto_version)
             VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7, ?8)",
            params![
                session_uuid,
                timestamp,
                trigger_source,
                detected_language,
                target_language,
                encrypted_source.ciphertext,
                encrypted_source.nonce.as_slice(),
                encrypted_source.crypto_version,
            ],
        )?;

        for item in prepared {
            let provider_name: String = tx
                .query_row(
                    "SELECT name FROM providers WHERE uuid=?1",
                    [&item.provider_uuid],
                    |row| row.get(0),
                )
                .unwrap_or_else(|_| "Deleted provider".to_string());
            let (result_cipher, result_nonce, result_version) = item
                .result
                .as_ref()
                .map(|field| {
                    (
                        Some(field.ciphertext.as_slice()),
                        Some(field.nonce.as_slice()),
                        field.crypto_version,
                    )
                })
                .unwrap_or((None, None, crypto::HISTORY_CRYPTO_VERSION));
            let (error_cipher, error_nonce) = item
                .error
                .as_ref()
                .map(|field| {
                    (
                        Some(field.ciphertext.as_slice()),
                        Some(field.nonce.as_slice()),
                    )
                })
                .unwrap_or((None, None));
            tx.execute(
                "INSERT INTO history_results
                 (result_uuid, session_uuid, provider_uuid, provider_name_snapshot,
                  engine_id, elapsed_ms, outcome_tag, result_text_encrypted,
                  result_text_nonce, error_kind, error_message_encrypted,
                  error_message_nonce, crypto_version)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    item.uuid,
                    session_uuid,
                    item.provider_uuid,
                    provider_name,
                    item.engine_id,
                    elapsed_ms,
                    item.outcome_tag,
                    result_cipher,
                    result_nonce,
                    item.error_kind,
                    error_cipher,
                    error_nonce,
                    result_version,
                ],
            )?;
        }
        tx.commit()?;
        Ok(true)
    })
    .map_err(HistoryPersistError::Db)
}

fn history_error_kind(error: &crate::error::Error) -> &'static str {
    match error {
        crate::error::Error::FallbackEligible(_) => "fallback_eligible",
        crate::error::Error::Config(_) => "config",
        crate::error::Error::Keystore(_) => "keystore",
        crate::error::Error::LocalNoFallback => "local_no_fallback",
    }
}

use futures::future::BoxFuture;
use linguaray_kernel::{
    ActivationContext, CapabilityPlugin, KernelHandle, PluginDescriptor, PluginError, PluginId,
    ServiceId, ServiceKey,
};
use std::sync::Arc;

pub static HISTORY: ServiceKey<HistoryHub> = ServiceKey::new("linguaray.history");
static PROVIDES: &[ServiceId] = &[ServiceId("linguaray.history")];
static REQUIRED: &[ServiceId] = &[ServiceId("linguaray.database")];
static OPTIONAL: &[ServiceId] = &[ServiceId("linguaray.secrets")];

pub struct HistoryHub {
    kernel: KernelHandle,
}

impl HistoryHub {
    fn secrets(&self) -> Result<linguaray_kernel::ServiceLease<Keystore>, String> {
        self.kernel
            .optional(crate::plugins::secrets::SECRETS)
            .ok_or_else(|| "keystore unavailable: startup init failed (recovery required)".into())
    }

    pub fn set_enabled(&self, db: &Database, enabled: bool) -> Result<(), String> {
        let secrets = self.secrets()?;
        secrets
            .with(|ks| crate::db::history::set_enabled(db, ks, enabled))
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())
    }

    pub fn search(
        &self,
        db: &Database,
        query: &str,
        cursor: Option<&str>,
    ) -> Result<search::HistoryPage, String> {
        let secrets = self.secrets()?;
        secrets
            .with(|ks| search::search(db, ks, query, cursor))
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())
    }

    pub fn require_writable(&self) -> Result<(), String> {
        let _ = self.secrets()?;
        Ok(())
    }

    pub fn vocabulary_add(
        &self,
        db: &Database,
        word: &str,
        definition: &str,
        source_language: &str,
        target_language: &str,
    ) -> Result<crate::plugins::vocabulary::VocabularyItem, String> {
        let secrets = self.secrets()?;
        secrets
            .with(|ks| {
                crate::plugins::vocabulary::add_word(
                    db,
                    ks,
                    word,
                    definition,
                    source_language,
                    target_language,
                )
            })
            .map_err(|e| e.to_string())?
    }

    pub fn vocabulary_list(
        &self,
        db: &Database,
        cursor: Option<&str>,
    ) -> Result<crate::plugins::vocabulary::VocabularyPage, String> {
        let secrets = self.secrets()?;
        secrets
            .with(|ks| crate::plugins::vocabulary::list_words(db, ks, cursor))
            .map_err(|e| e.to_string())?
    }

    pub fn vocabulary_export(
        &self,
        db: &Database,
        file_path: &str,
        format: &str,
    ) -> Result<String, String> {
        let secrets = self.secrets()?;
        secrets
            .with(|ks| crate::plugins::vocabulary::export_file(db, ks, file_path, format))
            .map_err(|e| e.to_string())?
    }

    pub fn export(
        &self,
        db: &Database,
        file_path: &str,
        format: &str,
        filter: &export::HistoryFilter,
    ) -> Result<String, String> {
        let secrets = self.secrets()?;
        let fmt = export::ExportFormat::parse(format).map_err(|e| e.to_string())?;
        secrets
            .with(|ks| export::export_all(db, ks, filter))
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())
            .and_then(|sessions| {
                export::write_export_file(&sessions, std::path::Path::new(file_path), fmt)
                    .map_err(|e| e.to_string())?;
                Ok(file_path.to_string())
            })
    }
}

pub struct HistoryPlugin;

impl CapabilityPlugin for HistoryPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: PluginId("history"),
            required: REQUIRED,
            optional: OPTIONAL,
            provides: PROVIDES,
            manifest: None,
            restart_on_optional_change: false,
        }
    }

    fn config_fingerprint(&self) -> u64 {
        1
    }

    fn activate(&self, ctx: ActivationContext) -> BoxFuture<'_, Result<(), PluginError>> {
        Box::pin(async move {
            let _db = ctx.require(crate::plugins::database::DATABASE)?;
            let _ = ctx.optional(crate::plugins::secrets::SECRETS);
            ctx.stage_provide(
                HISTORY,
                Arc::new(HistoryHub {
                    kernel: ctx.handle(),
                }),
            )?;
            Ok(())
        })
    }
}
