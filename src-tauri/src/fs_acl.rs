//! Shared file/directory permission helpers.
//!
//! Extracted from `keystore.rs` so both the keystore and the SQLite database
//! use the same permission logic.
//!
//! - Unix: `0o700` for directories, `0o600` for files (owner-only).
//! - Windows: current-user-owner protected DACL (one ACE = current user,
//!   GENERIC_ALL, SE_DACL_PROTECTED blocks inheritance). Directories get an
//!   inheritable ACE so files created inside auto-receive current-user control;
//!   files get a non-inheritable leaf ACE.

use std::path::Path;

#[derive(Debug)]
pub enum AclError {
    Io(std::io::Error),
    /// Windows ACL operation failed (SetEntriesInAclW / SetNamedSecurityInfoW).
    Win32(String),
}

impl std::fmt::Display for AclError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AclError::Io(e) => write!(f, "io: {e}"),
            AclError::Win32(s) => write!(f, "win32 acl: {s}"),
        }
    }
}

impl std::error::Error for AclError {}

impl From<std::io::Error> for AclError {
    fn from(e: std::io::Error) -> Self { AclError::Io(e) }
}

// ── Backup error type (staged, not generic Io) ─────────────────────────

/// Staged backup error: identifies which phase failed.
#[derive(Debug)]
pub enum BackupError {
    /// `OpenOptions::create_new` on the staging path failed.
    CreateStaging(std::io::Error),
    /// `secure_file` on the staging path failed.
    SecureStaging(AclError),
    /// `write_all` or `flush` on the staging file failed.
    WriteStaging(std::io::Error),
    /// `sync_all` on the staging file failed.
    SyncStaging(std::io::Error),
    /// Publishing (hard_link / MoveFileExW) failed.
    Publish(std::io::Error),
    /// Parent-directory `sync_all` failed (Unix only).
    SyncParent(std::io::Error),
}

impl std::fmt::Display for BackupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackupError::CreateStaging(e) => write!(f, "create staging: {e}"),
            BackupError::SecureStaging(e) => write!(f, "secure staging: {e}"),
            BackupError::WriteStaging(e) => write!(f, "write staging: {e}"),
            BackupError::SyncStaging(e) => write!(f, "sync staging: {e}"),
            BackupError::Publish(e) => write!(f, "publish backup: {e}"),
            BackupError::SyncParent(e) => write!(f, "sync parent dir: {e}"),
        }
    }
}

impl std::error::Error for BackupError {}

// ── Unix ──────────────────────────────────────────────────────────────

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> Result<(), AclError> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))?;
    Ok(())
}

/// Secure a directory: 0o700 on Unix; protected DACL (inheritable) on Windows.
/// Failure is a hard error — the directory MUST be secured before any data
/// is written into it.
pub fn secure_dir(dir: &Path) -> Result<(), AclError> {
    #[cfg(unix)]
    { set_mode(dir, 0o700) }
    #[cfg(windows)]
    { set_win32_owner_dacl(dir, true) }
    #[cfg(not(any(unix, windows)))]
    { let _ = dir; Ok(()) }
}

/// Secure a file: 0o600 on Unix; protected DACL (non-inheritable) on Windows.
/// Failure is a hard error — the file MUST be secured before it is published
/// as a backup or canonical store.
pub fn secure_file(path: &Path) -> Result<(), AclError> {
    #[cfg(unix)]
    { set_mode(path, 0o600) }
    #[cfg(windows)]
    { set_win32_owner_dacl(path, false) }
    #[cfg(not(any(unix, windows)))]
    { let _ = path; Ok(()) }
}

/// RAII cleanup guard: removes the staging file on drop unless disarmed.
struct StagingGuard {
    path: std::path::PathBuf,
    armed: bool,
}

impl StagingGuard {
    fn new(path: std::path::PathBuf) -> Self {
        Self { path, armed: true }
    }
    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for StagingGuard {
    fn drop(&mut self) {
        if self.armed {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

/// Crash-safe atomic publish of a backup file (S2a P0).
///
/// Phases (all must succeed; any failure cleans up staging and returns Err):
/// 1. `create_new` (O_EXCL) staging file — atomic ownership of a unique name.
/// 2. `secure_file` on the staging path — permissions are correct before data.
/// 3. `write_all` + `flush` via the SAME writable handle that created the file.
/// 4. `sync_all` via the SAME handle — on Windows this calls
///    `FlushFileBuffers` which requires a writable handle. Using `File::open`
///    (read-only) would produce Access Denied on Windows.
/// 5. Publish: Unix `hard_link` (no-clobber) or Windows `MoveFileExW`
///    (MOVEFILE_WRITE_THROUGH, no replace). If final already exists,
///    the staging file is cleaned up and `Ok(())` is returned.
/// 6. Sync parent directory (Unix) so the new entry is durable.
///
/// The final backup path is only ever observable as a complete, secured,
/// synced file. A crash before publish leaves at most a staging file (cleaned
/// on the next attempt).
pub fn crash_safe_backup(
    source_bytes: &[u8],
    final_path: &Path,
    staging_dir: &Path,
) -> Result<(), BackupError> {
    use std::io::Write;

    // 0. No-clobber fast path: a prior complete backup wins.
    if final_path.exists() {
        return Ok(());
    }

    // 1. create_new (O_EXCL) on staging — atomic, unique name.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let stem = final_path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "backup".to_string());
    let staging = staging_dir.join(format!(
        ".{stem}-staging-{}-{}",
        now.as_secs(),
        now.subsec_nanos()
    ));

    let mut guard = StagingGuard::new(staging.clone());

    // Create the staging file with a WRITABLE handle via create_new (O_EXCL).
    // This handle is kept alive through write + sync so FlushFileBuffers
    // has the write access it needs on Windows.
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&staging)
        .map_err(BackupError::CreateStaging)?;

    // 2. Secure the staging file BEFORE writing data.
    secure_file(&staging).map_err(BackupError::SecureStaging)?;

    // 3. Write all bytes + flush user-space buffers.
    file.write_all(source_bytes).map_err(BackupError::WriteStaging)?;
    file.flush().map_err(BackupError::WriteStaging)?;

    // 4. Fsync — same writable handle (Windows FlushFileBuffers needs write).
    file.sync_all().map_err(BackupError::SyncStaging)?;
    drop(file); // release the handle before publish

    // 5. Publish: atomically make the final path visible (no-clobber).
    publish_backup(&staging, final_path)?;
    // Staging path is consumed by the publish (moved/hard-linked).
    // On hard_link success we need to remove the staging name;
    // on MoveFileExW success the staging no longer exists.
    guard.disarm(); // publish succeeded; don't double-delete

    // 6. Sync parent directory (Unix) so the new directory entry is durable.
    #[cfg(unix)]
    {
        let dir = std::fs::File::open(staging_dir).map_err(BackupError::SyncParent)?;
        dir.sync_all().map_err(BackupError::SyncParent)?;
    }

    Ok(())
}

/// Platform-specific atomic no-clobber publish.
///
/// - Unix/macOS: `hard_link(staging → final)` — atomic, fails with
///   `AlreadyExists` if final exists. Then unlink the staging name (the
///   inode now has two links; unlinking staging leaves final as sole link).
/// - Windows: `MoveFileExW(MOVEFILE_WRITE_THROUGH)` — atomic rename that
///   does NOT set `MOVEFILE_REPLACE_EXISTING`. `ERROR_FILE_EXISTS` /
///   `ERROR_ALREADY_EXISTS` means another complete backup was published:
///   delete staging, keep the existing final backup.
#[allow(unused_variables)]
fn publish_backup(staging: &Path, final_path: &Path) -> Result<(), BackupError> {
    #[cfg(unix)]
    {
        match std::fs::hard_link(staging, final_path) {
            Ok(()) => {
                // Staging name still exists (hard link); remove it so final
                // is the sole link.
                let _ = std::fs::remove_file(staging);
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                // Another publisher won; their backup is authoritative.
                let _ = std::fs::remove_file(staging);
                Ok(())
            }
            Err(e) => Err(BackupError::Publish(e)),
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        // MoveFileExW with MOVEFILE_WRITE_THROUGH (0x8) — no REPLACE_EXISTING.
        // The move is atomic and durable; if final exists, the call fails.
        const MOVEFILE_WRITE_THROUGH: u32 = 0x8;
        let src_wide: Vec<u16> = staging.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        let dst_wide: Vec<u16> = final_path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        let ok = unsafe {
            windows_sys::Win32::Storage::FileSystem::MoveFileExW(
                src_wide.as_ptr(),
                dst_wide.as_ptr(),
                MOVEFILE_WRITE_THROUGH,
            )
        };
        if ok != 0 {
            return Ok(()); // staging moved to final; staging no longer exists
        }
        let err = std::io::Error::last_os_error();
        let raw = err.raw_os_error().unwrap_or(0);
        // ERROR_FILE_EXISTS (80) or ERROR_ALREADY_EXISTS (183) — another
        // complete backup was published. Delete staging, keep existing.
        if raw == 80 || raw == 183 {
            let _ = std::fs::remove_file(staging);
            return Ok(());
        }
        // Other errors propagate; staging will be cleaned by the RAII guard.
        Err(BackupError::Publish(err))
    }
    #[cfg(not(any(unix, windows)))]
    {
        // Fallback: simple rename (not atomic on all platforms).
        match std::fs::rename(staging, final_path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let _ = std::fs::remove_file(staging);
                Ok(())
            }
            Err(e) => Err(BackupError::Publish(e)),
        }
    }
}

// ── Windows Win32 ACL implementation ──────────────────────────────────

#[cfg(windows)]
pub fn current_user_sid() -> Result<Vec<u8>, AclError> {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::{GetTokenInformation, TOKEN_QUERY, TokenUser};
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    let mut token: HANDLE = std::ptr::null_mut();
    let ok = unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) };
    if ok == 0 {
        return Err(AclError::Io(std::io::Error::last_os_error()));
    }
    struct TokenHandle(HANDLE);
    impl Drop for TokenHandle {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.0) };
        }
    }
    let _token_guard = TokenHandle(token);

    let mut needed: u32 = 0;
    unsafe {
        GetTokenInformation(token, TokenUser, std::ptr::null_mut(), 0, &mut needed);
    }
    if needed == 0 {
        return Err(AclError::Io(std::io::Error::last_os_error()));
    }
    let mut buf: Vec<u8> = vec![0u8; needed as usize];
    let ok = unsafe {
        GetTokenInformation(token, TokenUser, buf.as_mut_ptr() as *mut _, needed, &mut needed)
    };
    if ok == 0 {
        return Err(AclError::Io(std::io::Error::last_os_error()));
    }
    Ok(buf)
}

#[cfg(windows)]
pub fn sid_from_token_user_buf(buf: &[u8]) -> Result<windows_sys::Win32::Security::PSID, AclError> {
    use windows_sys::Win32::Security::TOKEN_USER;
    if buf.len() < std::mem::size_of::<TOKEN_USER>() {
        return Err(AclError::Win32("token buffer too small for TOKEN_USER".into()));
    }
    let user: &TOKEN_USER = unsafe { &*(buf.as_ptr() as *const TOKEN_USER) };
    Ok(user.User.Sid)
}

/// Windows: set owner = current-user SID + DACL with one ACE
/// (current user, GENERIC_ALL). `inherit` controls behavior:
/// - true (directory): ACE is inheritable (SUB_CONTAINERS_AND_OBJECTS_INHERIT)
///   so child files inherit it. PROTECTED_DACL is NOT set on directories
///   because SetNamedSecurityInfoW with PROTECTED_DACL strips ACE inheritance
///   flags. Instead, the explicit current-user ACE (SET_ACCESS mode) replaces
///   any inherited ACE for this SID, so the directory is effectively locked
///   to the current user even without PROTECTED.
/// - false (file): ACE is non-inheritable (leaf). PROTECTED_DACL IS set
///   (files have no children to propagate to, so stripping is harmless).
#[cfg(windows)]
pub(crate) fn set_win32_owner_dacl(path: &Path, inherit: bool) -> Result<(), AclError> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::GENERIC_ALL;
    use windows_sys::Win32::Security::{
        ACL, ACL_REVISION_DS, AddAccessAllowedAceEx, InitializeAcl,
        CONTAINER_INHERIT_ACE, DACL_SECURITY_INFORMATION, OBJECT_INHERIT_ACE,
        OWNER_SECURITY_INFORMATION, PSID,
        PSECURITY_DESCRIPTOR, SE_DACL_PROTECTED,
        InitializeSecurityDescriptor, SetSecurityDescriptorControl,
        SetSecurityDescriptorOwner, SetSecurityDescriptorDacl, SetFileSecurityW,
    };

    let sid_buf = current_user_sid()?;
    let sid: PSID = sid_from_token_user_buf(&sid_buf)?;

    // Build ACL with AddAccessAllowedAceEx (sets AceFlags directly).
    const ACL_BUF_SIZE: usize = 128;
    let mut acl_buf: [u8; ACL_BUF_SIZE] = [0u8; ACL_BUF_SIZE];
    let acl: *mut ACL = acl_buf.as_mut_ptr() as *mut ACL;
    if unsafe { InitializeAcl(acl, ACL_BUF_SIZE as u32, ACL_REVISION_DS) } == 0 {
        return Err(AclError::Win32("InitializeAcl failed".into()));
    }
    let ace_flags = if inherit { OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE } else { 0 };
    if unsafe { AddAccessAllowedAceEx(acl, ACL_REVISION_DS, ace_flags, GENERIC_ALL, sid) } == 0 {
        return Err(AclError::Win32("AddAccessAllowedAceEx failed".into()));
    }

    // Build an absolute security descriptor. SetFileSecurityW applies it
    // WITHOUT the ACE-flag normalization that SetNamedSecurityInfoW performs.
    const SD_BUF_SIZE: usize = 256;
    let mut sd_buf: [u8; SD_BUF_SIZE] = [0u8; SD_BUF_SIZE];
    let sd: PSECURITY_DESCRIPTOR = sd_buf.as_mut_ptr() as PSECURITY_DESCRIPTOR;
    // SECURITY_DESCRIPTOR_REVISION = 1 (constant from Win32 SDK):
    if unsafe { InitializeSecurityDescriptor(sd, 1u32) } == 0 {
        return Err(AclError::Win32("InitializeSecurityDescriptor failed".into()));
    }
    // Owner = current user (bOwnerDefaulted = FALSE = 0):
    if unsafe { SetSecurityDescriptorOwner(sd, sid, 0) } == 0 {
        return Err(AclError::Win32("SetSecurityDescriptorOwner failed".into()));
    }
    // DACL present + the ACL we built (bDaclDefaulted = FALSE = 0):
    if unsafe { SetSecurityDescriptorDacl(sd, 1, acl as *const ACL, 0) } == 0 {
        return Err(AclError::Win32("SetSecurityDescriptorDacl failed".into()));
    }
    // Set SE_DACL_PROTECTED (blocks inheritance from parent) via control bits.
    // This does NOT strip ACE inheritance flags (unlike SetNamedSecurityInfoW
    // with PROTECTED_DACL_SECURITY_INFORMATION):
    if unsafe { SetSecurityDescriptorControl(sd, SE_DACL_PROTECTED, SE_DACL_PROTECTED) } == 0 {
        return Err(AclError::Win32("SetSecurityDescriptorControl (PROTECTED) failed".into()));
    }

    let path_wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let rc = unsafe {
        SetFileSecurityW(
            path_wide.as_ptr(),
            DACL_SECURITY_INFORMATION | OWNER_SECURITY_INFORMATION,
            sd,
        )
    };
    if rc == 0 {
        return Err(AclError::Win32(format!("SetFileSecurityW failed: {}", std::io::Error::last_os_error())));
    }
    Ok(())
}
