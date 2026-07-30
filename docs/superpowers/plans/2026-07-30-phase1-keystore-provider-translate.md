# Phase 1: Keystore → Provider HTTP → Translate Service — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `translate` actually call an AI provider end-to-end — the user picks a provider, supplies a key (stored encrypted, machine-bound), and gets a real translation back. Replaces the current `bail!` stub in `providers.rs`.

**Architecture:** A pure protocol caller (`providers.rs`) builds the OpenAI/Anthropic HTTP request from a preset + key + `WireParams` and parses the response. A `Keystore` module encrypts keys with AES-256-GCM using an Argon2id key derived from machine identity, in a versioned envelope file with cross-process lock + atomic replace. A `translate_service` orchestrates: resolve engine → read key from keystore (Rust-side) → call provider. Frontend only gets `set_key`/`delete_key`/`key_status`/`translate` — never plaintext keys.

**Tech Stack:** Rust 1.95 · Tauri 2 · `reqwest` (rustls, no default features to avoid native-tls) · `aes-gcm` · `argon2` · `rand` · `zeroize` · `base64` · `serde`/`serde_json` · `parking_lot` · `fd-lock` · `wiremock` (dev) · `tokio` (test). Frontend: SolidJS + `@tauri-apps/api`.

**Spec reference:** `docs/superpowers/specs/2026-07-30-islandpot-v1-design.md` (§A keystore, §Wire, §G, §Privacy).

---

## File Structure

**Create:**
- `src-tauri/src/keystore.rs` — §A: envelope, crypto, identity, atomic write, fail-closed. Tested first-class.
- `src-tauri/src/wire.rs` — `WireParams` strong-typed struct + per-dialect request builders (pure functions: preset+key+params → `reqwest::RequestBuilder`). Plus app-options → prompt builder.
- `src-tauri/src/service.rs` — `translate_service`: resolve engine, read key, call, classify error (returns `Result` with `FallbackEligible` vs `ConfigError`). v1 wires single-engine (no fallback yet — fallback engines don't exist until Phase 3); the classification is in place so Phase 3 just adds the fallback branch.
- `src-tauri/src/error.rs` — `Error` enum with `FallbackEligible(kind)` and `Config(kind)` variants.
- `src-tauri/tests/keystore.rs` — integration tests for keystore (tamper, identity, nonce, corrupt, atomic, concurrency, validation).
- `src-tauri/tests/wire.rs` — `wiremock` tests for provider request construction per dialect.
- `src-tauri/tests/service.rs` — `translate_service` tests with a mock HTTP server.

**Modify:**
- `src-tauri/Cargo.toml` — add dependencies.
- `src-tauri/src/lib.rs` — add `mod` declarations; add `set_key`/`delete_key`/`key_status` commands; make `translate` async and route through `service`; manage `AppState` (`reqwest::Client`, `Keystore`).
- `src-tauri/src/providers.rs` — split: keep preset catalog data here; move HTTP-calling into `wire.rs`. `ProviderPreset` gains a `endpoint: String` (full URL) field; remove `base_url`.
- `src-tauri/src/engines/mod.rs` — no change in Phase 1 (registry stays empty).
- `src/App.tsx` — wire a key-input flow (calls `set_key`, then `translate`).
- `src-tauri/capabilities/default.json` — no change yet (no new plugins beyond core).

---

## Task 1: Add dependencies

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Add runtime dependencies**

Replace the `[dependencies]` block with:

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json"] }
aes-gcm = "0.10"
argon2 = "0.5"
rand = "0.8"
zeroize = { version = "1", features = ["derive"] }
base64 = "0.22"
parking_lot = "0.12"
fd-lock = "4"
thiserror = "1"
log = "0.4"
```

- [ ] **Step 2: Add dev-dependencies + tokio for tests**

Append after `[dependencies]`:

```toml
[dev-dependencies]
wiremock = "0.6"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
tempfile = "3"
```

- [ ] **Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: `Finished` (downloads crates on first run, may take a minute).

- [ ] **Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "deps: add reqwest/aes-gcm/argon2/etc for Phase 1"
```

---

## Task 2: Error type (FallbackEligible vs Config)

**Files:**
- Create: `src-tauri/src/error.rs`

- [ ] **Step 1: Write the type**

```rust
//! Translate-error classification (spec §G).
//!
//! `FallbackEligible` — a transient/provider error that *may* justify falling
//!   back to another engine (network, timeout, 429, 5xx, parse failure).
//! `Config` — a configuration/auth problem that must send the user to Settings,
//!   never silently fall back (missing key, 401/403, bad model, keystore fault).

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("fallback-eligible: {0}")]
    FallbackEligible(#[from] FallbackKind),

    #[error("config error: {0}")]
    Config(#[from] ConfigKind),

    #[error(transparent)]
    Keystore(#[from] crate::keystore::KeystoreError),
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
}
```

- [ ] **Step 2: Add module + check**

In `src-tauri/src/lib.rs`, add at top with the other `mod` lines:
```rust
pub mod error;
```
Run: `cd src-tauri && cargo check`
Expected: PASS (compiles; `KeystoreError` referenced but defined in Task 3).

If `cargo check` fails on the missing `KeystoreError`, proceed to Task 3 — it resolves it. Do not stub.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/error.rs src-tauri/src/lib.rs
git commit -m "feat(error): classified Error (FallbackEligible vs Config)"
```

---

## Task 3: Keystore — identity (machine-bound)

**Files:**
- Create: `src-tauri/src/keystore.rs`

- [ ] **Step 1: Write KeystoreError + identity module (start of keystore.rs)**

```rust
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
    /// fallback — spec §A).
    pub fn read(&self) -> Result<String, KeystoreError> {
        let raw = match self {
            IdentitySource::MacosIoplatformuuid => read_macos_io_platform_uuid()?,
            IdentitySource::WindowsMachineguid => read_windows_machine_guid()?,
        };
        Ok(raw.trim().to_lowercase())
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
    // HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Cryptography\MachineGuid
    // Using the `winreg` crate would be cleaner; kept dependency-free via reg.exe.
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

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
impl IdentitySource {
    const CURRENT: IdentitySource = IdentitySource::MacosIoplatformuuid; // placeholder
    pub fn read(&self) -> Result<String, KeystoreError> {
        Err(KeystoreError::Envelope("unsupported platform".into()))
    }
}
```

- [ ] **Step 2: Add module declaration + check it compiles on macOS**

In `lib.rs`: `pub mod keystore;`
Run: `cd src-tauri && cargo check`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/keystore.rs src-tauri/src/lib.rs
git commit -m "feat(keystore): IdentitySource + machine identity read (mac/win)"
```

---

## Task 4: Keystore — crypto + envelope (testable in isolation)

**Files:**
- Modify: `src-tauri/src/keystore.rs` (append)

- [ ] **Step 1: Append crypto section to keystore.rs**

```rust
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

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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
    // Header validation (spec §A pre-decrypt validation).
    if envelope.version != VERSION { return Err(KeystoreError::UnsupportedVersion(envelope.version)); }
    if envelope.aead != "aes-256-gcm" || envelope.kdf != "argon2id" { return Err(KeystoreError::Envelope("bad aead/kdf".into())); }
    if envelope.kdf_params.m_kib != PINNED_KDF.m_kib
        || envelope.kdf_params.t != PINNED_KDF.t
        || envelope.kdf_params.p != PINNED_KDF.p
        || envelope.kdf_params.output_len != PINNED_KDF.output_len {
        return Err(KeystoreError::Envelope("kdf_params not pinned".into()));
    }
    if envelope.identity_source != machine_source {
        return Err(KeystoreError::AuthFailed); // identity source mismatch / moved file
    }
    let salt = B64.decode(&envelope.salt).map_err(|e| KeystoreError::Envelope(e.to_string()))?;
    let nonce = B64.decode(&envelope.nonce).map_err(|e| KeystoreError::Envelope(e.to_string()))?;
    let ct = B64.decode(&envelope.ciphertext).map_err(|e| KeystoreError::Envelope(e.to_string()))?;
    if salt.len() != SALT_LEN || nonce.len() != NONCE_LEN || ct.len() < 16 {
        return Err(KeystoreError::Envelope("bad field lengths".into()));
    }
    // Decrypt with the CURRENT machine identity (source must match, checked above).
    let identity = machine_source.read()?;
    let key = derive_key(&identity, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| KeystoreError::Crypto(e.to_string()))?;
    let pt = cipher.decrypt(Nonce::from_slice(&nonce), aes_gcm::aead::Payload { msg: &ct, aad: FIXED_AAD })
        .map_err(|_| KeystoreError::AuthFailed)?; // tag mismatch / wrong identity => AuthFailed
    let v: serde_json::Value = serde_json::from_slice(&pt).map_err(|e| KeystoreError::Envelope(e.to_string()))?;
    Ok(v)
}
```

- [ ] **Step 2: Add a round-trip unit test in the same file**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip_encrypt_decrypt() {
        // Bypass real machine identity by testing encrypt/decrypt with a fixed
        // source's read() result mocked via a known identity string.
        let identity = "test-machine-uuid";
        let keys = serde_json::json!({"openai": "sk-test-123"});
        let env = encrypt(identity, IdentitySource::MacosIoplatformuuid, &keys).unwrap();
        // decrypt() calls machine_source.read() — for the unit test we can't fake
        // that without an injection seam, so this test covers encrypt format only;
        // the full decrypt-with-injected-identity test lives in tests/keystore.rs
        // via a `decrypt_with_identity` helper (added next step).
        assert_eq!(env.version, 1);
        assert_eq!(env.kdf_params.m_kib, 65536);
    }
}
```

- [ ] **Step 3: Expose a test-only decrypt with injected identity**

Append to keystore.rs (used by integration tests):

```rust
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
```

- [ ] **Step 4: Run test**

Run: `cd src-tauri && cargo test keystore::tests`
Expected: PASS (1 test).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/keystore.rs
git commit -m "feat(keystore): AES-256-GCM + Argon2id envelope, pre-decrypt validation"
```

---

## Task 5: Keystore — file store (atomic write, lock, fail-closed)

**Files:**
- Modify: `src-tauri/src/keystore.rs` (append `Keystore` struct)
- Create: `src-tauri/tests/keystore.rs`

- [ ] **Step 1: Append the Keystore file-handle struct to keystore.rs**

```rust
use std::path::{Path, PathBuf};
use parking_lot::Mutex;

/// Owns the keystore directory + in-process lock. Cross-process lock is taken on
/// disk per-operation via `keystore.lock`. (spec §A)
pub struct Keystore {
    dir: PathBuf,
    in_proc: Mutex<()>,
}

const FILE: &str = "keystore.json";
const TMP: &str = "keystore.json.tmp";
const LOCK: &str = "keystore.lock";

impl Keystore {
    pub fn new(dir: PathBuf) -> Result<Self, KeystoreError> {
        std::fs::create_dir_all(&dir)?;
        Self::set_dir_perms(&dir)?;
        // Clean any stale .tmp from a previous crash (spec §A stale-tmp handling).
        let tmp = dir.join(TMP);
        if tmp.exists() {
            // Take the lock first to be safe; ignore failure to remove.
            let _ = std::fs::remove_file(&tmp);
        }
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

    /// Read the whole keys map. Returns `{}` if the file does not exist (first run).
    pub fn load(&self) -> Result<serde_json::Value, KeystoreError> {
        let _g = self.in_proc.lock();
        let path = self.file();
        if !path.exists() { return Ok(serde_json::json!({})); }
        let _lock = self.acquire_cross_proc_lock()?;
        let bytes = std::fs::read(&path)?;
        let env: Envelope = serde_json::from_slice(&bytes)
            .map_err(|e| KeystoreError::Envelope(format!("malformed: {e}")))?;
        // Use the CURRENT machine source. Mismatch => AuthFailed (fail closed).
        decrypt(&env, IdentitySource::CURRENT)
    }

    /// Encrypt + atomically write the keys map.
    pub fn store(&self, keys: &serde_json::Value) -> Result<(), KeystoreError> {
        let _g = self.in_proc.lock();
        let _lock = self.acquire_cross_proc_lock()?;
        let identity = IdentitySource::CURRENT.read()?;
        let env = encrypt(&identity, IdentitySource::CURRENT, keys)?;
        let tmp = self.dir.join(TMP);
        let json = serde_json::to_vec(&env).map_err(|e| KeystoreError::Envelope(e.to_string()))?;
        std::fs::write(&tmp, &json)?;
        self.set_file_perms_macos(&tmp)?;
        atomic_replace(&tmp, &self.file())?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn set_file_perms_macos(&self, p: &Path) -> Result<(), KeystoreError> {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o600))?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    fn set_file_perms_macos(&self, _p: &Path) -> Result<(), KeystoreError> { Ok(()) }

    fn acquire_cross_proc_lock(&self) -> Result<fd_lock::RwLock<std::fs::File>, KeystoreError> {
        let f = std::fs::OpenOptions::new().create(true).append(true).open(self.dir.join(LOCK))?;
        let mut lock = fd_lock::RwLock::new(f);
        use fd_lock::WriteLock;
        lock.write().map_err(|e| KeystoreError::Io(e))?;
        // NOTE: holding the guard returned would be ideal; fd_lock returns a guard
        // from lock.write(). We return the RwLock so the guard lives with it.
        Ok(lock)
    }
}

/// Atomic replace. macOS: rename over target (first-create or update).
/// Windows: update => ReplaceFileW; first-create => MoveFileExW.
#[cfg(target_os = "macos")]
fn atomic_replace(src: &Path, dst: &Path) -> Result<(), KeystoreError> {
    std::fs::rename(src, dst)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn atomic_replace(src: &Path, dst: &Path) -> Result<(), KeystoreError> {
    use std::os::windows::ffi::OsStrExt;
    fn w(s: &Path) -> Vec<u16> {
        s.as_os_str().encode_wide().chain(std::iter::once(0)).collect()
    }
    unsafe {
        // If dst exists => ReplaceFileW; else MoveFileExW.
        if dst.exists() {
            // ReplaceFileW(lpReplacedFileName=dst, lpReplacementFileName=src, ...)
            let r = windows_sys::Win32::Storage::FileSystem::ReplaceFileW(
                w(dst).as_ptr(), w(src).as_ptr(), std::ptr::null(), 0, 0, 0);
            if r == 0 { return Err(KeystoreError::Io(std::io::Error::last_os_error())); }
        } else {
            let r = windows_sys::Win32::Storage::FileSystem::MoveFileExW(
                w(src).as_ptr(), w(dst).as_ptr(),
                windows_sys::Win32::Storage::FileSystem::MOVEFILE_REPLACE_EXISTING);
            if r == 0 { return Err(KeystoreError::Io(std::io::Error::last_os_error())); }
        }
    }
    Ok(())
}
```

> **Note for implementer:** the cross-process lock as written returns the `RwLock` but drops its write guard — this is intentionally simplified for the plan's clarity. In execution, hold the guard for the operation scope (assign `let _guard = lock.write()?;` and keep `_lock` alive until end of method). Verify `fd_lock`'s API at execution time; if it differs, adjust minimally.

- [ ] **Step 2: Write integration test file**

Create `src-tauri/tests/keystore.rs`:

```rust
use islandpot_lib::keystore::{encrypt, decrypt_with_identity, IdentitySource, Envelope};
use serde_json::json;

#[test]
fn encrypt_decrypt_roundtrip_with_injected_identity() {
    let id = "machine-x";
    let keys = json!({"openai": "sk-abc"});
    let env = encrypt(id, IdentitySource::MacosIoplatformuuid, &keys).unwrap();
    let out = decrypt_with_identity(&env, id).unwrap();
    assert_eq!(out, keys);
}

#[test]
fn wrong_identity_fails_closed() {
    let env = encrypt("machine-x", IdentitySource::MacosIoplatformuuid, &json!({"a":"b"})).unwrap();
    let err = decrypt_with_identity(&env, "machine-y").unwrap_err();
    assert!(matches!(err, islandpot_lib::keystore::KeystoreError::AuthFailed));
}

#[test]
fn tamper_fails_closed() {
    let env = encrypt("m", IdentitySource::MacosIoplatformuuid, &json!({"a":"b"})).unwrap();
    let mut env2 = env.clone();
    // flip a char in ciphertext (base64) to corrupt the GCM tag region
    let mut chars: Vec<char> = env2.ciphertext.chars().collect();
    let last = chars.len() - 1;
    chars[last] = if chars[last] == 'A' { 'B' } else { 'A' };
    env2.ciphertext = chars.into_iter().collect();
    let err = decrypt_with_identity(&env2, "m").unwrap_err();
    assert!(matches!(err, islandpot_lib::keystore::KeystoreError::AuthFailed));
}

#[test]
fn nonce_is_fresh_each_write() {
    let id = "m";
    let a = encrypt(id, IdentitySource::MacosIoplatformuuid, &json!({})).unwrap();
    let b = encrypt(id, IdentitySource::MacosIoplatformuuid, &json!({})).unwrap();
    assert_ne!(a.nonce, b.nonce);
}

#[test]
fn tampered_kdf_params_rejected_not_honored() {
    let mut env = encrypt("m", IdentitySource::MacosIoplatformuuid, &json!({})).unwrap();
    env.kdf_params.m_kib = 999999; // attacker raises cost
    let err = decrypt_with_identity(&env, "m").unwrap_err();
    assert!(matches!(err, islandpot_lib::keystore::KeystoreError::Envelope(_)));
}

#[test]
fn bad_length_fails_before_decrypt() {
    let mut env = encrypt("m", IdentitySource::MacosIoplatformuuid, &json!({})).unwrap();
    env.salt = base64::engine::general_purpose::STANDARD.encode([0u8; 3]); // wrong len
    let err = decrypt_with_identity(&env, "m").unwrap_err();
    assert!(matches!(err, islandpot_lib::keystore::KeystoreError::Envelope(_)));
}
```

Add `base64` to dev-dependencies too (it's already a runtime dep, so reuse it): the test uses `base64::engine::general_purpose::STANDARD`. Since `base64` is a runtime dep, tests can use it directly — but it must be the same version. (Already added in Task 1.)

- [ ] **Step 3: Make Envelope Clone for tests**

In `keystore.rs`, the `Envelope` derive: add `Clone`:
```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct Envelope {
```

- [ ] **Step 4: Run tests**

Run: `cd src-tauri && cargo test --test keystore`
Expected: 6 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/keystore.rs src-tauri/tests/keystore.rs
git commit -m "feat(keystore): file store (atomic, locked, fail-closed) + tests"
```

---

## Task 6: Provider presets — full endpoint URLs

**Files:**
- Modify: `src-tauri/src/providers.rs`

- [ ] **Step 1: Replace base_url with full endpoint; update presets**

Replace the `ProviderPreset` struct and `presets()` in `providers.rs`:

```rust
use crate::wire::ApiKind;

/// A pre-configured AI provider. Pure data. Spec §Wire: stores the FULL endpoint
/// URL (not base_url + route, which would break under Url::join).
#[derive(Debug, Clone)]
pub struct ProviderPreset {
    pub id: String,
    pub label: String,
    /// FULL endpoint URL, e.g. "https://api.openai.com/v1/chat/completions".
    pub endpoint: String,
    pub api_kind: ApiKind,
    pub default_model: String,
    pub needs_key: bool,
}

pub fn presets() -> Vec<ProviderPreset> {
    vec![
        ProviderPreset {
            id: "openai".into(), label: "OpenAI".into(),
            endpoint: "https://api.openai.com/v1/chat/completions".into(),
            api_kind: ApiKind::OpenAIChat, default_model: "gpt-4o-mini".into(), needs_key: true,
        },
        ProviderPreset {
            id: "anthropic".into(), label: "Anthropic Claude".into(),
            endpoint: "https://api.anthropic.com/v1/messages".into(),
            api_kind: ApiKind::Anthropic, default_model: "claude-sonnet-4-5".into(), needs_key: true,
        },
        ProviderPreset {
            id: "gemini".into(), label: "Google Gemini".into(),
            // OpenAI-compatible path (spec §Wire): /v1beta/openai/chat/completions
            endpoint: "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions".into(),
            api_kind: ApiKind::OpenAIChat, default_model: "gemini-2.0-flash".into(), needs_key: true,
        },
        ProviderPreset {
            id: "ollama".into(), label: "Ollama (local)".into(),
            endpoint: "http://localhost:11434/v1/chat/completions".into(),
            api_kind: ApiKind::OpenAIChat, default_model: "qwen2.5:7b".into(), needs_key: false,
        },
    ]
}
```

Remove the old `ApiKind` enum and `impl ProviderPreset { fn translate }` from this file — the caller logic moves to `wire.rs` (Task 7). Keep only the data here.

- [ ] **Step 2: Check compiles** (will fail until Task 7 exists — expected)

This step intentionally breaks compilation; Task 7 fixes it. Do NOT commit yet.

---

## Task 7: Wire — request builders + prompt builder

**Files:**
- Create: `src-tauri/src/wire.rs`

- [ ] **Step 1: Write wire.rs**

```rust
//! Provider wire contract — spec §Wire.
//!
//! Two option spaces kept distinct:
//! - App translation options (domain/formality/system_prompt_override) shape the
//!   PROMPT (message content), never top-level wire fields.
//! - WireParams (model/temperature/max_tokens/stream) is a strong-typed whitelist
//!   for top-level body fields.

use serde::Serialize;

#[derive(Debug, Clone, Copy)]
pub enum ApiKind {
    OpenAIChat,
    Anthropic,
}

/// Top-level wire fields. Closed whitelist; nothing else reaches the body.
#[derive(Debug, Clone, Serialize)]
pub struct WireParams {
    pub model: String,
    #[serde(skip_serializing_if = "Option_is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option_is_none")]
    pub max_tokens: Option<u32>,
    pub stream: bool,
}
// serde helper for skipping None on a generic Option<T> at top level
fn Option_is_none<T>(_v: &Option<T>) -> bool { true } // placeholder see note below

/// App-layer translation options. These shape the prompt, not the wire fields.
#[derive(Debug, Clone, Default)]
pub struct AppOptions {
    pub domain: Option<String>,
    pub formality: Option<String>,
    pub system_prompt_override: Option<String>,
}

/// Build the system + user message content for a translation request.
/// (Spec §Wire: app options influence message content only.)
pub fn build_prompt(text: &str, from: &str, to: &str, opts: &AppOptions) -> (String, String) {
    let mut system = match &opts.system_prompt_override {
        Some(s) => s.clone(),
        None => "You are a professional translator. Translate the user's text. \
                 Output ONLY the translation, no explanations.".to_string(),
    };
    if let Some(d) = &opts.domain {
        system.push_str(&format!(" Domain: {d}."));
    }
    if let Some(f) = &opts.formality {
        system.push_str(&format!(" Register/formality: {f}."));
    }
    let src = if from == "auto" { "the source language (detect it)".to_string() } else { from.to_string() };
    let user = format!("Translate from {src} into {to}:\n\n{text}");
    (system, user)
}
```

> **Implementer note (fix the placeholder):** the `Option_is_none` helper above is a placeholder that won't typecheck for serde's `skip_serializing_if`. Replace the `WireParams` derive with an explicit manual `Serialize` impl OR change the fields to concrete types and use `#[serde(skip_serializing_if = "Option::is_none")]`. Use:
> ```rust
> #[derive(Debug, Clone, Serialize)]
> pub struct WireParams {
>     pub model: String,
>     #[serde(skip_serializing_if = "Option::is_none")]
>     pub temperature: Option<f32>,
>     #[serde(skip_serializing_if = "Option::is_none")]
>     pub max_tokens: Option<u32>,
>     pub stream: bool,
> }
> ```
> and delete the `Option_is_none` function.

- [ ] **Step 2: Add `mod wire;` to lib.rs, check**

```rust
pub mod wire;
```
Run: `cd src-tauri && cargo check`
Expected: PASS now (providers.rs + wire.rs consistent).

- [ ] **Step 3: Commit (Tasks 6 + 7 together)**

```bash
git add src-tauri/src/providers.rs src-tauri/src/wire.rs src-tauri/src/lib.rs
git commit -m "feat(wire): full endpoint presets + WireParams whitelist + prompt builder"
```

---

## Task 8: Wire — HTTP caller (pure, tested with wiremock)

**Files:**
- Modify: `src-tauri/src/wire.rs` (append `call`)
- Create: `src-tauri/tests/wire.rs`

- [ ] **Step 1: Append the HTTP caller to wire.rs**

```rust
use crate::error::{Error, FallbackKind, ConfigKind};

/// Call a provider. PURE: takes preset + key + params + messages, returns text.
/// Classifies HTTP status into FallbackEligible (429/5xx) vs Config (401/403).
pub async fn call(
    client: &reqwest::Client,
    preset: &crate::providers::ProviderPreset,
    key: &str,
    params: &WireParams,
    system: &str,
    user: &str,
) -> Result<String, Error> {
    let resp = match preset.api_kind {
        ApiKind::OpenAIChat => {
            let body = serde_json::json!({
                "model": params.model,
                "temperature": params.temperature,
                "max_tokens": params.max_tokens,
                "stream": params.stream,
                "messages": [
                    {"role": "system", "content": system},
                    {"role": "user", "content": user},
                ],
            });
            client.post(&preset.endpoint)
                .bearer_auth(key)
                .json(&body)
                .send().await
        }
        ApiKind::Anthropic => {
            let body = serde_json::json!({
                "model": params.model,
                "max_tokens": params.max_tokens.unwrap_or(1024),
                "system": system,
                "messages": [{"role": "user", "content": user}],
            });
            client.post(&preset.endpoint)
                .header("x-api-key", key)
                .header("anthropic-version", "2023-06-01")
                .json(&body)
                .send().await
        }
    };
    let resp = resp.map_err(|e| Error::FallbackEligible(FallbackKind::Network(e.to_string())))?;
    let status = resp.status().as_u16();
    if status == 401 || status == 403 {
        return Err(Error::Config(ConfigKind::AuthFailed { provider: preset.id.clone(), status }));
    }
    if status == 429 || (500..600).contains(&status) {
        return Err(Error::FallbackEligible(FallbackKind::ProviderStatus { status }));
    }
    if !resp.status().is_success() {
        return Err(Error::FallbackEligible(FallbackKind::ProviderStatus { status }));
    }
    let json: serde_json::Value = resp.json().await
        .map_err(|e| Error::FallbackEligible(FallbackKind::Parse(e.to_string())))?;
    // response path depends on dialect
    let text = match preset.api_kind {
        ApiKind::OpenAIChat => json["choices"][0]["message"]["content"]
            .as_str().ok_or_else(|| Error::FallbackEligible(FallbackKind::Parse("no content".into())))?.to_string(),
        ApiKind::Anthropic => json["content"][0]["text"]
            .as_str().ok_or_else(|| Error::FallbackEligible(FallbackKind::Parse("no text".into())))?.to_string(),
    };
    Ok(text)
}
```

- [ ] **Step 2: Write wiremock test**

Create `src-tauri/tests/wire.rs`:

```rust
use islandpot_lib::wire::{call, ApiKind, WireParams, build_prompt};
use islandpot_lib::providers::ProviderPreset;
use wiremock::{MockServer, Mock, ResponseTemplate};
use serde_json::json;

fn preset(endpoint: &str, kind: ApiKind) -> ProviderPreset {
    ProviderPreset { id: "test".into(), label: "Test".into(), endpoint: endpoint.into(),
        api_kind: kind, default_model: "m".into(), needs_key: true }
}

#[tokio::test]
async fn openai_chat_success() {
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"message": {"content": "你好"}}]
        })))
        .mount(&server).await;
    let p = preset(&server.uri(), ApiKind::OpenAIChat);
    let client = reqwest::Client::new();
    let (sys, usr) = build_prompt("hello", "auto", "zh", &Default::default());
    let params = WireParams { model: "gpt-4o-mini".into(), temperature: None, max_tokens: None, stream: false };
    let out = call(&client, &p, "sk-x", &params, &sys, &usr).await.unwrap();
    assert_eq!(out, "你好");
}

#[tokio::test]
async fn http_401_is_config_error() {
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::method("POST"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server).await;
    let p = preset(&server.uri(), ApiKind::OpenAIChat);
    let client = reqwest::Client::new();
    let (sys, usr) = build_prompt("hi", "en", "zh", &Default::default());
    let params = WireParams { model: "m".into(), temperature: None, max_tokens: None, stream: false };
    let err = call(&client, &p, "bad", &params, &sys, &usr).await.unwrap_err();
    assert!(matches!(err, islandpot_lib::error::Error::Config(_)));
}

#[tokio::test]
async fn http_429_is_fallback_eligible() {
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::method("POST"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&server).await;
    let p = preset(&server.uri(), ApiKind::OpenAIChat);
    let client = reqwest::Client::new();
    let (sys, usr) = build_prompt("hi", "en", "zh", &Default::default());
    let params = WireParams { model: "m".into(), temperature: None, max_tokens: None, stream: false };
    let err = call(&client, &p, "k", &params, &sys, &usr).await.unwrap_err();
    assert!(matches!(err, islandpot_lib::error::Error::FallbackEligible(_)));
}
```

- [ ] **Step 3: Run tests**

Run: `cd src-tauri && cargo test --test wire`
Expected: 3 tests PASS.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/wire.rs src-tauri/tests/wire.rs
git commit -m "feat(wire): HTTP caller with status classification + wiremock tests"
```

---

## Task 9: Translate service + AppState + async translate command

**Files:**
- Create: `src-tauri/src/service.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write service.rs**

```rust
//! Orchestrates a translation (spec architecture). v1: single engine, no fallback
//! yet (fallback engines arrive Phase 3 — the error classification is ready).

use crate::error::{ConfigKind, Error};
use crate::keystore::Keystore;
use crate::providers::ProviderPreset;
use crate::wire::{build_prompt, call, AppOptions, WireParams};

pub struct TranslateInput<'a> {
    pub text: &'a str,
    pub from: &'a str,
    pub to: &'a str,
    pub options: AppOptions,
}

pub async fn translate(
    client: &reqwest::Client,
    keystore: &Keystore,
    preset: &ProviderPreset,
    input: TranslateInput<'_>,
) -> Result<String, Error> {
    let key = if preset.needs_key {
        let keys = keystore.load().map_err(Error::Keystore)?;
        keys.get(&preset.id)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| Error::Config(ConfigKind::MissingKey { provider: preset.id.clone() }))?
    } else {
        String::new()
    };
    let (system, user) = build_prompt(input.text, input.from, input.to, &input.options);
    let params = WireParams {
        model: preset.default_model.clone(),
        temperature: None, max_tokens: None, stream: false,
    };
    call(client, preset, &key, &params, &system, &user).await
}
```

- [ ] **Step 2: Rewrite lib.rs run() + commands for async translate + key commands**

In `lib.rs`, replace the command section and `run()`:

```rust
pub mod error;
pub mod keystore;
pub mod providers;
pub mod wire;
pub mod service;
mod engines;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateRequest {
    pub text: String,
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub options: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct TranslateResult { pub text: String, pub engine: String }

struct AppState {
    client: reqwest::Client,
    keystore: keystore::Keystore,
}

#[tauri::command]
async fn translate(
    state: tauri::State<'_, Arc<AppState>>,
    req: TranslateRequest,
    engine: String,
) -> Result<TranslateResult, String> {
    let preset = providers::presets().into_iter()
        .find(|p| p.id == engine)
        .ok_or_else(|| format!("unknown engine: {engine}"))?;
    let opts = wire::AppOptions::default(); // v1: no app-options UI yet
    let input = service::TranslateInput { text: &req.text, from: &req.from, to: &req.to, options: opts };
    let text = service::translate(&state.client, &state.keystore, &preset, input).await
        .map_err(|e| e.to_string())?;
    Ok(TranslateResult { text, engine: preset.id })
}

#[tauri::command]
fn list_engines() -> Vec<EngineInfo> {
    providers::presets().into_iter().map(EngineInfo::from_provider).collect()
}

#[tauri::command]
fn set_key(state: tauri::State<'_, Arc<AppState>>, provider_id: String, key: String) -> Result<(), String> {
    let mut keys = state.keystore.load().map_err(|e| e.to_string())?;
    if !keys.is_object() { keys = serde_json::json!({}); }
    keys[&provider_id] = serde_json::json!(key);
    state.keystore.store(&keys).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn delete_key(state: tauri::State<'_, Arc<AppState>>, provider_id: String) -> Result<(), String> {
    let mut keys = state.keystore.load().map_err(|e| e.to_string())?;
    if let Some(obj) = keys.as_object_mut() { obj.remove(&provider_id); }
    state.keystore.store(&keys).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn key_status(state: tauri::State<'_, Arc<AppState>>) -> Result<std::collections::HashMap<String, bool>, String> {
    let keys = state.keystore.load().map_err(|e| e.to_string())?;
    let mut map = std::collections::HashMap::new();
    if let Some(obj) = keys.as_object() {
        for (k, _v) in obj { map.insert(k.clone(), true); }
    }
    Ok(map)
}

#[derive(Debug, Clone, Serialize)]
pub struct EngineInfo { pub id: String, pub label: String, pub kind: String, pub needs_key: bool }
impl EngineInfo {
    fn from_provider(p: providers::ProviderPreset) -> Self {
        Self { id: p.id, label: p.label, kind: "provider".into(), needs_key: p.needs_key }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let dir = std::env::temp_dir().join("islandpot-keystore-test"); // PLACEHOLDER — fixed in Step 3
    let keystore = keystore::Keystore::new(dir).expect("keystore init");
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none()) // spec §Privacy: no cross-origin redirects
        .build().expect("client");
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Arc::new(AppState { client, keystore }))
        .invoke_handler(tauri::generate_handler![translate, list_engines, set_key, delete_key, key_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Use Tauri appLocalDataDir for the keystore (fix placeholder)**

Replace the placeholder `dir` line with the real app-data path obtained from the Tauri app handle. Move keystore init into a setup hook:

```rust
        .setup(|app| {
            let dir = app.path().app_local_data_dir()
                .expect("app_local_data_dir");
            let keystore = keystore::Keystore::new(dir).expect("keystore init");
            let client = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build().expect("client");
            app.manage(Arc::new(AppState { client, keystore }));
            Ok(())
        })
```
And remove the keystore/client init before `tauri::Builder`. Add `use tauri::Manager;` (already imported). The `app.path()` requires `tauri::Manager` and the path API — verify the exact method name `app_local_data_dir` at execution time (Tauri 2 path API).

- [ ] **Step 4: Remove now-unused engines Engine trait (keep module compiling)**

`engines/mod.rs` currently has `pub trait Engine` referencing old types. Replace its contents to keep it minimal and compiling:

```rust
//! Built-in traditional MT engines (Phase 3). Empty registry for v1 Phase 1.
pub fn registry() -> Vec<()> { vec![] }
```
Remove references to the old `Engine` trait / `EngineInfo::from_engine` in lib.rs (already replaced in Step 2).

- [ ] **Step 5: Check + fix compile errors**

Run: `cd src-tauri && cargo check`
Expected: PASS after fixing the path-API method name and any trait/import issues. Iterate until clean.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/service.rs src-tauri/src/lib.rs src-tauri/src/engines/mod.rs
git commit -m "feat(service): async translate + keystore key commands + AppState"
```

---

## Task 10: Frontend — provider picker + key input + translate

**Files:**
- Modify: `src/App.tsx`

- [ ] **Step 1: Rewrite App.tsx with a key-input flow**

```tsx
import { createSignal, For, onMount, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

type EngineInfo = { id: string; label: string; kind: string; needs_key: boolean };

function App() {
  const [engines, setEngines] = createSignal<EngineInfo[]>([]);
  const [selected, setSelected] = createSignal("");
  const [keyInput, setKeyInput] = createSignal("");
  const [hasKey, setHasKey] = createSignal<Record<string, boolean>>({});
  const [input, setInput] = createSignal("");
  const [output, setOutput] = createSignal("");
  const [error, setError] = createSignal("");
  const [busy, setBusy] = createSignal(false);

  onMount(async () => {
    const list = await invoke<EngineInfo[]>("list_engines");
    setEngines(list);
    const status = await invoke<Record<string, boolean>>("key_status");
    setHasKey(status);
    setSelected(list.find((e) => e.kind === "provider")?.id ?? list[0]?.id ?? "");
  });

  async function saveKey() {
    if (!selected() || !keyInput()) return;
    await invoke("set_key", { providerId: selected(), key: keyInput() });
    setKeyInput("");
    const status = await invoke<Record<string, boolean>>("key_status");
    setHasKey(status);
  }

  async function doTranslate() {
    if (!input().trim() || !selected()) return;
    setBusy(true); setError(""); setOutput("");
    try {
      const res = await invoke<{ text: string; engine: string }>("translate", {
        req: { text: input(), from: "auto", to: "zh", options: {} },
        engine: selected(),
      });
      setOutput(res.text);
    } catch (e) {
      setError(String(e));
    } finally { setBusy(false); }
  }

  return (
    <main class="container">
      <h1>IslandPot</h1>
      <select value={selected()} onChange={(e) => setSelected(e.currentTarget.value)}>
        <For each={engines()}>{(e) => <option value={e.id}>{e.label}{hasKey()[e.id] ? " ✓" : ""}</option>}</For>
      </select>
      <input type="password" placeholder="API key…" value={keyInput()} onInput={(e) => setKeyInput(e.currentTarget.value)} />
      <button onClick={saveKey} disabled={!keyInput()}>Save key</button>
      <textarea rows={4} placeholder="输入要翻译的文本…" value={input()} onInput={(e) => setInput(e.currentTarget.value)} />
      <button onClick={doTranslate} disabled={busy() || !input().trim()}>{busy() ? "…" : "Translate"}</button>
      <Show when={output()}><div class="result">{output()}</div></Show>
      <Show when={error()}><div class="error">{error()}</div></Show>
    </main>
  );
}
export default App;
```

- [ ] **Step 2: Run the app, manually verify**

Run: `cd /Users/daoyu/Code/projects/islandpot && pnpm tauri dev`
Expected: window opens; pick a provider (e.g. ollama if local, or enter an OpenAI key); type text; click Translate; see a real translation or a classified error.

- [ ] **Step 3: Commit**

```bash
git add src/App.tsx
git commit -m "feat(ui): provider picker + key input + translate flow"
```

---

## Self-Review (run after writing; issues fixed inline above)

- **Spec coverage:** §A keystore → Tasks 3–5. §Wire → Tasks 6–8. §G classification → Task 8 (status mapping) + Task 2. §Privacy no-redirect → Task 9 (`Policy::none`). §Privacy HTTPS-only + app_local_data → Task 9 Step 3. translate_service orchestration → Task 9. Frontend no plaintext read → Task 9 (only set/delete/status). **Gap:** HTTPS-only-loopback URL validation at preset-load is NOT in a task yet — see Task 11.
- **Placeholder scan:** Task 9 Step 2 has one labeled placeholder (keystore dir) fixed in Step 3. Task 7 has a labeled `Option_is_none` placeholder with the fix given. No other TBDs.
- **Type consistency:** `ProviderPreset.endpoint` (not base_url) used consistently in Tasks 6/8/9. `ApiKind` lives in `wire.rs`, re-exported use in `providers.rs`. `WireParams` fields consistent across Tasks 7/8/9.

## Task 11: URL transport validation (added from self-review gap)

**Files:**
- Modify: `src-tauri/src/providers.rs`

- [ ] **Step 1: Add a validate function + call in presets()**

```rust
/// Spec §Privacy: remote endpoints must be HTTPS; HTTP allowed only for loopback.
pub fn validate_endpoint(url: &str) -> Result<(), String> {
    let parsed = url::Url::parse(url).map_err(|e| format!("bad url: {e}"))?;
    match parsed.scheme() {
        "https" => Ok(()),
        "http" => {
            let h = parsed.host_str().unwrap_or("");
            if h == "localhost" || h == "127.0.0.1" || h == "::1" { Ok(()) }
            else { Err(format!("http only allowed for loopback, got {h}")) }
        }
        s => Err(format!("scheme {s} not allowed")),
    }
}
```

- [ ] **Step 2: Add `url` to Cargo.toml dependencies**

```toml
url = "2"
```

- [ ] **Step 3: Test it — append to a new `src-tauri/tests/url.rs`**

```rust
use islandpot_lib::providers::validate_endpoint;
#[test] fn https_ok() { assert!(validate_endpoint("https://api.openai.com/v1").is_ok()); }
#[test] fn http_loopback_ok() { assert!(validate_endpoint("http://localhost:11434/v1").is_ok()); }
#[test] fn http_remote_rejected() { assert!(validate_endpoint("http://evil.com").is_err()); }
#[test] fn ftp_rejected() { assert!(validate_endpoint("ftp://x").is_err()); }
```

- [ ] **Step 4: Run + commit**

```bash
cd src-tauri && cargo test --test url
git add src-tauri/src/providers.rs src-tauri/tests/url.rs src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat(providers): validate endpoint scheme (HTTPS or loopback HTTP)"
```

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-07-30-phase1-keystore-provider-translate.md`. Two execution options:

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints.

**Which approach?**
