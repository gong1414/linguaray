//! Latest-wins generation token + selection mutex (spec §concurrency).
use std::sync::atomic::{AtomicU64, Ordering};
use parking_lot::Mutex;

pub struct GenerationToken {
    current: AtomicU64,
    selection: Mutex<()>,
}

impl GenerationToken {
    pub fn new() -> Self {
        Self { current: AtomicU64::new(0), selection: Mutex::new(()) }
    }
    /// Allocate the next generation; it becomes "current". Returns the new gen.
    pub fn next(&self) -> u64 {
        self.current.fetch_add(1, Ordering::SeqCst) + 1
    }
    /// True iff `gen` is still the latest (no newer `next()` has run).
    pub fn is_latest(&self, gen: u64) -> bool {
        self.current.load(Ordering::SeqCst) == gen
    }
    pub fn selection_lock(&self) -> parking_lot::MutexGuard<'_, ()> {
        self.selection.lock()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn newest_is_latest() {
        let t = GenerationToken::new();
        let a = t.next();
        let b = t.next();
        assert!(!t.is_latest(a));
        assert!(t.is_latest(b));
    }
}
