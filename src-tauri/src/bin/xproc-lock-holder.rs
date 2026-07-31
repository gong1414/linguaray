//! Test helper: acquire the keystore cross-process lock on a given dir, write a
//! `holding` flag, sleep for N seconds WHILE holding the lock, then release+exit.
//! Used by the child-process lock test to PROVE cross-process mutual exclusion
//! (round-2 review P1 #6).
//!
//! Usage: xproc-lock-holder <dir> <hold_seconds>
//!
//! The lock is held across the sleep because the sleep runs INSIDE update_keys'
//! closure (with_locks holds the fs2 exclusive flock for the whole body).
use islandpot_lib::keystore::Keystore;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = std::path::PathBuf::from(args.get(1).expect("usage: <dir> <hold_seconds>"));
    let hold_secs: u64 = args.get(2).expect("usage: <dir> <hold_seconds>").parse().expect("secs");
    let ks = Keystore::new(dir.clone()).expect("keystore init");
    ks.update_keys(|_k| {
        // We are now holding BOTH the in-proc mutex AND the fs2 exclusive flock.
        let flag = dir.join("holding");
        let _ = std::fs::write(&flag, b"1");
        std::thread::sleep(std::time::Duration::from_secs(hold_secs));
        let _ = std::fs::remove_file(&flag);
    }).expect("update_keys (holds lock across sleep)");
}
