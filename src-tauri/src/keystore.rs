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

const DOMAIN_SEPARATOR: &[u8] = b"islandpot-keystore-v1\0";
const FIXED_AAD: &[u8] = b"islandpot-keystore-envelope-v1";
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
    if envelope.version != VERSION { return Err(KeystoreError::UnsupportedVersion(envelope.version)); }
    if envelope.aead != "aes-256-gcm" || envelope.kdf != "argon2id" { return Err(KeystoreError::Envelope("bad aead/kdf".into())); }
    if envelope.kdf_params.m_kib != PINNED_KDF.m_kib
        || envelope.kdf_params.t != PINNED_KDF.t
        || envelope.kdf_params.p != PINNED_KDF.p
        || envelope.kdf_params.output_len != PINNED_KDF.output_len {
        return Err(KeystoreError::Envelope("kdf_params not pinned".into()));
    }
    if envelope.identity_source != machine_source {
        return Err(KeystoreError::AuthFailed);
    }
    let salt = B64.decode(&envelope.salt).map_err(|e| KeystoreError::Envelope(e.to_string()))?;
    let nonce = B64.decode(&envelope.nonce).map_err(|e| KeystoreError::Envelope(e.to_string()))?;
    let ct = B64.decode(&envelope.ciphertext).map_err(|e| KeystoreError::Envelope(e.to_string()))?;
    if salt.len() != SALT_LEN || nonce.len() != NONCE_LEN || ct.len() < 16 {
        return Err(KeystoreError::Envelope("bad field lengths".into()));
    }
    let identity = machine_source.read()?;
    let key = derive_key(&identity, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| KeystoreError::Crypto(e.to_string()))?;
    let pt = cipher.decrypt(Nonce::from_slice(&nonce), aes_gcm::aead::Payload { msg: &ct, aad: FIXED_AAD })
        .map_err(|_| KeystoreError::AuthFailed)?;
    let v: serde_json::Value = serde_json::from_slice(&pt).map_err(|e| KeystoreError::Envelope(e.to_string()))?;
    Ok(v)
}

#[doc(hidden)]
/// Test-only: decrypt with an explicit identity string instead of reading the
/// machine. Lets tests drive the crypto without touching real OS identity.
pub fn decrypt_with_identity(envelope: &Envelope, identity: &str) -> Result<serde_json::Value, KeystoreError> {
    if envelope.version != VERSION { return Err(KeystoreError::UnsupportedVersion(envelope.version)); }
    if envelope.aead != "aes-256-gcm" || envelope.kdf != "argon2id" { return Err(KeystoreError::Envelope("bad aead/kdf".into())); }
    if envelope.kdf_params.m_kib != PINNED_KDF.m_kib || envelope.kdf_params.t != PINNED_KDF.t
        || envelope.kdf_params.p != PINNED_KDF.p || envelope.kdf_params.output_len != PINNED_KDF.output_len {
        return Err(KeystoreError::Envelope("kdf_params not pinned".into()));
    }
    let salt = B64.decode(&envelope.salt).map_err(|e| KeystoreError::Envelope(e.to_string()))?;
    let nonce = B64.decode(&envelope.nonce).map_err(|e| KeystoreError::Envelope(e.to_string()))?;
    let ct = B64.decode(&envelope.ciphertext).map_err(|e| KeystoreError::Envelope(e.to_string()))?;
    if salt.len() != SALT_LEN || nonce.len() != NONCE_LEN || ct.len() < 16 {
        return Err(KeystoreError::Envelope("bad field lengths".into()));
    }
    let key = derive_key(identity, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| KeystoreError::Crypto(e.to_string()))?;
    let pt = cipher.decrypt(Nonce::from_slice(&nonce), aes_gcm::aead::Payload { msg: &ct, aad: FIXED_AAD })
        .map_err(|_| KeystoreError::AuthFailed)?;
    serde_json::from_slice(&pt).map_err(|e| KeystoreError::Envelope(e.to_string()))
}

use std::path::{Path, PathBuf};
use parking_lot::Mutex;

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

    #[cfg(target_os = "macos")]
    fn set_dir_perms(dir: &Path) -> Result<(), KeystoreError> {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    fn set_dir_perms(_dir: &Path) -> Result<(), KeystoreError> { Ok(()) }

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

    /// Locked read (caller holds BOTH locks). Used by with_locks bodies.
    fn load_locked(&self) -> Result<serde_json::Value, KeystoreError> {
        let path = self.file();
        if !path.exists() { return Ok(serde_json::json!({})); }
        let bytes = std::fs::read(&path)?;
        let env: Envelope = serde_json::from_slice(&bytes)
            .map_err(|e| KeystoreError::Envelope(format!("malformed: {e}")))?;
        decrypt(&env, IdentitySource::CURRENT)
    }

    /// Encrypt + atomically write the keys map. Takes both locks.
    pub fn store(&self, keys: &serde_json::Value) -> Result<(), KeystoreError> {
        // Clone to satisfy the closure borrow (keys: &Value → owned for the body).
        let keys = keys.clone();
        self.with_locks(|ks| ks.store_locked(&keys))
    }

    /// Locked write (caller holds BOTH locks). Used by with_locks bodies.
    fn store_locked(&self, keys: &serde_json::Value) -> Result<(), KeystoreError> {
        let identity = IdentitySource::CURRENT.read()?;
        let env = encrypt(&identity, IdentitySource::CURRENT, keys)?;
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
        self.with_locks(|ks| {
            // (stale-tmp cleanup is done once in with_locks under the lock.)
            let mut keys = ks.load_locked()?;
            if !keys.is_object() {
                keys = serde_json::json!({});
            }
            mutator(&mut keys);
            ks.store_locked(&keys)
        })
    }

    /// Move keystore.json → keystore.json.broken-<ts> (user-initiated recovery).
    /// Per §A fail-closed: only an explicit user action does this.
    pub fn archive(&self) -> Result<std::path::PathBuf, KeystoreError> {
        self.with_locks(|ks| {
            let src = ks.file();
            if !src.exists() {
                return Err(KeystoreError::Envelope("no keystore to archive".into()));
            }
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let dst = ks.dir.join(format!("keystore.json.broken-{ts}"));
            std::fs::rename(&src, &dst)?;
            Ok(dst)
        })
    }

    /// User-initiated reset (fresh start). Round-2 review P1 #4: per the §A
    /// fail-closed protocol, Reset must NOT unrecoverably delete the canonical file
    /// — it MOVES it to keystore.json.broken-<ts> (recoverable), then clears tmp.
    /// A subsequent store() starts a fresh keystore. Returns the archive path if a
    /// canonical file existed (None if there was nothing to archive).
    pub fn reset(&self) -> Result<Option<std::path::PathBuf>, KeystoreError> {
        self.with_locks(|ks| {
            let src = ks.file();
            let archived = if src.exists() {
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let dst = ks.dir.join(format!("keystore.json.broken-{ts}"));
                std::fs::rename(&src, &dst)?;
                Some(dst)
            } else {
                None
            };
            let _ = std::fs::remove_file(ks.dir.join(TMP));
            Ok(archived)
        })
    }

    #[cfg(target_os = "macos")]
    fn set_file_perms(&self, p: &Path) -> Result<(), KeystoreError> {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o600))?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    fn set_file_perms(&self, _p: &Path) -> Result<(), KeystoreError> { Ok(()) }
}

/// Atomic replace. macOS: rename over target (first-create or update).
#[cfg(target_os = "macos")]
fn atomic_replace(src: &Path, dst: &Path) -> Result<(), KeystoreError> {
    std::fs::rename(src, dst)?;
    Ok(())
}

// NOTE: the spec describes a Windows atomic_replace using ReplaceFileW for updates
// and MoveFileExW for first-create. That code is #[cfg(target_os = "windows")] and is
// omitted from this macOS build; it will be added (or the windows-sys dep pulled in)
// when building for Windows. For now, only the macOS path exists.
#[cfg(not(target_os = "macos"))]
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
}
