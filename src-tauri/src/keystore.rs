//! Self-encrypted key store — spec §A.
//!
//! AES-256-GCM with an Argon2id key derived from machine identity. Versioned
//! envelope. Cross-process lock + atomic replace. Fail-closed on any fault.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum KeystoreError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("envelope: {0}")]
    Envelope(String),
    #[error("crypto: {0}")]
    Crypto(String),
    #[error("auth failed (wrong identity / tampered)")]
    AuthFailed,
    #[error("unsupported version {0}")]
    UnsupportedVersion(u64),
    #[error("acl: {0}")]
    Acl(#[from] crate::fs_acl::AclError),
}

/// Where the machine identity comes from. Recorded at creation; frozen for the
/// life of the file (spec §A: never auto-switch source).
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IdentitySource {
    MacosIoplatformuuid,
    WindowsMachineguid,
}

impl IdentitySource {
    #[cfg(target_os = "macos")]
    const CURRENT: IdentitySource = IdentitySource::MacosIoplatformuuid;
    #[cfg(target_os = "windows")]
    const CURRENT: IdentitySource = IdentitySource::WindowsMachineguid;

    /// Read + normalize the identity string. Fail-closed if unavailable (no weak
    /// fallback — spec §A). The body is cfg-gated per platform so each platform's
    /// impl references only that platform's helper (otherwise the Windows arm
    /// would reference a function absent on macOS, and vice versa).
    #[cfg(target_os = "macos")]
    pub fn read(&self) -> Result<String, KeystoreError> {
        let raw = match self {
            IdentitySource::MacosIoplatformuuid => read_macos_io_platform_uuid()?,
            IdentitySource::WindowsMachineguid => {
                return Err(KeystoreError::Envelope(
                    "WindowsMachineguid not readable on macOS".into(),
                ))
            }
        };
        Ok(raw.trim().to_lowercase())
    }

    #[cfg(target_os = "windows")]
    pub fn read(&self) -> Result<String, KeystoreError> {
        let raw = match self {
            IdentitySource::WindowsMachineguid => read_windows_machine_guid()?,
            IdentitySource::MacosIoplatformuuid => {
                return Err(KeystoreError::Envelope(
                    "MacosIoplatformuuid not readable on Windows".into(),
                ))
            }
        };
        Ok(raw.trim().to_lowercase())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    pub fn read(&self) -> Result<String, KeystoreError> {
        Err(KeystoreError::Envelope("unsupported platform".into()))
    }
}

#[cfg(target_os = "macos")]
fn read_macos_io_platform_uuid() -> Result<String, KeystoreError> {
    use std::process::Command;
    let out = Command::new("ioreg")
        .args(["-d2", "-c", "IOPlatformExpertDevice"])
        .output()
        .map_err(|e| KeystoreError::Envelope(format!("ioreg spawn: {e}")))?;
    let s = String::from_utf8_lossy(&out.stdout);
    s.lines()
        .find(|l| l.contains("\"IOPlatformUUID\""))
        .and_then(|l| l.split('=').nth(1))
        .map(|v| v.trim().trim_matches('"').to_string())
        .ok_or_else(|| KeystoreError::Envelope("IOPlatformUUID not found".into()))
}

#[cfg(target_os = "windows")]
fn read_windows_machine_guid() -> Result<String, KeystoreError> {
    use std::process::Command;
    let out = Command::new("reg")
        .args(["query", "HKLM\\SOFTWARE\\Microsoft\\Cryptography", "/v", "MachineGuid"])
        .output()
        .map_err(|e| KeystoreError::Envelope(format!("reg spawn: {e}")))?;
    let s = String::from_utf8_lossy(&out.stdout);
    s.lines()
        .find(|l| l.contains("MachineGuid"))
        .and_then(|l| l.split("REG_SZ").nth(1))
        .map(|v| v.trim().to_string())
        .ok_or_else(|| KeystoreError::Envelope("MachineGuid not found".into()))
}

use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce};
use argon2::Argon2;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const DOMAIN_SEPARATOR: &[u8] = b"linguaray-keystore-v1\0";
const FIXED_AAD: &[u8] = b"linguaray-keystore-envelope-v1";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const VERSION: u64 = 1;

/// Pinned Argon2id params (spec §A): m=64MiB, t=3, p=1, out=32.
fn argon2() -> Argon2<'static> {
    Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(65536, 3, 1, Some(32)).unwrap(),
    )
}

/// Derive the 32-byte AES key. `salt` is used ONLY as the Argon2 salt.
fn derive_key(identity: &str, salt: &[u8]) -> [u8; 32] {
    let mut password = DOMAIN_SEPARATOR.to_vec();
    password.extend_from_slice(identity.as_bytes());
    let argon = argon2();
    let mut out = [0u8; 32];
    argon.hash_password_into(&password, salt, &mut out)
        .expect("argon2 with valid params");
    zeroize::Zeroize::zeroize(&mut password);
    out
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Envelope {
    pub version: u64,
    pub aead: String,
    pub kdf: String,
    pub kdf_params: KdfParams,
    pub identity_source: IdentitySource,
    pub salt: String,    // base64
    pub nonce: String,   // base64
    pub ciphertext: String, // base64 (includes appended 16-byte GCM tag)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KdfParams {
    pub m_kib: u32,
    pub t: u32,
    pub p: u32,
    pub output_len: u32,
}

const PINNED_KDF: KdfParams = KdfParams { m_kib: 65536, t: 3, p: 1, output_len: 32 };

// ── Versioned inner payload (spec §A, S2a) ────────────────────────────
//
// The outer crypto envelope (Envelope above) encrypts a `serde_json::Value`.
// KeystoreData is the versioned INNER structure that value is (de)serialized
// to/from. v1 keystore payloads were a flat map {provider_id: key}; v2 wraps
// that map with explicit fields for future history/external-token opt-ins.

/// Version of the KeystoreData inner payload. Bumped on breaking schema changes.
pub const KEYSTORE_DATA_VERSION: u32 = 2;

/// Serializable form of a symmetric key (opt-in future fields). Not populated
/// in S2a; kept as a placeholder so history_key can be added without a second
/// schema bump. Tuple struct so the JSON shape stays a flat array of 32 ints.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SerializableKey(pub [u8; 32]);

/// Versioned inner keystore payload (spec §A, S2a).
///
/// Serialized to a `serde_json::Value` BEFORE the outer `encrypt` call, and
/// deserialized FROM the value `decrypt` returns. The crypto layer is
/// unchanged — KeystoreData only shapes the plaintext.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct KeystoreData {
    pub version: u32,
    pub provider_keys: HashMap<String, String>,
    /// Opt-in; not populated in S2a.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history_key: Option<SerializableKey>,
    /// Opt-in; not populated in S2a.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_api_token: Option<String>,
}

impl KeystoreData {
    /// Build a fresh v2 payload carrying just the given provider keys.
    pub fn new_v2(provider_keys: HashMap<String, String>) -> Self {
        Self { version: KEYSTORE_DATA_VERSION, provider_keys, history_key: None, external_api_token: None }
    }

    /// Serialize to a serde_json::Value (the shape `encrypt` consumes).
    pub fn to_value(&self) -> Result<serde_json::Value, KeystoreError> {
        serde_json::to_value(self).map_err(|e| KeystoreError::Envelope(e.to_string()))
    }

    /// Deserialize from a serde_json::Value (the shape `decrypt` returns).
    pub fn from_value(v: &serde_json::Value) -> Result<Self, KeystoreError> {
        serde_json::from_value(v.clone()).map_err(|e| KeystoreError::Envelope(e.to_string()))
    }

    /// Get a provider key by secret_ref.
    pub fn get_provider_key(&self, secret_ref: &str) -> Option<&str> {
        self.provider_keys.get(secret_ref).map(|s| s.as_str())
    }

    /// Set a provider key by secret_ref.
    pub fn set_provider_key(&mut self, secret_ref: impl Into<String>, key: impl Into<String>) {
        self.provider_keys.insert(secret_ref.into(), key.into());
    }

    /// Remove a provider key by secret_ref; returns the removed value.
    pub fn remove_provider_key(&mut self, secret_ref: &str) -> Option<String> {
        self.provider_keys.remove(secret_ref)
    }
}

/// Outcome of inspecting the keystore file on disk (spec §A, S2a). Replaces the
/// ambiguous `{}` that the old `load()` returned for both "missing" and "empty"
/// so callers can drive migration vs fresh-install vs corrupt-recovery.
#[derive(Debug)]
pub enum KeystoreLoadState {
    /// File doesn't exist — fresh install.
    Missing,
    /// Pre-v2 flat map `{key: value}` (no `version` field) — needs migration.
    LegacyV1(HashMap<String, String>),
    /// Versioned v2 payload.
    CurrentV2(KeystoreData),
    /// File exists but couldn't be read/decrypted/parsed — fail-closed.
    Corrupt(KeystoreError),
}

/// Does a decrypted payload carry a `version` field? A v1 flat map has none.
fn has_version_field(v: &serde_json::Value) -> bool {
    v.get("version").is_some()
}

/// The allowlist of top-level keys a versioned v2 payload may carry. Any other
/// top-level key means the payload is a mixed v1/v2 structure (a v1 flat map
/// merged with a v2 envelope, or a forward-incompatible v3 leak) and must be
/// treated as Corrupt so the user is sent to recovery instead of silently
/// losing data.
const V2_ALLOWED_KEYS: &[&str] =
    &["version", "provider_keys", "history_key", "external_api_token"];

/// Classify an already-decrypted payload into LegacyV1 / CurrentV2 / Corrupt.
/// Shared between the production path (decrypt with machine identity) and the
/// test path (decrypt with an injected identity).
///
/// A versioned payload is only accepted as CurrentV2 when:
/// 1. `version == KEYSTORE_DATA_VERSION` (any other value, present or missing
///    type, is Corrupt — we never silently migrate an unknown version forward),
/// 2. the top-level object carries ONLY keys from [`V2_ALLOWED_KEYS`] (extra
///    keys ⇒ a mixed v1/v2 structure ⇒ Corrupt, so the recovery banner fires
///    rather than the runtime normalization dropping keys on the floor).
fn classify_payload(payload: &serde_json::Value) -> Result<KeystoreLoadState, KeystoreError> {
    if has_version_field(payload) {
        // Versioned payload: validate version + key shape BEFORE handing to
        // KeystoreData::from_value (which would happily ignore unknown fields).
        let obj = payload.as_object().ok_or_else(|| {
            KeystoreError::Envelope("versioned payload is not a JSON object".into())
        })?;
        let version = obj.get("version").and_then(|v| v.as_u64()).ok_or_else(|| {
            KeystoreError::Envelope("versioned payload has non-integer 'version'".into())
        })?;
        if version as u32 != KEYSTORE_DATA_VERSION {
            // Unknown/unsupported version → Corrupt (never auto-upgrade).
            return Ok(KeystoreLoadState::Corrupt(KeystoreError::Envelope(format!(
                "unsupported keystore data version: got {version}, expected {KEYSTORE_DATA_VERSION}"
            ))));
        }
        // Reject extra top-level keys (mixed v1/v2 or a forward-incompatible v3).
        for key in obj.keys() {
            if !V2_ALLOWED_KEYS.contains(&key.as_str()) {
                return Ok(KeystoreLoadState::Corrupt(KeystoreError::Envelope(format!(
                    "unknown top-level key in versioned keystore payload: '{key}'"
                ))));
            }
        }
        // version + key shape are valid → KeystoreData. A structurally wrong
        // nested value (e.g. provider_keys not an object) still surfaces as Corrupt.
        match KeystoreData::from_value(payload) {
            Ok(data) => Ok(KeystoreLoadState::CurrentV2(data)),
            Err(e) => Ok(KeystoreLoadState::Corrupt(e)),
        }
    } else {
        // Legacy v1 flat map {key: value}. A non-object payload here is treated
        // as Corrupt (the v1 shape was always an object map).
        let map: HashMap<String, String> = serde_json::from_value(payload.clone())
            .map_err(|e| KeystoreError::Envelope(format!("legacy v1 parse: {e}")))?;
        Ok(KeystoreLoadState::LegacyV1(map))
    }
}

/// Normalize a decrypted keystore payload into a v2 [`KeystoreData`], no matter
/// which on-disk shape it currently has (spec §A, S2a P0).
///
/// The runtime translate path and every typed accessor (`Keystore::get_key`,
/// `set_key`, ...) go through here so they all agree on where a provider key
/// lives — a single fixpoint instead of N reinvented v1-vs-v2 lookups.
///
/// **Fail-closed (P0):** reuses [`classify_payload`] so the typed accessors apply
/// the SAME strict classification as the dedicated `load_state` path. Any payload
/// [`classify_payload`] marks `Corrupt` — unsupported version, mixed v1/v2 shape,
/// malformed `provider_keys`, a non-object without a `version` field, ... —
/// propagates as `Err` here. The caller therefore must NOT write a normalized
/// empty v2 over authenticated contents; it bails out before touching the file.
///
/// Shapes accepted (`Ok`):
/// - **v2** (`{"version":2,"provider_keys":{...}}`): deserialized directly.
/// - **v1 flat map** (`{"openai":"sk-..."}` with no `version`): upgraded to a v2
///   payload carrying the same keys. `secret_ref == legacy key name`.
/// - **Empty object `{}`** (fresh-install decrypt): upgraded to an empty v2
///   payload, so the first `set_key` on a brand-new install lands as v2. This is
///   the ONLY legitimate empty case.
///
/// Shapes rejected (`Err`):
/// - Versioned but `version != KEYSTORE_DATA_VERSION` → `Err`.
/// - Versioned v2 with extra top-level keys (mixed v1/v2) → `Err`.
/// - Versioned v2 with a malformed `provider_keys` (e.g. not an object) → `Err`.
/// - Non-object without a `version` field (`null`, array, string, ...) → `Err`.
///
/// `history_key` / `external_api_token` are preserved when reading a v2 payload
/// and left `None` when upgrading from v1 (those opt-in fields didn't exist in v1).
fn payload_to_v2(v: &serde_json::Value) -> Result<KeystoreData, KeystoreError> {
    // Delegate to classify_payload so this path and the dedicated load_state path
    // can never drift apart on what counts as Corrupt. A Corrupt classification
    // becomes Err here (fail-closed); CurrentV2/LegacyV1 are unwrapped to a v2
    // KeystoreData.
    match classify_payload(v)? {
        KeystoreLoadState::CurrentV2(data) => Ok(data),
        KeystoreLoadState::LegacyV1(map) => Ok(KeystoreData::new_v2(map)),
        // classify_payload classifies a VALUE, never a file, so it cannot return
        // Missing in practice; treat a defensive Missing as a fresh empty payload.
        KeystoreLoadState::Missing => Ok(KeystoreData::new_v2(HashMap::new())),
        KeystoreLoadState::Corrupt(e) => Err(e),
    }
}

/// Encrypt a keys map into an envelope (fresh salt + nonce each call).
pub fn encrypt(identity: &str, identity_source: IdentitySource, keys: &serde_json::Value) -> Result<Envelope, KeystoreError> {
    let mut salt = [0u8; SALT_LEN];
    let mut nonce = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut nonce);
    let key = derive_key(identity, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| KeystoreError::Crypto(e.to_string()))?;
    let plaintext = serde_json::to_vec(keys).map_err(|e| KeystoreError::Envelope(e.to_string()))?;
    let ct = cipher.encrypt(Nonce::from_slice(&nonce), aes_gcm::aead::Payload { msg: &plaintext, aad: FIXED_AAD })
        .map_err(|e| KeystoreError::Crypto(e.to_string()))?;
    Ok(Envelope {
        version: VERSION, aead: "aes-256-gcm".into(), kdf: "argon2id".into(),
        kdf_params: PINNED_KDF, identity_source,
        salt: B64.encode(salt), nonce: B64.encode(nonce), ciphertext: B64.encode(ct),
    })
}

/// Validate the envelope header against the whitelist BEFORE decryption, then
/// decrypt. A tampered `kdf_params` is rejected outright (DoS guard) — never honored.
pub fn decrypt(envelope: &Envelope, machine_source: IdentitySource) -> Result<serde_json::Value, KeystoreError> {
    if machine_source != IdentitySource::CURRENT {
        // Historical callers passed an explicit source; the current pin is the
        // platform's CURRENT source. Only CURRENT is honored (the envelope's
        // recorded source is still checked inside decrypt_with).
        return Err(KeystoreError::AuthFailed);
    }
    decrypt_with(envelope, Identity::Machine)
}

#[doc(hidden)]
/// Test-only: decrypt with an explicit identity string instead of reading the
/// machine. Lets tests drive the crypto without touching real OS identity.
pub fn decrypt_with_identity(envelope: &Envelope, identity: &str) -> Result<serde_json::Value, KeystoreError> {
    decrypt_with(envelope, Identity::Injected(identity))
}

use std::path::{Path, PathBuf};
use parking_lot::Mutex;

/// Where the identity used to (de)crypt comes from. The single axis along which
/// the production path and the test seam differ — every other piece of locked
/// read / backup / rewrite logic is shared via the `*_locked_core` functions
/// below, so tests exercise the SAME core as production (no duplicated logic
/// that could drift and produce false greens).
///
/// `Machine`  → read the real OS identity via `IdentitySource::CURRENT` (and
///              validate the envelope's `identity_source` against it). Production.
/// `Injected` → use the given string verbatim (no source check). Test only.
#[derive(Clone, Copy)]
enum Identity<'a> {
    Machine,
    Injected(&'a str),
}

impl<'a> Identity<'a> {
    /// Validate the envelope header + decrypt, branching on the identity source.
    /// `Machine` validates `identity_source == CURRENT` and reads the OS identity;
    /// `Injected` skips the source check and uses the injected string. All other
    /// validation (version/aead/kdf/lengths) is identical to the public `decrypt`
    /// and `decrypt_with_identity` paths — those now delegate here too.
    fn decrypt_for(&self, envelope: &Envelope) -> Result<serde_json::Value, KeystoreError> {
        decrypt_with(envelope, *self)
    }
}

/// Validate the envelope header against the pinned whitelist, then decrypt using
/// the identity selected by `id`. The single crypto entry point for both the
/// production (`Machine`) and test (`Injected`) paths. The public `decrypt` and
/// `decrypt_with_identity` are thin facades over this.
fn decrypt_with(envelope: &Envelope, id: Identity<'_>) -> Result<serde_json::Value, KeystoreError> {
    if envelope.version != VERSION { return Err(KeystoreError::UnsupportedVersion(envelope.version)); }
    if envelope.aead != "aes-256-gcm" || envelope.kdf != "argon2id" {
        return Err(KeystoreError::Envelope("bad aead/kdf".into()));
    }
    if envelope.kdf_params.m_kib != PINNED_KDF.m_kib
        || envelope.kdf_params.t != PINNED_KDF.t
        || envelope.kdf_params.p != PINNED_KDF.p
        || envelope.kdf_params.output_len != PINNED_KDF.output_len
    {
        return Err(KeystoreError::Envelope("kdf_params not pinned".into()));
    }
    // Identity resolution. Production validates the recorded source against the
    // current platform's source AND reads the real OS identity; tests inject the
    // string directly and skip the source check (the envelope was sealed with a
    // known test identity regardless of its recorded source field).
    let identity = match id {
        Identity::Machine => {
            if envelope.identity_source != IdentitySource::CURRENT {
                return Err(KeystoreError::AuthFailed);
            }
            IdentitySource::CURRENT.read()?
        }
        Identity::Injected(s) => s.to_string(),
    };
    let salt = B64.decode(&envelope.salt).map_err(|e| KeystoreError::Envelope(e.to_string()))?;
    let nonce = B64.decode(&envelope.nonce).map_err(|e| KeystoreError::Envelope(e.to_string()))?;
    let ct = B64.decode(&envelope.ciphertext).map_err(|e| KeystoreError::Envelope(e.to_string()))?;
    if salt.len() != SALT_LEN || nonce.len() != NONCE_LEN || ct.len() < 16 {
        return Err(KeystoreError::Envelope("bad field lengths".into()));
    }
    let key = derive_key(&identity, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| KeystoreError::Crypto(e.to_string()))?;
    let pt = cipher.decrypt(Nonce::from_slice(&nonce), aes_gcm::aead::Payload { msg: &ct, aad: FIXED_AAD })
        .map_err(|_| KeystoreError::AuthFailed)?;
    serde_json::from_slice(&pt).map_err(|e| KeystoreError::Envelope(e.to_string()))
}

/// Owns the keystore directory + in-process lock + cross-process sidecar lock.
///
/// Per spec §A: every load-modify-store and the stale-tmp cleanup run under BOTH
/// the in-process `Mutex` (serializes writers within this Tauri process) AND the
/// cross-process `keystore.lock` sidecar (held via fs2 flock for the duration of the
/// RMW). `tauri-plugin-single-instance` is defense-in-depth; it does NOT replace
/// this sidecar — if a second instance ever starts (or an external tool touches the
/// file), the sidecar is what serializes them.
pub struct Keystore {
    dir: PathBuf,
    in_proc: Mutex<()>,
}

const FILE: &str = "keystore.json";
const TMP: &str = "keystore.json.tmp";
// Sidecar lock file for the cross-process flock (fs2). Lives in self.dir.
const LOCK: &str = "keystore.lock";

impl Keystore {
    pub fn new(dir: PathBuf) -> Result<Self, KeystoreError> {
        std::fs::create_dir_all(&dir)?;
        Self::set_dir_perms(&dir)?;
        // Round-2 review P1 #3: do NOT delete a stale .tmp here — this runs BEFORE
        // the cross-process lock, so a second instance could remove a tmp the first
        // is actively writing. Stale-tmp cleanup happens UNDER the lock, inside
        // every write path (update_keys) — see with_locks/​update_keys. Construction
        // only prepares the dir + perms.
        Ok(Self { dir, in_proc: Mutex::new(()) })
    }

    fn set_dir_perms(dir: &Path) -> Result<(), KeystoreError> {
        Ok(crate::fs_acl::secure_dir(dir)?)
    }

    fn file(&self) -> PathBuf { self.dir.join(FILE) }

    /// Run `body` under BOTH the in-proc mutex AND an exclusive flock on
    /// self.dir/keystore.lock. The flock File is opened per-call and bound to THIS
    /// keystore's dir (so different dirs don't share a lock path); it's held alive
    /// for the whole critical section, then unlocked + dropped under the mutex.
    /// Per §A this serializes a second instance or external tool on the same dir;
    /// single-instance is only defense-in-depth.
    fn with_locks<R, F>(&self, body: F) -> Result<R, KeystoreError>
    where
        F: FnOnce(&Self) -> Result<R, KeystoreError>,
    {
        use fs2::FileExt;
        let _inproc = self.in_proc.lock();
        // Open THIS dir's sidecar lock file and acquire an exclusive flock.
        let lock_file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false) // sidecar lock: never truncate (clippy + correctness)
            .open(self.dir.join(LOCK))?;
        lock_file.lock_exclusive().map_err(KeystoreError::Io)?;
        // Stale-tmp cleanup UNDER the lock (round-2 review P1 #3): every write path
        // goes through with_locks, so a stale keystore.json.tmp from a prior crash is
        // removed here — safely, with no other instance able to touch it.
        let tmp = self.dir.join(TMP);
        if tmp.exists() {
            let _ = std::fs::remove_file(&tmp);
        }
        // Hold lock_file alive across the critical section.
        let result = body(self);
        // Release (unlock + drop). Errors here are not fatal to the result.
        let _ = lock_file.unlock();
        drop(lock_file);
        result
    }

    /// Read the whole keys map. Returns `{}` if the file does not exist (first run).
    pub fn load(&self) -> Result<serde_json::Value, KeystoreError> {
        self.with_locks(|ks| ks.load_locked())
    }

    /// Locked read (caller holds BOTH locks). Used by with_locks bodies. Reads
    /// the machine identity via `IdentitySource::CURRENT`.
    fn load_locked(&self) -> Result<serde_json::Value, KeystoreError> {
        self.load_locked_core(Identity::Machine)
    }

    /// THE locked read core (caller holds BOTH locks). Reads the file and
    /// decrypts with the identity selected by `id`. Both the production
    /// [`load_locked`](Self::load_locked) (`Identity::Machine`) and the
    /// [`update_keys_with_identity`](Self::update_keys_with_identity) test seam
    /// (`Identity::Injected`) go through here — no duplicated read/decrypt logic
    /// to drift.
    fn load_locked_core(&self, id: Identity<'_>) -> Result<serde_json::Value, KeystoreError> {
        let path = self.file();
        if !path.exists() { return Ok(serde_json::json!({})); }
        let bytes = std::fs::read(&path)?;
        let env: Envelope = serde_json::from_slice(&bytes)
            .map_err(|e| KeystoreError::Envelope(format!("malformed: {e}")))?;
        id.decrypt_for(&env)
    }

    /// Inspect the keystore and return a typed load state (S2a). Same decrypt
    /// path as `load()`/`load_locked()` (reads the machine identity via
    /// `IdentitySource::CURRENT`), but distinguishes Missing / LegacyV1 /
    /// CurrentV2 / Corrupt instead of collapsing all non-object results to `{}`.
    /// Returns `Missing` WITHOUT taking the flock when the file is absent, so a
    /// fresh-install probe never contends with a writer on the same dir.
    pub fn load_state(&self) -> KeystoreLoadState {
        self.load_state_with(Identity::Machine)
    }

    /// Test-only: same as [`load_state`](Self::load_state) but decrypts with an
    /// explicit identity string instead of reading the machine. Lets tests drive
    /// the classification without touching real OS identity. Delegates to the SAME
    /// locked-read core as the production path (no duplicated read/decrypt logic).
    #[doc(hidden)]
    pub fn load_state_with_identity(&self, identity: &str) -> KeystoreLoadState {
        self.load_state_with(Identity::Injected(identity))
    }

    /// Shared front end for both `load_state` (production) and
    /// `load_state_with_identity` (test). The fast-path Missing check is done
    /// OUTSIDE the flock (fresh-install probe never contends with a writer); the
    /// actual read + decrypt + classify happens under BOTH locks via the core.
    fn load_state_with(&self, id: Identity<'_>) -> KeystoreLoadState {
        // Fast path: if the file doesn't exist, there's nothing to lock on.
        // Checking first (without locks) is safe — absence is monotonic for our
        // usage and Missing-vs-the-rest is what callers branch on. If a file
        // appears between this check and the locked read below, the locked read
        // observes it; the worst case is one redundant Missing→Corrupt transition
        // which never happens in practice (single app instance + flock).
        if !self.file().exists() {
            return KeystoreLoadState::Missing;
        }
        match self.with_locks(|ks| ks.load_state_locked_core(id)) {
            Ok(state) => state,
            Err(e) => KeystoreLoadState::Corrupt(e),
        }
    }

    /// THE locked classification core (caller holds BOTH locks). Reads the file,
    /// decrypts with the identity selected by `id`, and classifies the payload
    /// into Missing / LegacyV1 / CurrentV2. Both the production path
    /// (`Identity::Machine`) and the test seam (`Identity::Injected`) go through
    /// here — no duplicated read/decrypt logic to drift. Any
    /// read/decrypt/parse failure is returned as `Err` so the public wrapper can
    /// fold it into `Corrupt`.
    fn load_state_locked_core(&self, id: Identity<'_>) -> Result<KeystoreLoadState, KeystoreError> {
        let path = self.file();
        // Re-check absence under the lock (race with a concurrent reset/archive).
        if !path.exists() { return Ok(KeystoreLoadState::Missing); }
        let bytes = std::fs::read(&path)?;
        let env: Envelope = serde_json::from_slice(&bytes)
            .map_err(|e| KeystoreError::Envelope(format!("malformed: {e}")))?;
        let payload = id.decrypt_for(&env)?;
        classify_payload(&payload)
    }

    /// Encrypt + atomically write the keys map. Takes both locks.
    pub fn store(&self, keys: &serde_json::Value) -> Result<(), KeystoreError> {
        // Clone to satisfy the closure borrow (keys: &Value → owned for the body).
        let keys = keys.clone();
        self.with_locks(|ks| ks.store_locked(&keys))
    }

    /// Locked write (caller holds BOTH locks). Used by with_locks bodies. Reads
    /// the machine identity and delegates to [`store_locked_core`](Self::store_locked_core).
    fn store_locked(&self, keys: &serde_json::Value) -> Result<(), KeystoreError> {
        self.store_locked_core(keys, Identity::Machine)
    }

    /// THE locked write core (caller holds BOTH locks). Encrypts with the identity
    /// selected by `id` and atomically replaces the keystore file. The production
    /// [`store_locked`](Self::store_locked) and the migration test seam both go
    /// through here — no duplicated encrypt/atomic-replace logic to drift.
    fn store_locked_core(&self, keys: &serde_json::Value, id: Identity<'_>) -> Result<(), KeystoreError> {
        // Production reads the real OS identity + records CURRENT as the source.
        // Tests inject the identity string and record the macOS source (the test
        // identity is the same on every host regardless of the platform it runs
        // on, so the recorded source only matters for the production path).
        let (identity, source) = match id {
            Identity::Machine => (IdentitySource::CURRENT.read()?, IdentitySource::CURRENT),
            Identity::Injected(s) => (s.to_string(), IdentitySource::MacosIoplatformuuid),
        };
        let env = encrypt(&identity, source, keys)?;
        let tmp = self.dir.join(TMP);
        let json = serde_json::to_vec(&env).map_err(|e| KeystoreError::Envelope(e.to_string()))?;
        std::fs::write(&tmp, &json)?;
        self.set_file_perms(&tmp)?;
        atomic_replace(&tmp, &self.file())?;
        Ok(())
    }

    /// Atomic read-modify-write under BOTH locks + stale-tmp cleanup.
    /// This is the ONLY sanctioned way to mutate the keystore — `set_key`/
    /// `delete_key` that do load() then a separate store() interleave and clobber.
    pub fn update_keys<F>(&self, mutator: F) -> Result<(), KeystoreError>
    where
        F: FnOnce(&mut serde_json::Value),
    {
        self.update_keys_core(mutator, Identity::Machine)
    }

    /// THE RMW core (caller's mutator runs under BOTH locks via `with_locks`).
    /// Shared by the production [`update_keys`](Self::update_keys) (machine
    /// identity) and the [`update_keys_with_identity`](Self::update_keys_with_identity)
    /// test seam (injected identity) — no duplicated load/mutate/store logic to
    /// drift. Stale-tmp cleanup happens once in `with_locks` under the lock.
    fn update_keys_core<F>(
        &self,
        mutator: F,
        id: Identity<'_>,
    ) -> Result<(), KeystoreError>
    where
        F: FnOnce(&mut serde_json::Value),
    {
        self.with_locks(|ks| {
            let mut keys = ks.load_locked_core(id)?;
            if !keys.is_object() {
                keys = serde_json::json!({});
            }
            mutator(&mut keys);
            ks.store_locked_core(&keys, id)
        })
    }

    /// Test-only: same as [`update_keys`](Self::update_keys) but the load +
    /// store inside the RMW use an injected identity string instead of reading
    /// the machine identity. Lets tests exercise the sanctioned atomic RMW
    /// (delete-resume, key removal) without touching real OS identity. Delegates
    /// to the SAME `update_keys_core` as the production path.
    #[doc(hidden)]
    pub fn update_keys_with_identity<F>(
        &self,
        mutator: F,
        identity: &str,
    ) -> Result<(), KeystoreError>
    where
        F: FnOnce(&mut serde_json::Value),
    {
        self.update_keys_core(mutator, Identity::Injected(identity))
    }

    // ── Typed KeystoreData accessors (spec §A, S2a P0) ──────────────────────
    //
    // The legacy `update_keys`/`load` surface hands callers a raw
    // `serde_json::Value`, so each call site reinvented the v1-vs-v2 lookup
    // (and the runtime translate path read `keys[preset.id]` against the flat
    // map, returning None after v2 migration). These typed accessors are the
    // single source of truth: every read goes through `KeystoreData` and every
    // write converges the on-disk payload to v2, so callers can't accidentally
    // create a mixed v1/v2 structure.
    //
    // Every mutating accessor runs through the SAME locked RMW core as
    // `update_keys` (in-process Mutex + fs2 flock), so atomicity is preserved.
    // The shared `update_data_core` normalizes the loaded value to v2 BEFORE
    // handing a typed `&mut KeystoreData` to the mutator and stores the v2
    // value back — a fresh install (load returns `{}`) therefore lands as a
    // proper v2 payload, not a v1 flat map.

    /// Read a provider key by `secret_ref`. Returns `Ok(None)` when the key is
    /// absent (including a fresh-install keystore with no keys yet). Handles
    /// both on-disk shapes (v1 flat map and v2 nested `provider_keys`), so the
    /// runtime translate path works whether or not migration has run.
    pub fn get_key(&self, secret_ref: &str) -> Result<Option<String>, KeystoreError> {
        self.get_key_with(secret_ref, Identity::Machine)
    }

    /// Set a provider key by `secret_ref`, atomically. The payload is normalized
    /// to v2 first (a v1 flat map is upgraded in place), so every write lands as
    /// the versioned shape — including the very first key on a fresh install.
    pub fn set_key(&self, secret_ref: &str, key: &str) -> Result<(), KeystoreError> {
        self.set_key_with(secret_ref, key, Identity::Machine)
    }

    /// Remove a provider key by `secret_ref`, atomically. Idempotent: removing
    /// an absent key is a successful no-op. The payload is normalized to v2.
    pub fn delete_key(&self, secret_ref: &str) -> Result<(), KeystoreError> {
        self.delete_key_with(secret_ref, Identity::Machine)
    }

    /// Does a key for `secret_ref` exist? `Ok(false)` for a fresh-install or an
    /// unset provider. Handles both v1 and v2 on-disk shapes.
    pub fn key_status(&self, secret_ref: &str) -> Result<bool, KeystoreError> {
        Ok(self.get_key(secret_ref)?.is_some())
    }

    /// Enumerate every `secret_ref` currently stored. Used by the frontend's
    /// "which providers have a key set?" probe (`key_status` Tauri command),
    /// which previously iterated the raw flat map. Reads the typed payload so a
    /// v2 keystore reports `provider_keys` (and a v1 keystore still reports its
    /// flat keys). The order is unspecified (HashMap iteration order).
    pub fn list_provider_key_refs(&self) -> Result<Vec<String>, KeystoreError> {
        self.list_provider_key_refs_with(Identity::Machine)
    }

    /// Test-only: same as [`get_key`](Self::get_key) but decrypts with an
    /// injected identity. Delegates to the SAME locked-read core as the
    /// production path.
    #[doc(hidden)]
    pub fn get_key_with_identity(&self, secret_ref: &str, identity: &str) -> Result<Option<String>, KeystoreError> {
        self.get_key_with(secret_ref, Identity::Injected(identity))
    }

    /// Test-only: same as [`set_key`](Self::set_key) but the RMW uses an
    /// injected identity. Delegates to the SAME typed-RMW core as the
    /// production path.
    #[doc(hidden)]
    pub fn set_key_with_identity(&self, secret_ref: &str, key: &str, identity: &str) -> Result<(), KeystoreError> {
        self.set_key_with(secret_ref, key, Identity::Injected(identity))
    }

    /// Test-only: same as [`delete_key`](Self::delete_key) but the RMW uses an
    /// injected identity.
    #[doc(hidden)]
    pub fn delete_key_with_identity(&self, secret_ref: &str, identity: &str) -> Result<(), KeystoreError> {
        self.delete_key_with(secret_ref, Identity::Injected(identity))
    }

    /// Test-only: same as [`list_provider_key_refs`](Self::list_provider_key_refs)
    /// but decrypts with an injected identity.
    #[doc(hidden)]
    pub fn list_provider_key_refs_with_identity(&self, identity: &str) -> Result<Vec<String>, KeystoreError> {
        self.list_provider_key_refs_with(Identity::Injected(identity))
    }

    /// Shared read core for [`get_key`](Self::get_key): load + decrypt under
    /// BOTH locks, normalize to `KeystoreData`, then return the named key.
    fn get_key_with(&self, secret_ref: &str, id: Identity<'_>) -> Result<Option<String>, KeystoreError> {
        self.with_locks(|ks| {
            let v = ks.load_locked_core(id)?;
            // Fail-closed: payload_to_v2 propagates Corrupt as Err (a corrupt
            // payload must NOT degrade to a silent Ok(None) that masks the failure).
            Ok(payload_to_v2(&v)?.get_provider_key(secret_ref).map(|s| s.to_string()))
        })
    }

    /// Shared typed-RMW core for [`set_key`](Self::set_key): load + decrypt under
    /// BOTH locks, normalize to v2, hand a typed mutator the data, then store the
    /// v2 value. One core for every mutating typed accessor (set/delete) so the
    /// v2-convergence guarantee lives in exactly one place.
    fn update_data_core<F>(&self, mutator: F, id: Identity<'_>) -> Result<(), KeystoreError>
    where
        F: FnOnce(&mut KeystoreData),
    {
        self.with_locks(|ks| {
            let raw = ks.load_locked_core(id)?;
            // Always normalize to v2 BEFORE the mutator: a fresh-install `{}`,
            // a v1 flat map, and an existing v2 payload all converge here, so a
            // write can never produce a mixed v1/v2 shape.
            //
            // Fail-closed (P0): payload_to_v2 propagates Corrupt as Err. A corrupt
            // keystore therefore NEVER gets silently overwritten with an empty v2
            // by set_key/delete_key — the write is aborted before the mutator runs.
            let mut data = payload_to_v2(&raw)?;
            mutator(&mut data);
            let value = data.to_value()?;
            ks.store_locked_core(&value, id)
        })
    }

    /// Shared set core: typed-RMW with `set_provider_key`.
    fn set_key_with(&self, secret_ref: &str, key: &str, id: Identity<'_>) -> Result<(), KeystoreError> {
        self.update_data_core(|data| data.set_provider_key(secret_ref, key), id)
    }

    /// Shared delete core: typed-RMW with `remove_provider_key` (idempotent).
    fn delete_key_with(&self, secret_ref: &str, id: Identity<'_>) -> Result<(), KeystoreError> {
        self.update_data_core(|data| {
            data.remove_provider_key(secret_ref);
        }, id)
    }

    /// Shared list core: load + normalize to v2, return the `secret_ref`s.
    fn list_provider_key_refs_with(&self, id: Identity<'_>) -> Result<Vec<String>, KeystoreError> {
        self.with_locks(|ks| {
            let v = ks.load_locked_core(id)?;
            Ok(payload_to_v2(&v)?.provider_keys.into_keys().collect())
        })
    }

    /// Move keystore.json → keystore.json.broken-<secs>-<nanos> (user-initiated
    /// recovery). Per §A fail-closed: only an explicit user action does this.
    /// The suffix uses nanosecond precision so two archives taken within the same
    /// second never collide (a second-precision timestamp would silently
    /// overwrite the prior archive via the `rename`).
    pub fn archive(&self) -> Result<std::path::PathBuf, KeystoreError> {
        self.with_locks(|ks| {
            let src = ks.file();
            if !src.exists() {
                return Err(KeystoreError::Envelope("no keystore to archive".into()));
            }
            let dst = broken_archive_path(&ks.dir);
            std::fs::rename(&src, &dst)?;
            Ok(dst)
        })
    }

    /// User-initiated reset (fresh start). Round-2 review P1 #4: per the §A
    /// fail-closed protocol, Reset must NOT unrecoverably delete the canonical file
    /// — it MOVES it to keystore.json.broken-<secs>-<nanos> (recoverable), then
    /// clears tmp. A subsequent store() starts a fresh keystore. The nanosecond
    /// suffix prevents same-second collisions with a prior archive. Returns the
    /// archive path if a canonical file existed (None if there was nothing to
    /// archive).
    pub fn reset(&self) -> Result<Option<std::path::PathBuf>, KeystoreError> {
        self.with_locks(|ks| {
            let src = ks.file();
            let archived = if src.exists() {
                let dst = broken_archive_path(&ks.dir);
                std::fs::rename(&src, &dst)?;
                Some(dst)
            } else {
                None
            };
            let _ = std::fs::remove_file(ks.dir.join(TMP));
            Ok(archived)
        })
    }

    fn set_file_perms(&self, p: &Path) -> Result<(), KeystoreError> {
        Ok(crate::fs_acl::secure_file(p)?)
    }

    /// Path to the pre-migration backup (spec §A, S2a). Exposed so tests can
    /// assert on it without hardcoding the filename string.
    fn backup_path(&self) -> PathBuf { self.dir.join(BACKUP_PRE_MIGRATION) }

    /// Under BOTH locks: create the pre-migration backup (TRUE no-clobber —
    /// `create_new` only, never overwrites a prior backup). Reads `keystore.json`
    /// bytes and writes them DIRECTLY to `keystore.json.bak-pre-migration` via
    /// `create_new` (O_EXCL), secured with `fs_acl::secure_file` + fsync'd. No-op
    /// if the canonical file is absent or the backup already exists. This is the
    /// locked core shared by `migrate_to_v2_locked_core` and the standalone
    /// [`backup_keystore`](crate::keystore::backup_keystore) free function (Step 1
    /// of the migration coordinator: backup SEPARATE from rewrite).
    fn backup_locked(&self) -> Result<(), KeystoreError> {
        let src = self.file();
        if !src.exists() {
            // Nothing to back up (fresh install). Idempotent no-op.
            return Ok(());
        }
        let bak = self.backup_path();
        let bytes = std::fs::read(&src)?;
        // Atomically create the FINAL backup path with O_EXCL (create_new). If
        // it already exists, this returns `AlreadyExists` and we skip — TRUE
        // no-clobber. This avoids the `exists()` + `rename()` TOCTOU of the old
        // shape, where `rename` would silently clobber a prior backup on Unix.
        use std::io::Write;
        let mut dst = match std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&bak)
        {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                // A prior backup exists — leave it untouched.
                return Ok(());
            }
            Err(e) => return Err(KeystoreError::Io(e)),
        };
        dst.write_all(&bytes)?;
        dst.flush()?;
        dst.sync_all()?;
        drop(dst);
        // Secure the final backup file so on-disk permissions are correct.
        crate::fs_acl::secure_file(&bak)?;
        Ok(())
    }

    /// Under BOTH locks: create the idempotent pre-migration backup (see
    /// [`backup_locked`](Self::backup_locked)), then atomically rewrite the
    /// keystore with the v2 payload. Used by `migrate_to_v2`.
    fn migrate_to_v2_locked(&self, data: &KeystoreData) -> Result<(), KeystoreError> {
        self.migrate_to_v2_locked_core(data, Identity::Machine)
    }

    /// THE migration core (caller holds BOTH locks). Holds ALL migration logic:
    /// (1) the idempotent pre-migration backup and (2) the atomic v2 rewrite. The
    /// production [`migrate_to_v2_locked`](Self::migrate_to_v2_locked) and the test
    /// seam go through here — no duplicated backup or rewrite logic to drift.
    fn migrate_to_v2_locked_core(&self, data: &KeystoreData, id: Identity<'_>) -> Result<(), KeystoreError> {
        // (1) Backup. Idempotent: a prior backup (from a standalone backup_keystore
        // call or a previous migration) is never overwritten.
        self.backup_locked()?;
        // (2) Atomic rewrite of the v2 payload via the same encrypt +
        // atomic_replace path the rest of the keystore uses.
        let value = data.to_value()?;
        self.store_locked_core(&value, id)
    }
}

/// Filename of the one-time pre-migration backup (spec §A, S2a).
const BACKUP_PRE_MIGRATION: &str = "keystore.json.bak-pre-migration";

/// Derive a fresh, collision-resistant archive path for the broken-keystore
/// recovery flow (`archive` / `reset`). Uses `<secs>-<nanos>` precision so two
/// archives taken within the same second land on distinct paths — a
/// second-only suffix would let a rapid second archive silently overwrite the
/// first via the `rename`.
fn broken_archive_path(dir: &Path) -> std::path::PathBuf {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    dir.join(format!("keystore.json.broken-{}-{}", now.as_secs(), now.subsec_nanos()))
}

/// Inspect the keystore at `dir` and return its typed load state (S2a). Standalone
/// entry point (takes a dir, not a `&Keystore`) so callers can probe before
/// deciding to migrate. See [`Keystore::load_state`] for the classification rules.
pub fn load_state(dir: &Path) -> KeystoreLoadState {
    // Keystore::new creates the dir if missing; here we ONLY classify, so probe
    // for existence first to return Missing without creating anything.
    if !dir.join(FILE).exists() {
        return KeystoreLoadState::Missing;
    }
    match Keystore::new(dir.to_path_buf()) {
        Ok(ks) => ks.load_state(),
        Err(e) => KeystoreLoadState::Corrupt(e),
    }
}

/// Test-only: same as [`load_state`] but decrypts with an injected identity
/// string instead of reading the machine. Lets migration tests drive the
/// classification (and enumerate candidates from it) without touching real OS
/// identity. Delegates to the SAME `Keystore::load_state_with` core as the
/// production free function (no duplicated read/decrypt/classify logic).
#[doc(hidden)]
pub fn load_state_with_identity(dir: &Path, identity: &str) -> KeystoreLoadState {
    if !dir.join(FILE).exists() {
        return KeystoreLoadState::Missing;
    }
    match Keystore::new(dir.to_path_buf()) {
        Ok(ks) => ks.load_state_with(Identity::Injected(identity)),
        Err(e) => KeystoreLoadState::Corrupt(e),
    }
}

/// Migrate a legacy v1 flat map to a v2 `KeystoreData` payload (S2a).
///
/// Standalone (takes a dir, not a `&Keystore`): creates a `Keystore` internally
/// to get the dual-lock + atomic-replace machinery. Performed UNDER the keystore's
/// fs2 flock via `migrate_to_v2_locked`. Before the rewrite, copies
/// `keystore.json` → `keystore.json.bak-pre-migration` (create-new only — a prior
/// backup is never overwritten) and secures the backup with `fs_acl::secure_file`.
pub fn migrate_to_v2(dir: &Path, legacy_map: HashMap<String, String>) -> Result<(), KeystoreError> {
    let ks = Keystore::new(dir.to_path_buf())?;
    let data = KeystoreData::new_v2(legacy_map);
    ks.with_locks(|k| k.migrate_to_v2_locked(&data))
}

/// Create the one-time pre-migration backup SEPARATELY from the v2 rewrite (S2a).
///
/// The migration coordinator (Step 4) needs Phase-1 backup and Phase-4 rewrite to
/// be distinct operations. This is the standalone backup entry point: it takes a
/// dir (not a `&Keystore`), classifies the keystore via [`load_state`], then —
/// under the keystore's fs2 flock — copies `keystore.json` →
/// `keystore.json.bak-pre-migration` (create-new only, never overwrites a prior
/// backup) and secures the copy with `fs_acl::secure_file`.
///
/// Idempotent: a second call (or a call after `migrate_to_v2` already made the
/// backup) is a no-op.
///
/// Classification:
/// - `Missing`        → `Ok(())` (nothing to back up).
/// - `CurrentV2(_)`   → `Ok(())` (already migrated; no v1 to preserve).
/// - `LegacyV1(_)`    → performs the backup.
/// - `Corrupt(e)`     → `Err(e)` (cannot back up an unreadable file).
pub fn backup_keystore(dir: &Path) -> Result<(), KeystoreError> {
    backup_keystore_with(dir, Identity::Machine)
}

/// Test-only: write an envelope (encrypted with an explicit identity) directly to
/// `dir/keystore.json`. Lets tests seed a v1 or v2 payload without touching real
/// OS identity or the production `store` path. Mirrors how the file looks on disk
/// after a real write (Envelope JSON, atomic replace, secure perms).
#[doc(hidden)]
pub fn store_with_identity(
    dir: &Path,
    identity: &str,
    identity_source: IdentitySource,
    keys: &serde_json::Value,
) -> Result<(), KeystoreError> {
    let env = encrypt(identity, identity_source, keys)?;
    let json = serde_json::to_vec(&env).map_err(|e| KeystoreError::Envelope(e.to_string()))?;
    let path = dir.join(FILE);
    std::fs::write(&path, &json)?;
    crate::fs_acl::secure_file(&path)?;
    Ok(())
}

/// Test-only: migrate a legacy map to v2 using an injected identity. Same as
/// [`migrate_to_v2`] but the rewrite (and backup classification) uses the given
/// identity instead of the machine identity. Delegates to the SAME
/// `migrate_to_v2_locked_core` as the production path (no duplicated
/// backup/rewrite logic), so tests exercise the real migration code.
#[doc(hidden)]
pub fn migrate_to_v2_with_identity(
    dir: &Path,
    legacy_map: HashMap<String, String>,
    identity: &str,
) -> Result<(), KeystoreError> {
    let ks = Keystore::new(dir.to_path_buf())?;
    let data = KeystoreData::new_v2(legacy_map);
    ks.with_locks(|k| k.migrate_to_v2_locked_core(&data, Identity::Injected(identity)))
}

/// Test-only: standalone backup with an injected identity (mirror of
/// [`backup_keystore`]). Classifies the keystore with the injected identity and,
/// if it is `LegacyV1`, creates the create-new backup. Delegates to the same
/// `backup_locked` core as the production path.
#[doc(hidden)]
pub fn backup_keystore_with_identity(dir: &Path, identity: &str) -> Result<(), KeystoreError> {
    backup_keystore_with(dir, Identity::Injected(identity))
}

/// Shared backup entry point for the production (`Machine`) and test (`Injected`)
/// paths. Classifies the keystore with `id`, then — only for `LegacyV1` — creates
/// the idempotent create-new backup under the keystore's fs2 flock.
fn backup_keystore_with(dir: &Path, id: Identity<'_>) -> Result<(), KeystoreError> {
    // Probe existence first so a fresh-install dir returns early without creating
    // a Keystore (matches load_state's fast-path contract).
    if !dir.join(FILE).exists() {
        return Ok(());
    }
    let ks = Keystore::new(dir.to_path_buf())?;
    match ks.load_state_with(id) {
        KeystoreLoadState::Missing => Ok(()),
        KeystoreLoadState::CurrentV2(_) => Ok(()),
        KeystoreLoadState::LegacyV1(_) => ks.with_locks(|k| k.backup_locked()),
        KeystoreLoadState::Corrupt(e) => Err(e),
    }
}

/// Test-only: path to the pre-migration backup inside `dir`.
#[doc(hidden)]
pub fn backup_path_in(dir: &Path) -> PathBuf {
    dir.join(BACKUP_PRE_MIGRATION)
}

/// Atomic replace. macOS: rename over target (first-create or update).
/// Windows: MoveFileExW for first-create; ReplaceFileW for updates.
#[cfg(target_os = "macos")]
fn atomic_replace(src: &Path, dst: &Path) -> Result<(), KeystoreError> {
    std::fs::rename(src, dst)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn atomic_replace(src: &Path, dst: &Path) -> Result<(), KeystoreError> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, ReplaceFileW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    fn wide(path: &Path) -> Vec<u16> {
        path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    let src_wide = wide(src);
    let dst_wide = wide(dst);
    // SAFETY: both buffers are NUL-terminated and remain alive for the duration
    // of the Win32 call. The optional backup/exclude/preserved pointers are null.
    let result = unsafe {
        if dst.exists() {
            ReplaceFileW(
                dst_wide.as_ptr(),
                src_wide.as_ptr(),
                std::ptr::null(),
                0,
                std::ptr::null(),
                std::ptr::null(),
            )
        } else {
            MoveFileExW(
                src_wide.as_ptr(),
                dst_wide.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        }
    };
    if result == 0 {
        return Err(KeystoreError::Io(std::io::Error::last_os_error()));
    }
    Ok(())
}

// Win32 ACL functions (current_user_sid, sid_from_token_user_buf,
// set_win32_owner_dacl) have been extracted to fs_acl.rs and are shared
// by both keystore and database. The keystore verification test below
// references them via crate::fs_acl::.

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn atomic_replace(_src: &Path, _dst: &Path) -> Result<(), KeystoreError> {
    Err(KeystoreError::Envelope("atomic_replace not implemented on this platform".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encrypt_produces_pinned_envelope() {
        let env = encrypt("test-machine-uuid", IdentitySource::MacosIoplatformuuid, &serde_json::json!({"openai":"sk-test"})).unwrap();
        assert_eq!(env.version, 1);
        assert_eq!(env.aead, "aes-256-gcm");
        assert_eq!(env.kdf, "argon2id");
        assert_eq!(env.kdf_params.m_kib, 65536);
        assert_eq!(env.kdf_params.t, 3);
        assert_eq!(env.kdf_params.p, 1);
        assert_eq!(env.kdf_params.output_len, 32);
        // round-trip via the test helper
        let out = decrypt_with_identity(&env, "test-machine-uuid").unwrap();
        assert_eq!(out, serde_json::json!({"openai":"sk-test"}));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn atomic_replace_first_create_moves_source() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("keystore.json.tmp");
        let dst = dir.path().join("keystore.json");
        std::fs::write(&src, b"first").unwrap();

        atomic_replace(&src, &dst).unwrap();

        assert_eq!(std::fs::read(&dst).unwrap(), b"first");
        assert!(!src.exists());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn atomic_replace_update_replaces_destination() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("keystore.json.tmp");
        let dst = dir.path().join("keystore.json");
        std::fs::write(&dst, b"old").unwrap();
        std::fs::write(&src, b"new").unwrap();

        atomic_replace(&src, &dst).unwrap();

        assert_eq!(std::fs::read(&dst).unwrap(), b"new");
        assert!(!src.exists());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn atomic_replace_failure_preserves_destination() {
        let dir = tempfile::tempdir().unwrap();
        let missing_src = dir.path().join("missing.tmp");
        let dst = dir.path().join("keystore.json");
        std::fs::write(&dst, b"canonical").unwrap();

        assert!(atomic_replace(&missing_src, &dst).is_err());
        assert_eq!(std::fs::read(&dst).unwrap(), b"canonical");
    }

    /// §A Windows perm verification (plan Task 2): after set_win32_owner_dacl, read the
    /// security descriptor back via GetNamedSecurityInfoW and assert the DACL has exactly
    /// ONE ACE, it's an ACCESS_ALLOWED ACE for the CURRENT USER with GENERIC_ALL, and the
    /// DACL is PROTECTED (SE_DACL_PROTECTED — inheritance blocked). This is the only test
    /// that can confirm the production ACL is actually what we intend (the macOS perms are
    /// not testable here; the Windows one is, against the real Win32 API).
    #[cfg(target_os = "windows")]
    #[test]
    fn win32_dacl_locks_to_current_user_and_is_protected() {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Foundation::LocalFree;
        use windows_sys::Win32::Security::Authorization::{
            GetNamedSecurityInfoW, SE_FILE_OBJECT,
        };
        use windows_sys::Win32::Security::{
            AclSizeInformation, EqualSid, GetAclInformation, GetAce,
            GetSecurityDescriptorControl, ACL_SIZE_INFORMATION,
            DACL_SECURITY_INFORMATION, OWNER_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR,
            SE_DACL_PROTECTED, ACL,
        };
        // ACCESS_ALLOWED_ACE_TYPE lives in SystemServices (not Security) and is u32;
        // ACE_HEADER.AceType is u8, so compare against 0u8 directly.
        const ACCESS_ALLOWED: u8 = 0;

        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("keystore.json");
        std::fs::write(&f, b"x").unwrap();
        crate::fs_acl::set_win32_owner_dacl(&f, false).expect("set_win32_owner_dacl should succeed");

        // Expected SID (same source the prod path used):
        let sid_buf = crate::fs_acl::current_user_sid().unwrap();
        let expected_sid = crate::fs_acl::sid_from_token_user_buf(&sid_buf).unwrap();

        let path_wide: Vec<u16> = f.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        let mut owner: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
        let mut dacl: *mut ACL = std::ptr::null_mut();
        // SAFETY: path_wide is NUL-terminated + alive; the out-params receive the SD/DACL.
        // GetNamedSecurityInfoW allocates the SD via the Local heap → we LocalFree it.
        let rc = unsafe {
            GetNamedSecurityInfoW(
                path_wide.as_ptr(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION | OWNER_SECURITY_INFORMATION,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut dacl,
                std::ptr::null_mut(),
                &mut owner,
            )
        };
        assert_eq!(rc, 0, "GetNamedSecurityInfoW failed: Win32 error {rc}");
        assert!(!owner.is_null(), "security descriptor allocated");
        struct SdGuard(PSECURITY_DESCRIPTOR);
        impl Drop for SdGuard {
            fn drop(&mut self) {
                // SAFETY: owner was Local-allocated by GetNamedSecurityInfoW.
                unsafe { LocalFree(self.0 as *mut _) };
            }
        }
        let _sd = SdGuard(owner);

        // (a) DACL is PROTECTED (inheritance blocked).
        let mut control: u16 = 0;
        let mut revision: u32 = 0;
        // SAFETY: owner is a valid SD from GetNamedSecurityInfoW.
        let ok = unsafe { GetSecurityDescriptorControl(owner, &mut control, &mut revision) };
        assert_ne!(ok, 0, "GetSecurityDescriptorControl failed");
        assert_ne!(
            control & SE_DACL_PROTECTED,
            0,
            "DACL must be PROTECTED (inheritance blocked); control=0x{control:x}"
        );

        // (b) Exactly ONE ACE. `dacl` was returned by GetNamedSecurityInfoW above and
        // points into the (still-alive via _sd) security descriptor.
        let mut size_info = ACL_SIZE_INFORMATION { AceCount: 0, AclBytesInUse: 0, AclBytesFree: 0 };
        // SAFETY: dacl is a valid DACL pointer from the SD; buf is sized to the type.
        let ok = unsafe {
            GetAclInformation(
                dacl,
                &mut size_info as *mut _ as *mut _,
                std::mem::size_of::<ACL_SIZE_INFORMATION>() as u32,
                AclSizeInformation,
            )
        };
        assert_ne!(ok, 0, "GetAclInformation failed");
        assert_eq!(size_info.AceCount, 1, "exactly one ACE (current user only)");

        // (c) That ACE is ACCESS_ALLOWED for the current user with GENERIC_ALL.
        let mut ace_ptr: *mut core::ffi::c_void = std::ptr::null_mut();
        // SAFETY: index 0 < AceCount(1).
        let ok = unsafe { GetAce(dacl, 0, &mut ace_ptr) };
        assert_ne!(ok, 0, "GetAce(0) failed");
        // ACCESS_ALLOWED_ACE = { Header: ACE_HEADER, Mask: u32, SidStart: u32 (start of SID) }.
        // The SID starts at &SidStart; ACE_HEADER.AceType == ACCESS_ALLOWED_ACE_TYPE.
        #[repr(C)]
        #[allow(non_snake_case)] // field names mirror the Win32 ACE_HEADER / ACCESS_ALLOWED_ACE layout
        struct AceHeader { AceType: u8, AceFlags: u8, AceSize: u16 }
        #[repr(C)]
        #[allow(non_snake_case)] // field names mirror the Win32 layout (Header/Mask/SidStart)
        struct AccessAllowedAce { Header: AceHeader, Mask: u32, SidStart: u32 }
        let ace: &AccessAllowedAce = unsafe { &*(ace_ptr as *const AccessAllowedAce) };
        assert_eq!(ace.Header.AceType, ACCESS_ALLOWED, "ACE is ACCESS_ALLOWED");
        // GENERIC_ALL (0x10000000) is what we stored, but on a FILE object GetAce returns
        // the file-mapped mask = FILE_ALL_ACCESS (DELETE|READ_CONTROL|WRITE_DAC|
        // WRITE_OWNER|SYNCHRONIZE|file perms = 0x1F01FF). Assert the full-control bits
        // are present (DELETE + WRITE_DAC + WRITE_OWNER proves ownership-level control,
        // incl. the DELETE ReplaceFileW needs).
        const DELETE: u32 = 0x0001_0000;
        const WRITE_DAC: u32 = 0x0004_0000;
        const WRITE_OWNER: u32 = 0x0008_0000;
        let full_ctrl = DELETE | WRITE_DAC | WRITE_OWNER;
        assert_eq!(
            ace.Mask & full_ctrl, full_ctrl,
            "ACE mask grants full control (DELETE|WRITE_DAC|WRITE_OWNER); mask=0x{:x}", ace.Mask
        );
        // SID begins at SidStart field.
        let ace_sid = (&ace.SidStart as *const u32) as windows_sys::Win32::Security::PSID;
        // SAFETY: both SIDs are valid (one from the token, one embedded in the ACE).
        let eq = unsafe { EqualSid(ace_sid, expected_sid) };
        assert_ne!(eq, 0, "ACE SID == current-user SID");
    }
}
