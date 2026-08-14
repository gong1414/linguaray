//! Field-level encryption for history and vocabulary records.
//!
//! Every encrypted field uses AES-256-GCM with a fresh 96-bit nonce and
//! domain-separated AAD that binds the ciphertext to its record UUID and field.

use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use rand::{rngs::OsRng, RngCore};
use thiserror::Error;

pub const HISTORY_CRYPTO_VERSION: u32 = 1;
pub const HISTORY_NONCE_LEN: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryField<'a> {
    SessionSource { uuid: &'a str },
    ResultText { uuid: &'a str },
    ResultError { uuid: &'a str },
    VocabularyWord { uuid: &'a str },
    VocabularyDefinition { uuid: &'a str },
}

impl HistoryField<'_> {
    pub fn aad(&self) -> Vec<u8> {
        let aad = match self {
            Self::SessionSource { uuid } => {
                format!("linguaray-history-v1|session|{uuid}|source")
            }
            Self::ResultText { uuid } => {
                format!("linguaray-history-v1|result|{uuid}|text")
            }
            Self::ResultError { uuid } => {
                format!("linguaray-history-v1|result|{uuid}|error")
            }
            Self::VocabularyWord { uuid } => {
                format!("linguaray-vocab-v1|item|{uuid}|word")
            }
            Self::VocabularyDefinition { uuid } => {
                format!("linguaray-vocab-v1|item|{uuid}|definition")
            }
        };
        aad.into_bytes()
    }

    fn has_identifier(&self) -> bool {
        match self {
            Self::SessionSource { uuid }
            | Self::ResultText { uuid }
            | Self::ResultError { uuid }
            | Self::VocabularyWord { uuid }
            | Self::VocabularyDefinition { uuid } => !uuid.is_empty(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptedField {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; HISTORY_NONCE_LEN],
    pub crypto_version: u32,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum HistoryCryptoError {
    #[error("history field identifier must not be empty")]
    EmptyIdentifier,
    #[error("unsupported history crypto version {0}")]
    UnsupportedVersion(u32),
    #[error("history encryption failed")]
    Encrypt,
    #[error("history authentication failed")]
    Authentication,
}

pub fn encrypt_field(
    key: &[u8; 32],
    field: &HistoryField<'_>,
    plaintext: &[u8],
) -> Result<EncryptedField, HistoryCryptoError> {
    if !field.has_identifier() {
        return Err(HistoryCryptoError::EmptyIdentifier);
    }

    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| HistoryCryptoError::Encrypt)?;
    let mut nonce = [0_u8; HISTORY_NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    let aad = field.aad();
    let ciphertext = cipher
        .encrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: plaintext,
                aad: &aad,
            },
        )
        .map_err(|_| HistoryCryptoError::Encrypt)?;

    Ok(EncryptedField {
        ciphertext,
        nonce,
        crypto_version: HISTORY_CRYPTO_VERSION,
    })
}

pub fn decrypt_field(
    key: &[u8; 32],
    field: &HistoryField<'_>,
    encrypted: &EncryptedField,
) -> Result<Vec<u8>, HistoryCryptoError> {
    if !field.has_identifier() {
        return Err(HistoryCryptoError::EmptyIdentifier);
    }
    if encrypted.crypto_version != HISTORY_CRYPTO_VERSION {
        return Err(HistoryCryptoError::UnsupportedVersion(
            encrypted.crypto_version,
        ));
    }

    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| HistoryCryptoError::Authentication)?;
    let aad = field.aad();
    cipher
        .decrypt(
            Nonce::from_slice(&encrypted.nonce),
            Payload {
                msg: &encrypted.ciphertext,
                aad: &aad,
            },
        )
        .map_err(|_| HistoryCryptoError::Authentication)
}
