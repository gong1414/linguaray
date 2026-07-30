//! Translate-error classification (spec §G).
//!
//! `FallbackEligible` — a transient/provider error that *may* justify falling
//!   back to another engine (network, timeout, 429, 5xx, parse failure).
//! `Config` — a configuration/auth problem that must send the user to Settings,
//!   never silently fall back (missing key, 401/403, bad model, keystore fault).
//! `LocalNoFallback` — the primary failed but no fallback was attempted (§G):
//!   either the primary was LOCAL (local-sacred forbids degrading to a remote
//!   engine) or no `fallback_engine` is configured.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("fallback-eligible: {0}")]
    FallbackEligible(#[from] FallbackKind),

    #[error("config error: {0}")]
    Config(#[from] ConfigKind),

    #[error(transparent)]
    Keystore(#[from] crate::keystore::KeystoreError),

    /// §G: no fallback was attempted. Either the primary was LOCAL (loopback)
    /// and the local-sacred rule forbids silent degradation to a remote engine,
    /// or no `fallback_engine` was configured. Surfaced to the UI as "no fallback".
    #[error("no fallback available (primary failed; local-primary sacred or no fallback configured)")]
    LocalNoFallback,
}

#[derive(Debug, Error)]
pub enum FallbackKind {
    #[error("network error: {0}")]
    Network(String),
    #[error("timeout")]
    Timeout,
    #[error("provider returned {status}")]
    ProviderStatus { status: u16 },
    #[error("response parse failed: {0}")]
    Parse(String),
}

#[derive(Debug, Error)]
pub enum ConfigKind {
    #[error("no API key set for provider {provider}")]
    MissingKey { provider: String },
    #[error("auth failed ({status}) for {provider}")]
    AuthFailed { provider: String, status: u16 },
    #[error("invalid model {model} for {provider}")]
    InvalidModel { provider: String, model: String },
    /// §G: a 4xx other than 401/403 (400/404/422/...) — bad request / invalid
    /// model / wrong endpoint. NOT fallback-eligible: retrying with a 2nd
    /// provider would needlessly send the text elsewhere.
    #[error("invalid request ({status}) for {provider}")]
    InvalidRequest { provider: String, status: u16 },
}
