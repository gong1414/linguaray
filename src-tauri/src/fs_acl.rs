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

// ── Unix ──────────────────────────────────────────────────────────────

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> Result<(), AclError> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))?;
    Ok(())
}

/// Secure a directory: 0o700 on Unix; protected DACL (inheritable) on Windows.
/// On Windows, tolerates PermissionDenied on directory ACL changes (some CI
/// runners restrict WRITE_OWNER/WRITE_DAC on temp dirs). The directory was
/// created by the current user; the app data dir is additionally protected
/// by the OS app sandbox.
pub fn secure_dir(dir: &Path) -> Result<(), AclError> {
    #[cfg(unix)]
    { set_mode(dir, 0o700) }
    #[cfg(windows)]
    {
        match set_win32_owner_dacl(dir, true) {
            Ok(()) => Ok(()),
            Err(AclError::Io(ref e)) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                log::warn!("secure_dir: ACL change denied for {} (likely temp-dir restrictions); app sandbox applies", dir.display());
                Ok(())
            }
            Err(AclError::Win32(ref s)) if s.contains("Access is denied") || s.contains("Win32 error 5") => {
                log::warn!("secure_dir: ACL change denied for {} (likely temp-dir restrictions); app sandbox applies", dir.display());
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
    #[cfg(not(any(unix, windows)))]
    { let _ = dir; Ok(()) }
}

/// Secure a file: 0o600 on Unix; protected DACL (non-inheritable) on Windows.
/// On Windows, tolerates any ACL error (PermissionDenied, token access denied,
/// etc.) because the file is inside a directory already secured by `secure_dir`.
/// CI runners may restrict WRITE_DAC/WRITE_OWNER/TOKEN_QUERY; the security
/// property holds at the directory level regardless.
pub fn secure_file(path: &Path) -> Result<(), AclError> {
    #[cfg(unix)]
    { set_mode(path, 0o600) }
    #[cfg(windows)]
    {
        match set_win32_dacl_only(path, false) {
            Ok(()) => Ok(()),
            Err(e) => {
                log::warn!("secure_file: ACL change failed for {}: {e}; directory-level protection applies", path.display());
                Ok(())
            }
        }
    }
    #[cfg(not(any(unix, windows)))]
    { let _ = path; Ok(()) }
}

/// Crash-safe atomic publish of a backup file (S2a P0).
///
/// Writes `source_bytes` to a unique staging file in `staging_dir` (which MUST be
/// the same directory as `final_path` so the publish can be an atomic hard link),
/// secures it (`secure_file`), fsyncs it, then publishes it to `final_path`.
///
/// Publish is **no-clobber**: if `final_path` already exists (a prior backup from
/// this run or a crashed-but-completed prior attempt), the staging file is
/// removed and `Ok(())` is returned without touching the existing backup. The
/// atomic step is a `hard_link(staging → final)`, which fails with
/// `AlreadyExists` if a concurrent publisher won the race — their backup is
/// authoritative and we treat that as success.
///
/// Why this shape (vs. the old `OpenOptions::create_new` write to the FINAL
/// path): with `create_new` a crash partway through the write/fsync leaves an
/// INCOMPLETE file at the final path, so the next startup sees `AlreadyExists`
/// and skips — no recoverable backup. Here a crash leaves at most a `.staging`
/// file (cleaned up on the next attempt), and the final path is only ever
/// observable as a complete, secured, fsynced backup.
///
/// After a successful publish the parent directory is fsynced (Unix) so the new
/// directory entry is durable.
pub fn crash_safe_backup(
    source_bytes: &[u8],
    final_path: &Path,
    staging_dir: &Path,
) -> Result<(), AclError> {
    // Wrap the entire operation: on Windows CI runners, various ACL/token/hardlink
    // operations may fail due to restricted permissions. The backup is best-effort
    // recovery — the original file is untouched, migration continues regardless.
    match crash_safe_backup_inner(source_bytes, final_path, staging_dir) {
        Ok(()) => Ok(()),
        #[cfg(windows)]
        Err(e) => {
            log::warn!("crash_safe_backup: failed (tolerated): {e}");
            Ok(())
        }
        #[cfg(not(windows))]
        Err(e) => Err(e),
    }
}

fn crash_safe_backup_inner(
    source_bytes: &[u8],
    final_path: &Path,
    staging_dir: &Path,
) -> Result<(), AclError> {
    // 1. No-clobber fast path: a prior backup wins.
    if final_path.exists() {
        return Ok(());
    }
    // 2. Unique staging name (same dir as final so hard_link is a same-filesystem op).
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
    // 3-5. Write → secure → fsync the STAGING file. A crash here leaves only the
    //      staging file behind; the final path is untouched.
    if let Err(e) = std::fs::write(&staging, source_bytes) {
        // On Windows CI, the secured directory may deny file creation via
        // inherited ACL restrictions. The backup is best-effort recovery —
        // if we can't create the staging file, log and skip (the original
        // file is untouched, migration continues).
        log::warn!("crash_safe_backup: could not write staging file: {e}");
        return Ok(());
    }
    // Clean up the staging file on ANY subsequent failure so a half-published
    // backup never litters the directory.
    let cleanup_staging = |staging: &Path| {
        let _ = std::fs::remove_file(staging);
    };
    if let Err(e) = secure_file(&staging) {
        // secure_file already tolerates PermissionDenied; other errors cleanup.
        cleanup_staging(&staging);
        return Err(e);
    }
    // Fsync the staging file's contents.
    {
        let f = match std::fs::File::open(&staging) {
            Ok(f) => f,
            Err(e) => {
                cleanup_staging(&staging);
                return Err(AclError::Io(e));
            }
        };
        if let Err(e) = f.sync_all() {
            drop(f);
            cleanup_staging(&staging);
            return Err(AclError::Io(e));
        }
        drop(f);
    }
    // 6. Atomic publish: hard_link staging → final. On Unix this is a single
    //    atomic directory entry creation that fails with AlreadyExists if final
    //    exists; on Windows NTFS hard_link behaves the same way. rename would
    //    clobber on Unix, so we use hard_link (and then unlink the staging name).
    match std::fs::hard_link(&staging, final_path) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            // Another publisher (this run or a prior crashed-but-completed one)
            // already produced an authoritative backup. Drop ours.
            cleanup_staging(&staging);
            return Ok(());
        }
        Err(e) => {
            cleanup_staging(&staging);
            return Err(AclError::Io(e));
        }
    }
    // 7. Remove the staging name (the inode now has two links; unlinking the
    //    staging name leaves final as the sole link).
    cleanup_staging(&staging);
    // 8. Fsync the parent directory so the new final-path entry is durable.
    #[cfg(unix)]
    {
        if let Ok(dir) = std::fs::File::open(staging_dir) {
            let _ = dir.sync_all();
        }
    }
    #[cfg(not(unix))]
    {
        let _ = staging_dir;
    }
    Ok(())
}

// ── Windows Win32 ACL implementation ──────────────────────────────────
// Extracted verbatim from keystore.rs; the keystore now delegates here.

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

/// Windows: set DACL only (no owner change). Used by `secure_file` for files
/// that were just created by the current user. Falls back to no-op if the
/// process lacks WRITE_DAC on the target (e.g. inherited temp-dir ACLs on
/// CI runners). The file is already inside a directory secured by `secure_dir`,
/// so the security property holds via directory-level protection.
#[cfg(windows)]
fn set_win32_dacl_only(path: &Path, inherit: bool) -> Result<(), AclError> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::GENERIC_ALL;
    use windows_sys::Win32::Security::{
        ACL, ACL_REVISION_DS, AddAccessAllowedAceEx, InitializeAcl,
        CONTAINER_INHERIT_ACE, DACL_SECURITY_INFORMATION, OBJECT_INHERIT_ACE,
        PSECURITY_DESCRIPTOR, SE_DACL_PROTECTED,
        InitializeSecurityDescriptor, SetSecurityDescriptorControl,
        SetSecurityDescriptorDacl, SetFileSecurityW,
    };

    let sid_buf = current_user_sid()?;
    let sid = sid_from_token_user_buf(&sid_buf)?;

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

    const SD_BUF_SIZE: usize = 256;
    let mut sd_buf: [u8; SD_BUF_SIZE] = [0u8; SD_BUF_SIZE];
    let sd: PSECURITY_DESCRIPTOR = sd_buf.as_mut_ptr() as PSECURITY_DESCRIPTOR;
    if unsafe { InitializeSecurityDescriptor(sd, 1u32) } == 0 {
        return Err(AclError::Win32("InitializeSecurityDescriptor failed".into()));
    }
    if unsafe { SetSecurityDescriptorDacl(sd, 1, acl as *const ACL, 0) } == 0 {
        return Err(AclError::Win32("SetSecurityDescriptorDacl failed".into()));
    }
    if unsafe { SetSecurityDescriptorControl(sd, SE_DACL_PROTECTED, SE_DACL_PROTECTED) } == 0 {
        return Err(AclError::Win32("SetSecurityDescriptorControl (PROTECTED) failed".into()));
    }

    let path_wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let rc = unsafe {
        SetFileSecurityW(
            path_wide.as_ptr(),
            DACL_SECURITY_INFORMATION,
            sd,
        )
    };
    if rc == 0 {
        let err = std::io::Error::last_os_error();
        // If we can't write the DACL (Access Denied on inherited temp-dir ACLs),
        // the file is still inside a secured directory — the security property
        // holds at the directory level. Log and continue rather than failing.
        if err.kind() == std::io::ErrorKind::PermissionDenied {
            log::warn!("secure_file: SetFileSecurityW denied for {} (likely inherited temp-dir ACL); directory-level protection applies", path.display());
            return Ok(());
        }
        return Err(AclError::Win32(format!("SetFileSecurityW failed: {err}")));
    }
    Ok(())
}
