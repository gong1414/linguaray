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
    #[cfg(target_os = "windows")]
    fn set_dir_perms(dir: &Path) -> Result<(), KeystoreError> {
        set_win32_owner_dacl(dir, true)
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
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
    #[cfg(target_os = "windows")]
    fn set_file_perms(&self, p: &Path) -> Result<(), KeystoreError> {
        set_win32_owner_dacl(p, false)
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn set_file_perms(&self, _p: &Path) -> Result<(), KeystoreError> { Ok(()) }
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

/// Windows §A perms: lock `path` (file or dir) to the current user only.
///
/// Sets owner = current-user SID and a PROTECTED DACL with exactly one explicit
/// ACE granting that SID `GENERIC_ALL` (which includes DELETE, so `ReplaceFileW`
/// can later replace the keystore file). `PROTECTED_DACL_SECURITY_INFORMATION`
/// blocks inherited ACEs — the object is reachable ONLY by the current user,
/// matching the macOS 0o600/0o700 intent. NOT icacls (plan Task 2).
///
/// SID source: the current process token's user SID (canonical + unambiguous,
/// unlike a trustee *name*). The SID buffer is owned by us and must outlive the
/// `SetNamedSecurityInfoW` call (it reads, then copies internally — we free after).
#[cfg(target_os = "windows")]
/// Current-user SID (canonical, locale-independent) from the process token.
/// Returns a Vec<u8> OWNING the TOKEN_USER + the SID it points to (GetTokenInformation
/// writes both into one caller buffer; the SID aliases into it). Shared by the
/// production set call AND the verification test (so they compare the SAME SID source).
#[cfg(target_os = "windows")]
fn current_user_sid() -> Result<Vec<u8>, KeystoreError> {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::{GetTokenInformation, TOKEN_QUERY, TokenUser};
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    let mut token: HANDLE = std::ptr::null_mut();
    // SAFETY: GetCurrentProcess returns a pseudo-handle (no real handle to leak); we pass
    // it straight to OpenProcessToken, which writes our `token` out-param. TOKEN_QUERY is
    // the minimal right needed for GetTokenInformation.
    let ok = unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) };
    if ok == 0 {
        return Err(KeystoreError::Io(std::io::Error::last_os_error()));
    }
    struct TokenHandle(HANDLE);
    impl Drop for TokenHandle {
        fn drop(&mut self) {
            // SAFETY: self.0 is a real token handle from OpenProcessToken; CloseHandle
            // is the documented release. Called exactly once via Drop.
            unsafe { CloseHandle(self.0) };
        }
    }
    let _token_guard = TokenHandle(token);

    // TOKEN_USER = { SID_AND_ATTRIBUTES { Sid: PSID, Attributes: u32 } }. GetTokenInformation
    // writes the TOKEN_USER struct AND the SID it points to into the caller's buffer (the
    // SID is NOT separately allocated — it aliases into the buffer). Query length first.
    let mut needed: u32 = 0;
    // SAFETY: first call is expected to fail with ERROR_INSUFFICIENT_BUFFER and write the
    // required length into `needed`; we ignore the return and read `needed`.
    unsafe {
        GetTokenInformation(token, TokenUser, std::ptr::null_mut(), 0, &mut needed);
    }
    if needed == 0 {
        return Err(KeystoreError::Io(std::io::Error::last_os_error()));
    }
    let mut buf: Vec<u8> = vec![0u8; needed as usize];
    // SAFETY: buf holds `needed` bytes; the call writes ≤ needed and sets the TOKEN_USER.
    let ok = unsafe {
        GetTokenInformation(token, TokenUser, buf.as_mut_ptr() as *mut _, needed, &mut needed)
    };
    if ok == 0 {
        return Err(KeystoreError::Io(std::io::Error::last_os_error()));
    }
    Ok(buf)
}

/// Borrow the SID pointer out of a TOKEN_USER buffer (the SID aliases into `buf`).
/// Caller MUST keep `buf` alive for as long as the returned PSID is used.
#[cfg(target_os = "windows")]
fn sid_from_token_user_buf(buf: &[u8]) -> Result<windows_sys::Win32::Security::PSID, KeystoreError> {
    use windows_sys::Win32::Security::TOKEN_USER;
    if buf.len() < std::mem::size_of::<TOKEN_USER>() {
        return Err(KeystoreError::Envelope("token buffer too small for TOKEN_USER".into()));
    }
    // SAFETY: buf came from current_user_sid() (a valid TOKEN_USER). We only read the
    // fixed-size header to get the SID pointer; buf outlives the returned borrow.
    let user: &TOKEN_USER = unsafe { &*(buf.as_ptr() as *const TOKEN_USER) };
    Ok(user.User.Sid)
}

#[cfg(target_os = "windows")]
fn set_win32_owner_dacl(path: &Path, inherit: bool) -> Result<(), KeystoreError> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{GENERIC_ALL, LocalFree};
    use windows_sys::Win32::Security::Authorization::{
        SetEntriesInAclW, SetNamedSecurityInfoW, EXPLICIT_ACCESS_W, SE_FILE_OBJECT,
        SET_ACCESS, TRUSTEE_FORM, TRUSTEE_IS_SID, TRUSTEE_W,
    };
    use windows_sys::Win32::Security::{
        ACL, DACL_SECURITY_INFORMATION, OBJECT_SECURITY_INFORMATION,
        OWNER_SECURITY_INFORMATION, PROTECTED_DACL_SECURITY_INFORMATION, PSID,
        SUB_CONTAINERS_AND_OBJECTS_INHERIT,
    };

    // 1. Current-user SID (owned buffer); the SID aliases into it and must outlive the Set call.
    let sid_buf = current_user_sid()?;
    let sid: PSID = sid_from_token_user_buf(&sid_buf)?;

    // 2. Build a one-entry DACL: current user → GENERIC_ALL (incl. DELETE).
    // We build the EXPLICIT_ACCESS_W directly rather than via BuildExplicitAccessWithNameW
    // (that helper is name-based + locale-dependent). TrusteeForm = TRUSTEE_IS_SID: the
    // ptstrName field is a pointer to the SID itself (NOT a name string), which is
    // unambiguous and matches how the SID was obtained.
    let mut ea: EXPLICIT_ACCESS_W = unsafe { std::mem::zeroed() };
    ea.grfAccessPermissions = GENERIC_ALL;
    ea.grfAccessMode = SET_ACCESS; // GRANT would merge with inherited; SET replaces.
    // Directories get an INHERITABLE ACE so files created inside (keystore.json.tmp,
    // keystore.json, keystore.lock) auto-receive current-user full control — matches the
    // macOS 0o700-dir-+umask intent. Files get inheritance=0 (leaf, nothing to propagate).
    // SE_DACL_PROTECTED (set below) still blocks the dir from inheriting its PARENT's ACEs,
    // so the dir is locked to current user even though its own ACE propagates downward.
    ea.grfInheritance = if inherit { SUB_CONTAINERS_AND_OBJECTS_INHERIT } else { 0 };
    let mut trustee: TRUSTEE_W = unsafe { std::mem::zeroed() };
    trustee.TrusteeForm = TRUSTEE_IS_SID as TRUSTEE_FORM; // ptstrName = SID (not a name)
    trustee.ptstrName = sid as *mut _;
    ea.Trustee = trustee;

    let mut new_acl: *mut ACL = std::ptr::null_mut();
    // SAFETY: 1 entry in `&ea`, no old ACL (null) → builds a fresh ACL. On success
    // returns ERROR_SUCCESS (0) and allocates `new_acl` on the Local heap; we free it.
    let rc = unsafe { SetEntriesInAclW(1, &ea, std::ptr::null(), &mut new_acl) };
    if rc != 0 || new_acl.is_null() {
        return Err(KeystoreError::Envelope(format!("SetEntriesInAclW failed: Win32 error {rc}")));
    }
    // RAII: free the ACL with LocalFree (SetEntriesInAclW allocates via the Local heap;
    // LocalFree is in Win32::Foundation, not Memory).
    struct AclGuard(*mut ACL);
    impl Drop for AclGuard {
        fn drop(&mut self) {
            // SAFETY: self.0 was allocated by SetEntriesInAclW (Local heap); LocalFree
            // is the documented free. Called exactly once via Drop.
            unsafe { LocalFree(self.0 as *mut _) };
        }
    }
    let _acl_guard = AclGuard(new_acl);

    // 3. Apply: set OWNER + DACL + PROTECTED (block inheritance) atomically on the path.
    let path_wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let info: OBJECT_SECURITY_INFORMATION = OWNER_SECURITY_INFORMATION
        | DACL_SECURITY_INFORMATION
        | PROTECTED_DACL_SECURITY_INFORMATION;
    // SAFETY: path_wide is NUL-terminated and alive across the call; sid aliases `buf`
    // (also alive); new_acl is valid (alive via _acl_guard). SetNamedSecurityInfoW copies
    // what it needs, so freeing sid-buf/acl after the call is correct.
    let rc = unsafe {
        SetNamedSecurityInfoW(
            path_wide.as_ptr(),
            SE_FILE_OBJECT,
            info,
            sid,
            std::ptr::null_mut(),
            new_acl,
            std::ptr::null_mut(),
        )
    };
    if rc != 0 {
        return Err(KeystoreError::Envelope(format!(
            "SetNamedSecurityInfoW failed: Win32 error {rc}"
        )));
    }
    Ok(())
}

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
        set_win32_owner_dacl(&f, false).expect("set_win32_owner_dacl should succeed");

        // Expected SID (same source the prod path used):
        let sid_buf = current_user_sid().unwrap();
        let expected_sid = sid_from_token_user_buf(&sid_buf).unwrap();

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
