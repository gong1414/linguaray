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
    /// An existing backup at the final path failed the caller-supplied
    /// validator (fail-closed: we never trust an existing file blindly). The
    /// string is the validator's reason.
    InvalidExisting(String),
    /// Removing a stale staging file left over from a prior crashed attempt
    /// failed. Surfaces so a stale cleanup regression doesn't silently mask
    /// a `create_new` AlreadyExists.
    CleanStaging(std::io::Error),
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
            BackupError::InvalidExisting(s) => {
                write!(f, "existing backup failed validation: {s}")
            }
            BackupError::CleanStaging(e) => write!(f, "clean stale staging: {e}"),
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
/// 0. No-clobber fast path. If `final_path` already exists:
///    - When `validator` is `Some`, the existing bytes are read and passed to
///      it. A valid existing backup wins (return `Ok(())`); an INVALID one
///      fails closed (`Err(InvalidExisting)`). We never blindly trust an
///      existing file — empty / truncated / corrupt backups must not pass.
///    - When `validator` is `None`, the existing file is accepted as-is
///      (backwards-compatible for callers that have no structural check).
///    - In both cases the parent directory is synced on Unix so the existing
///      entry is durable even if the directory was just created.
/// 1. Stale staging cleanup. A prior crashed attempt may have left a
///    `.{stem}-staging-*` file in `staging_dir`; before `create_new` we scan
///    for files matching the precise prefix `.{final_stem}-staging-` and remove
///    them, so the next attempt doesn't trip on `AlreadyExists`.
/// 2. `create_new` (O_EXCL) staging file — atomic ownership of a unique name.
///    On `AlreadyExists` (true concurrency / a cleanup race) a new name with an
///    incremented counter suffix is tried, up to 3 times total.
/// 3. `secure_file` on the staging path — permissions are correct before data.
/// 4. `write_all` + `flush` via the SAME writable handle that created the file.
/// 5. `sync_all` via the SAME handle — on Windows this calls
///    `FlushFileBuffers` which requires a writable handle. Using `File::open`
///    (read-only) would produce Access Denied on Windows.
/// 6. Publish: Unix `hard_link` (no-clobber) or Windows `MoveFileExW`
///    (MOVEFILE_WRITE_THROUGH, no replace). If final already exists,
///    the staging file is cleaned up and `Ok(())` is returned.
/// 7. Sync parent directory (Unix) so the new entry is durable.
///
/// The final backup path is only ever observable as a complete, secured,
/// synced file. A crash before publish leaves at most a staging file (cleaned
/// up on the next attempt via the stale-staging sweep).
/// Caller-supplied structural check for an existing backup at the final path.
/// Returning `Err(msg)` rejects the existing backup as untrustworthy
/// (`BackupError::InvalidExisting(msg)`); `Ok(())` accepts it (no-clobber,
/// existing wins). Kept as a dedicated alias so the `crash_safe_backup`
/// signature stays readable.
pub type BackupValidator<'a> = &'a dyn Fn(&[u8]) -> Result<(), String>;

pub fn crash_safe_backup(
    source_bytes: &[u8],
    final_path: &Path,
    staging_dir: &Path,
    validator: Option<BackupValidator<'_>>,
) -> Result<(), BackupError> {
    // 0. No-clobber fast path: a prior complete backup wins — but only after
    //    validation (when a validator is supplied). A blind `exists()` check
    //    would accept an empty / truncated / corrupt file (fail-open).
    if final_path.exists() {
        if let Some(v) = validator {
            let bytes = std::fs::read(final_path).map_err(BackupError::WriteStaging)?;
            v(&bytes).map_err(BackupError::InvalidExisting)?;
        }
        // Existing backup is authoritative (validated or validator-None). Still
        // sync the parent dir on Unix: if the directory was just created, the
        // existing entry's dirent may not yet be durable.
        sync_parent_dir(staging_dir)?;
        return Ok(());
    }

    // 1. Stale-staging cleanup: remove `.{stem}-staging-*` left over from a
    //    prior crashed attempt BEFORE create_new, otherwise the stale file
    //    would trip AlreadyExists on this attempt.
    let stem = final_path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "backup".to_string());
    clean_stale_staging(staging_dir, &stem)?;

    // 2..5. Create + secure + write + sync the staging file, retrying on
    //       AlreadyExists (high-concurrency name collision) with a fresh
    //       counter-suffixed name up to 3 attempts.
    let staging = create_stage_and_write(source_bytes, staging_dir, &stem)?;
    let mut guard = StagingGuard::new(staging.clone());

    // 6. Publish: atomically make the final path visible (no-clobber).
    publish_backup(&staging, final_path)?;
    // Staging path is consumed by the publish (moved/hard-linked).
    // On hard_link success we need to remove the staging name;
    // on MoveFileExW success the staging no longer exists.
    guard.disarm(); // publish succeeded; don't double-delete

    // 7. Sync parent directory (Unix) so the new directory entry is durable.
    sync_parent_dir(staging_dir)?;

    Ok(())
}

/// Create the staging file, secure it, write the bytes, and fsync — retrying
/// the `create_new` on `AlreadyExists` with a fresh counter-suffixed name (up
/// to 3 attempts total). Returns the path of the staging file that was written.
///
/// The original nanosecond-timestamp name can collide under high concurrency
/// (two publishers racing the same nanosecond); the counter suffix makes the
/// name unique across retries.
fn create_stage_and_write(
    source_bytes: &[u8],
    staging_dir: &Path,
    stem: &str,
) -> Result<std::path::PathBuf, BackupError> {
    use std::io::Write;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let nanos = now.subsec_nanos();

    let mut last_err: Option<std::io::Error> = None;
    for attempt in 0u32..3 {
        // First attempt uses the bare timestamp; subsequent attempts add a
        // counter suffix so a name collision is resolved deterministically.
        let staging = if attempt == 0 {
            staging_dir.join(format!(".{stem}-staging-{secs}-{nanos}"))
        } else {
            staging_dir.join(format!(".{stem}-staging-{secs}-{nanos}-{attempt}"))
        };

        let mut guard = StagingGuard::new(staging.clone());

        // create_new (O_EXCL) — atomic, unique ownership of the name.
        let mut file = match std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&staging)
        {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                // Collision with a concurrent publisher (or a sweep that
                // raced). Try the next counter suffix.
                last_err = Some(e);
                continue;
            }
            Err(e) => return Err(BackupError::CreateStaging(e)),
        };

        // Secure the staging file BEFORE writing data.
        secure_file(&staging).map_err(BackupError::SecureStaging)?;

        // Write all bytes + flush user-space buffers.
        file.write_all(source_bytes)
            .map_err(BackupError::WriteStaging)?;
        file.flush().map_err(BackupError::WriteStaging)?;

        // Fsync — same writable handle (Windows FlushFileBuffers needs write).
        file.sync_all().map_err(BackupError::SyncStaging)?;
        drop(file); // release the handle before publish

        guard.disarm(); // hand ownership of the path to the caller
        return Ok(staging);
    }

    // Exhausted retries —surface the last AlreadyExists.
    Err(BackupError::CreateStaging(
        last_err.unwrap_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::AlreadyExists, "staging name collision")
        }),
    ))
}

/// Scan `staging_dir` for stale staging files belonging to `stem` and remove
/// them. Only files whose name starts with `.{stem}-staging-` are touched —
/// unrelated files (other backups' staging, other final files) are left alone.
///
/// Best-effort on read errors (a transient read_dir failure shouldn't abort a
/// backup); a removal failure of an actual stale file is surfaced so the
/// subsequent `create_new` AlreadyExists isn't masked.
fn clean_stale_staging(staging_dir: &Path, stem: &str) -> Result<(), BackupError> {
    let prefix = format!(".{stem}-staging-");
    let entries = match std::fs::read_dir(staging_dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Staging dir doesn't exist yet — nothing to clean. create_new
            // will surface the real error if the dir can't be created by the
            // caller's setup.
            return Ok(());
        }
        Err(_) => {
            // A transient read_dir failure is best-effort: don't abort the
            // backup over an unreadable dir (the subsequent create_new is the
            // authoritative check).
            return Ok(());
        }
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(&prefix) {
            // Remove the stale staging file. An error here is surfaced so a
            // stuck stale file doesn't manifest as a confusing create_new
            // AlreadyExists on the next attempt.
            if let Err(e) = std::fs::remove_file(entry.path()) {
                // If it vanished between the readdir and the remove, that's
                // fine (another sweeper or publisher cleaned it). Anything
                // else is a real cleanup failure.
                if e.kind() != std::io::ErrorKind::NotFound {
                    return Err(BackupError::CleanStaging(e));
                }
            }
        }
    }
    Ok(())
}

/// Sync the parent directory on Unix so a freshly-created directory entry is
/// durable. No-op on non-Unix. Used both after a publish and after the
/// existing-valid fast path.
#[allow(unused_variables)]
fn sync_parent_dir(staging_dir: &Path) -> Result<(), BackupError> {
    #[cfg(unix)]
    {
        let dir = std::fs::File::open(staging_dir).map_err(BackupError::SyncParent)?;
        dir.sync_all().map_err(BackupError::SyncParent)?;
    }
    #[cfg(not(unix))]
    {
        let _ = staging_dir;
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

/// Atomic no-clobber archive publish (S2a P0).
///
/// Used by the broken-keystore / broken-DB archive flows
/// (`keystore.json.broken-<secs>-<nanos>`, `linguaray.db.broken-<secs>-<nanos>`).
/// The old shape was `fs::rename(src, archive)`, which OVERWRITES an existing
/// archive on Unix — a rapid second archive within the same nanosecond window,
/// or a colliding suffix, would silently clobber a recoverable prior archive.
/// This helper NEVER overwrites: it creates the archive path with
/// `create_new(true)` (O_EXCL), writes + secures + fsyncs the bytes, and on
/// `AlreadyExists` tries a new name with an incremented `-N` suffix (loop up to
/// 3 times). The source bytes are read from `source` and copied into the new
/// archive; the source file is NOT removed by this helper (the caller decides
/// whether to remove the source — e.g. `archive` removes it, `reset` removes
/// the canonical file).
///
/// Returns the archive path that was actually written (which may carry an `-N`
/// suffix if the bare name collided).
///
/// The archive path's parent directory is fsynced on Unix so the new entry is
/// durable.
pub fn atomic_archive_no_clobber(
    source: &Path,
    archive_path: &Path,
) -> Result<std::path::PathBuf, AclError> {
    use std::io::{Read, Seek, Write};

    // Read the source bytes ONCE (the archive is a copy, not a move — the
    // caller controls the source's lifetime).
    let mut source_file = std::fs::File::open(source)?;
    let parent = archive_path.parent().unwrap_or_else(|| Path::new("."));

    let mut written_path = archive_path.to_path_buf();
    for attempt in 0u32..3 {
        // First attempt uses the caller's path verbatim; subsequent attempts
        // append/extend a counter suffix on the file stem.
        if attempt > 0 {
            written_path = with_counter_suffix(archive_path, attempt);
        }
        let mut guard = StagingGuard::new(written_path.clone());

        // create_new (O_EXCL) — never overwrite an existing archive.
        let mut file = match std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&written_path)
        {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                // Another archive landed on the same name; try the next suffix.
                continue;
            }
            Err(e) => return Err(AclError::Io(e)),
        };

        // Secure the archive BEFORE writing data (owner-only on Unix,
        // protected DACL on Windows — same perms as the canonical file).
        secure_file(&written_path)?;

        // Stream-copy source → archive. Rewind the source for each attempt.
        source_file
            .seek(std::io::SeekFrom::Start(0))
            .map_err(AclError::Io)?;
        let mut buf = [0u8; 8192];
        loop {
            let n = source_file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            file.write_all(&buf[..n])?;
        }
        file.flush()?;
        file.sync_all()?; // durable archive bytes
        drop(file);

        guard.disarm(); // archive path is now authoritative

        // Sync the parent dir (Unix) so the archive dirent is durable.
        #[cfg(unix)]
        {
            if let Ok(dir) = std::fs::File::open(parent) {
                let _ = dir.sync_all();
            }
        }

        return Ok(written_path);
    }

    // Exhausted retries.
    Err(AclError::Io(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        format!(
            "archive name collision after retries: {}",
            archive_path.display()
        ),
    )))
}

/// Append a counter suffix to a path's file name. `foo.broken-1-2` with `n=1`
/// → `foo.broken-1-2-1`. Used by [`atomic_archive_no_clobber`] to disambiguate
/// same-timestamp archive collisions.
fn with_counter_suffix(path: &Path, n: u32) -> std::path::PathBuf {
    let mut name = path
        .file_name()
        .map(|s| s.to_os_string())
        .unwrap_or_default();
    name.push(format!("-{n}"));
    let mut out = path.to_path_buf();
    out.set_file_name(name);
    out
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
