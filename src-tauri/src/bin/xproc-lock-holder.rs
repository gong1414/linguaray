//! Test helper: flock dir/keystore.lock DIRECTLY (no update_keys, no Argon2),
//! write a `holding` flag, sleep N seconds, release. Round-4 review: the old
//! version called update_keys which runs Argon2 inside the lock — polluting the
//! timing assertion. This version is a pure flock hold so the parent's load()
//! (on an empty dir, no KDF) can be timed cleanly.
//!
//! Usage: xproc-lock-holder <dir> <hold_seconds>
use fs2::FileExt;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = std::path::PathBuf::from(args.get(1).expect("usage: <dir> <hold_seconds>"));
    let hold_secs: u64 = args.get(2).expect("usage: <dir> <hold_seconds>").parse().expect("secs");

    std::fs::create_dir_all(&dir).ok();
    let lock_path = dir.join("keystore.lock");
    let f = std::fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(&lock_path)
        .expect("open lock file");
    f.lock_exclusive().expect("lock_exclusive");

    // Signal we're holding the lock.
    let flag = dir.join("holding");
    std::fs::write(&flag, b"1").ok();

    std::thread::sleep(std::time::Duration::from_secs(hold_secs));

    let _ = std::fs::remove_file(&flag);
    let _ = f.unlock();
}
