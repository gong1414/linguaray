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
pub fn secure_dir(dir: &Path) -> Result<(), AclError> {
    #[cfg(unix)]
    { set_mode(dir, 0o700) }
    #[cfg(windows)]
    { set_win32_owner_dacl(dir, true) }
    #[cfg(not(any(unix, windows)))]
    { let _ = dir; Ok(()) }
}

/// Secure a file: 0o600 on Unix; protected DACL (non-inheritable) on Windows.
pub fn secure_file(path: &Path) -> Result<(), AclError> {
    #[cfg(unix)]
    { set_mode(path, 0o600) }
    #[cfg(windows)]
    { set_win32_owner_dacl(path, false) }
    #[cfg(not(any(unix, windows)))]
    { let _ = path; Ok(()) }
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
        OWNER_SECURITY_INFORMATION, PROTECTED_DACL_SECURITY_INFORMATION, PSID,
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
