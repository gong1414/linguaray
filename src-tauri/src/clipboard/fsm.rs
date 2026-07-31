//! Platform-neutral clipboard compound-write ownership state machine (Phase 4 Task 2b).
//!
//! This module is ALWAYS COMPILED (no `cfg(target_os = ...)`) and references NO Win32
//! or Cocoa type. It owns the dangerous handle-ownership transitions of the compound
//! clipboard restore (open → empty → submit each format, with exact free/re-empty
//! discipline on every failure path). The real platform adapter (`Win32ClipOps` in
//! `windows.rs`, `#[cfg(windows)]`) implements `ClipOps`; a `FakeClip` in the in-module
//! `#[cfg(test)]` tests does too, so the failure paths are unit-tested on ALL platforms
//! without a real clipboard — mirroring how `selection_engine::capture` is pure + tested
//! via a cross-platform Fake.
//!
//! Design rules (converged across plan review rounds 7-14):
//! - Format ids are plain `u32` params/entries — this module names neither
//!   `CF_UNICODETEXT` nor `CF_DIBV5` (those live in the cfg(windows) builder).
//! - `set()` transfers Handle ownership to the system on Ok; on Err it RETURNS the
//!   handle so the caller frees it (no use-after-move).
//! - `alloc()` postcondition: on Err, NO app-owned handle is live (the adapter frees
//!   any partial allocation internally). The real adapter's leak-safety is unit-tested
//!   by injecting a low-level memory fake (`GlobalMemOps`, windows.rs) — a ClipOps-level
//!   fake can't reach into the adapter's internals (round-13 review P1).
//! - After `open()` succeeds, `close()` runs exactly once via `OpenClip::drop` on
//!   success AND every error path (single-close, observable by the fake).
//! - The remedial `empty()` on a later-set failure is surfaced as `RestoreError::SetPartial`
//!   if IT fails too (honest "may contain partial data"), never silenced.

pub(super) trait ClipOps {
    type Handle;

    /// Open the clipboard for exclusive compound write. The real adapter stores the
    /// owner HWND internally (no HWND arg here — keeps the trait platform-neutral).
    fn open(&mut self) -> Result<(), String>;
    /// Close the clipboard. Called exactly once via `OpenClip::drop` after a successful
    /// `open()` (success AND every error path).
    fn close(&mut self);
    /// Empty/clear the clipboard (the system frees previously-submitted handles).
    fn empty(&mut self) -> Result<(), String>;
    /// Submit one prepared blob under format id `fmt`. On Ok, Handle ownership transfers
    /// to the system (caller must NOT free it). On Err, ownership is NOT transferred —
    /// the handle is returned in the tuple so the caller frees it.
    fn set(&mut self, fmt: u32, h: Self::Handle) -> Result<(), (Self::Handle, String)>;
    /// Allocate a movable blob of `bytes.len()` and copy `bytes` in. Postcondition: on
    /// Err, no app-owned handle is left live (the adapter frees any partial allocation
    /// internally before returning Err). See module docs.
    fn alloc(&mut self, bytes: &[u8]) -> Result<Self::Handle, String>;
    /// Free an app-owned handle that was NOT successfully submitted to the system.
    fn free(&mut self, h: Self::Handle);
}

/// Guard borrowing the adapter mutably; all post-open ops go THROUGH it so the fake
/// observes `close` and there's no second `&mut C` while it lives. `Drop` calls
/// `ClipOps::close` exactly once.
struct OpenClip<'a, C: ClipOps> {
    ops: &'a mut C,
}
impl<C: ClipOps> OpenClip<'_, C> {
    fn empty(&mut self) -> Result<(), String> {
        self.ops.empty()
    }
    fn set(&mut self, fmt: u32, h: C::Handle) -> Result<(), (C::Handle, String)> {
        self.ops.set(fmt, h)
    }
    fn free(&mut self, h: C::Handle) {
        self.ops.free(h)
    }
}
impl<C: ClipOps> Drop for OpenClip<'_, C> {
    fn drop(&mut self) {
        self.ops.close();
    }
}

/// Restore the prepared format list to the clipboard via `c`. `formats` is 0..=N entries
/// (0 = clear-only, the §B empty-original case; 1 = single flavor; 2 = text+image).
/// The PUBLIC `restore_snapshot` wrappers (macOS / windows.rs / the arboard stub) decide
/// the cardinality; this fn only owns the handle-lifecycle transitions.
///
/// Ownership discipline: allocate ALL payloads up front (any alloc failure frees the
/// held ones); `open()` (failure frees all allocated handles); `empty()` always (clears
/// the §B sentinel even for 0 formats); submit each format in order. On the first `set`
/// failure: free the returned handle + all remaining unsubmitted (already-submitted are
/// system-owned, NOT freed). If `submitted == 0` at failure, nothing was on the clipboard
/// (no remedial empty); if `submitted > 0`, a remedial `empty()` removes the live formats,
/// and if THAT fails, surface `SetPartial` (partial data MAY remain — honest).
pub(super) fn restore_with<C: ClipOps>(
    c: &mut C,
    formats: &[(u32, Vec<u8>)],
) -> Result<(), RestoreError> {
    // 1. Allocate all payloads up front. Slot in Option<Handle> so we can drain one at a
    //    time (clean ownership). On any alloc failure, free held ones (reverse) + return.
    let mut handles: Vec<Option<C::Handle>> = Vec::with_capacity(formats.len());
    for (_, bytes) in formats {
        match c.alloc(bytes) {
            Ok(h) => handles.push(Some(h)),
            Err(e) => {
                for slot in handles.into_iter().rev() {
                    c.free(slot.unwrap());
                }
                return Err(RestoreError::Alloc(e));
            }
        }
    }

    // 2. open. On FAILURE free all allocated handles via `c` directly (no guard exists
    //    yet → single mutable borrow). On success hand `c` to the guard.
    let mut clip = match c.open() {
        Ok(()) => OpenClip { ops: c },
        Err(e) => {
            for slot in handles.into_iter().rev() {
                c.free(slot.unwrap());
            }
            return Err(RestoreError::Open(e));
        }
    };

    // 3. empty (always — 0 formats too: clears the §B sentinel).
    if let Err(e) = clip.empty() {
        for slot in handles.into_iter().rev().flatten() {
            clip.free(slot);
        }
        return Err(RestoreError::Empty(e));
    }

    // 4. Submit each format in order, draining its slot.
    let mut submitted = 0usize;
    for (i, (fmt, _)) in formats.iter().enumerate() {
        let h = handles[i].take().unwrap();
        match clip.set(*fmt, h) {
            Ok(()) => submitted += 1,
            Err((h_back, e)) => {
                // Free the returned (un-taken) handle + all remaining un-drained handles.
                clip.free(h_back);
                for slot in handles[i + 1..].iter_mut().rev() {
                    clip.free(slot.take().unwrap());
                }
                if submitted == 0 {
                    return Err(RestoreError::Set(e)); // nothing was on the clipboard
                }
                // Some formats ARE live → remedial empty. If it fails, surface SetPartial
                // (partial data MAY remain — honest, not silenced).
                return match clip.empty() {
                    Ok(()) => Err(RestoreError::Set(e)),
                    Err(ce) => Err(RestoreError::SetPartial {
                        cause: e,
                        cleanup_err: ce,
                    }),
                };
            }
        }
    }
    Ok(()) // clip drops → close() exactly once on every path (incl. panic)
}

/// Errors from `restore_with`. `Display` (via thiserror) is asserted on in the
/// cross-platform fake tests via `to_string()` — these do NOT exercise the
/// Windows-only public wrapper.
#[derive(Debug, thiserror::Error)]
pub(super) enum RestoreError {
    #[error("clipboard allocation failed: {0}")]
    Alloc(String),
    #[error("clipboard open failed: {0}")]
    Open(String),
    #[error("clipboard empty failed: {0}")]
    Empty(String),
    #[error("clipboard set failed: {0}")]
    Set(String),
    /// A later `set` failed AND the remedial `empty` failed: earlier-submitted formats
    /// MAY still be on the clipboard. The honest message names both failures + the
    /// partial-data possibility (never silenced).
    #[error(
        "clipboard set failed: {cause}; cleanup also failed: {cleanup_err}; \
         clipboard may contain partial data"
    )]
    SetPartial {
        cause: String,
        cleanup_err: String,
    },
}

#[cfg(test)]
mod tests {
    //! Failure-injection unit tests for the ownership FSM. Run on ALL platforms (no
    //! clipboard needed): `FakeClip` impls `ClipOps` with `type Handle = u32`, records
    //! every call, and tracks handle ownership so it panics on double-free and asserts
    //! no-leak at end. These prove the exact free/empty/close counts + order for each
    //! failure branch — the danger zone of the compound write.
    use super::*;
    use std::cell::RefCell;

    /// Which adapter call to force into failing (None = no injected failure).
    #[derive(Default)]
    struct FailSpec {
        open: Option<&'static str>,
        empty: Option<&'static str>,
        /// fail the Nth (0-indexed) set call
        set_index: Option<usize>,
        /// fail the Nth (0-indexed) alloc call
        alloc_index: Option<usize>,
        /// fail the second empty (the remedial one) — used by cleanup_empty_fails
        remedial_empty: Option<&'static str>,
    }

    /// Fake adapter. Records the call log + ownership map. `set` Ok marks the handle
    /// system-owned (freeing it then panics = double-free detector); `empty` marks all
    /// system-transferred handles freed (so a post-failure empty clears residual state).
    struct FakeClip {
        fail: FailSpec,
        log: RefCell<Vec<&'static str>>,
        next_handle: RefCell<u32>,
        set_calls: RefCell<usize>,
        alloc_calls: RefCell<usize>,
        empty_calls: RefCell<usize>,
        /// handle id → "ours" (app-owned) or "system" (transferred via successful set)
        owned: RefCell<std::collections::HashMap<u32, &'static str>>,
        opened: RefCell<bool>,
    }

    impl FakeClip {
        fn new(fail: FailSpec) -> Self {
            FakeClip {
                fail,
                log: RefCell::new(Vec::new()),
                next_handle: RefCell::new(1),
                set_calls: RefCell::new(0),
                alloc_calls: RefCell::new(0),
                empty_calls: RefCell::new(0),
                owned: RefCell::new(std::collections::HashMap::new()),
                opened: RefCell::new(false),
            }
        }
        fn count(&self, tag: &'static str) -> usize {
            self.log.borrow().iter().filter(|t| **t == tag).count()
        }
    }

    impl ClipOps for FakeClip {
        type Handle = u32;

        fn open(&mut self) -> Result<(), String> {
            self.log.borrow_mut().push("open");
            if let Some(e) = self.fail.open {
                return Err(e.into());
            }
            *self.opened.borrow_mut() = true;
            Ok(())
        }
        fn close(&mut self) {
            self.log.borrow_mut().push("close");
            // close must only be called after a successful open (the guard is constructed
            // only then). Asserting here would catch a close-without-open regression.
            assert!(
                *self.opened.borrow(),
                "close called without a successful open"
            );
        }
        fn empty(&mut self) -> Result<(), String> {
            let n = *self.empty_calls.borrow();
            *self.empty_calls.borrow_mut() = n + 1;
            self.log.borrow_mut().push("empty");
            // First empty = the always-clear; second = remedial after a set failure.
            if n == 1 {
                // remedial empty
                if let Some(e) = self.fail.remedial_empty {
                    return Err(e.into());
                }
            } else if let Some(e) = self.fail.empty {
                return Err(e.into());
            }
            // empty marks system-transferred handles freed (residual state cleared).
            self.owned.borrow_mut().retain(|_, v| *v != "system");
            Ok(())
        }
        fn set(&mut self, _fmt: u32, h: u32) -> Result<(), (u32, String)> {
            let n = *self.set_calls.borrow();
            *self.set_calls.borrow_mut() = n + 1;
            self.log.borrow_mut().push("set");
            if self.fail.set_index == Some(n) {
                return Err((h, "set failed".into())); // ownership NOT transferred
            }
            // Success → ownership transfers to system.
            self.owned.borrow_mut().insert(h, "system");
            Ok(())
        }
        fn alloc(&mut self, _bytes: &[u8]) -> Result<u32, String> {
            let n = *self.alloc_calls.borrow();
            *self.alloc_calls.borrow_mut() = n + 1;
            self.log.borrow_mut().push("alloc");
            if self.fail.alloc_index == Some(n) {
                return Err("alloc failed".into());
            }
            let h = *self.next_handle.borrow();
            *self.next_handle.borrow_mut() = h + 1;
            self.owned.borrow_mut().insert(h, "ours");
            Ok(h)
        }
        fn free(&mut self, h: u32) {
            self.log.borrow_mut().push("free");
            let mut owned = self.owned.borrow_mut();
            match owned.remove(&h) {
                Some("ours") => {} // ok: freeing an app-owned handle
                Some("system") => {
                    panic!("double-free / freeing a system-owned handle: {h}");
                }
                None => panic!("free of unknown handle: {h}"),
                Some(other) => panic!("free of handle {h} with unexpected state: {other}"),
            }
        }
    }

    impl Drop for FakeClip {
        fn drop(&mut self) {
            // No leak: every handle is either system-owned (transferred) or freed.
            let leaked: Vec<u32> = self
                .owned
                .borrow()
                .iter()
                .filter(|(_, v)| **v == "ours")
                .map(|(k, _)| *k)
                .collect();
            assert!(leaked.is_empty(), "app-owned handles leaked: {leaked:?}");
        }
    }

    fn fmt(id: u32, b: &[u8]) -> (u32, Vec<u8>) {
        (id, b.to_vec())
    }

    #[test]
    fn zero_formats_clears() {
        // §B empty-original path: 0 entries → open/empty/close, no alloc/set/free.
        let mut f = FakeClip::new(FailSpec::default());
        restore_with(&mut f, &[]).unwrap();
        assert_eq!(f.count("open"), 1);
        assert_eq!(f.count("empty"), 1, "exactly one empty (the clear)");
        assert_eq!(f.count("close"), 1);
        assert_eq!(f.count("alloc"), 0);
        assert_eq!(f.count("set"), 0);
        assert_eq!(f.count("free"), 0);
    }

    #[test]
    fn one_format_success() {
        let mut f = FakeClip::new(FailSpec::default());
        restore_with(&mut f, &[fmt(1, b"hi")]).unwrap();
        assert_eq!(f.count("alloc"), 1);
        assert_eq!(f.count("set"), 1);
        assert_eq!(f.count("close"), 1);
        assert_eq!(f.count("free"), 0, "system-owned, not freed");
        assert_eq!(f.count("empty"), 1, "the initial clear only");
    }

    #[test]
    fn two_formats_success() {
        let mut f = FakeClip::new(FailSpec::default());
        restore_with(&mut f, &[fmt(1, b"a"), fmt(2, b"b")]).unwrap();
        assert_eq!(f.count("alloc"), 2);
        assert_eq!(f.count("set"), 2);
        assert_eq!(f.count("close"), 1);
        assert_eq!(f.count("free"), 0);
        assert_eq!(f.count("empty"), 1);
    }

    #[test]
    fn open_fails_frees_both_no_close() {
        let mut f = FakeClip::new(FailSpec {
            open: Some("open boom"),
            ..Default::default()
        });
        let r = restore_with(&mut f, &[fmt(1, b"a"), fmt(2, b"b")]);
        assert!(matches!(r, Err(RestoreError::Open(_))), "got {r:?}");
        assert_eq!(f.count("free"), 2, "both pre-allocated handles freed");
        assert_eq!(f.count("open"), 1);
        assert_eq!(f.count("empty"), 0);
        assert_eq!(f.count("set"), 0);
        assert_eq!(f.count("close"), 0, "guard never built");
    }

    #[test]
    fn empty_fails_frees_both_one_close() {
        let mut f = FakeClip::new(FailSpec {
            empty: Some("empty boom"),
            ..Default::default()
        });
        let r = restore_with(&mut f, &[fmt(1, b"a"), fmt(2, b"b")]);
        assert!(matches!(r, Err(RestoreError::Empty(_))), "got {r:?}");
        assert_eq!(f.count("free"), 2);
        assert_eq!(f.count("set"), 0);
        assert_eq!(f.count("close"), 1, "guard dropped → close once");
        assert_eq!(f.count("empty"), 1, "the failing empty only");
    }

    #[test]
    fn first_set_fails_frees_both_no_reempty() {
        let mut f = FakeClip::new(FailSpec {
            set_index: Some(0),
            ..Default::default()
        });
        let r = restore_with(&mut f, &[fmt(1, b"a"), fmt(2, b"b")]);
        assert!(matches!(r, Err(RestoreError::Set(_))), "got {r:?}");
        assert_eq!(f.count("free"), 2, "both unsubmitted handles freed");
        assert_eq!(f.count("close"), 1);
        assert_eq!(f.count("empty"), 1, "initial clear only — NO remedial empty");
    }

    #[test]
    fn second_set_fails_one_free_one_reempty() {
        let mut f = FakeClip::new(FailSpec {
            set_index: Some(1), // second set fails; first succeeds → h_text system-owned
            ..Default::default()
        });
        let r = restore_with(&mut f, &[fmt(1, b"a"), fmt(2, b"b")]);
        assert!(matches!(r, Err(RestoreError::Set(_))), "got {r:?}");
        assert_eq!(f.count("free"), 1, "only h_dib (h_text is system-owned)");
        assert_eq!(f.count("empty"), 2, "initial clear + remedial empty");
        assert_eq!(f.count("close"), 1);
    }

    #[test]
    fn first_alloc_fails_no_free_no_open() {
        let mut f = FakeClip::new(FailSpec {
            alloc_index: Some(0),
            ..Default::default()
        });
        let r = restore_with(&mut f, &[fmt(1, b"a"), fmt(2, b"b")]);
        assert!(matches!(r, Err(RestoreError::Alloc(_))), "got {r:?}");
        assert_eq!(f.count("free"), 0, "nothing held when the first alloc fails");
        assert_eq!(f.count("open"), 0);
        assert_eq!(f.count("close"), 0);
    }

    #[test]
    fn second_alloc_fails_one_free_no_open() {
        // The leak the round-10 design couldn't catch: first alloc ok, second fails →
        // the first handle MUST be freed (no open, no close).
        let mut f = FakeClip::new(FailSpec {
            alloc_index: Some(1),
            ..Default::default()
        });
        let r = restore_with(&mut f, &[fmt(1, b"a"), fmt(2, b"b")]);
        assert!(matches!(r, Err(RestoreError::Alloc(_))), "got {r:?}");
        assert_eq!(f.count("free"), 1, "the first allocated handle freed");
        assert_eq!(f.count("open"), 0);
        assert_eq!(f.count("close"), 0);
    }

    #[test]
    fn cleanup_empty_fails_surfaces_setpartial() {
        // second set fails AND the remedial empty fails → SetPartial (honest), NOT silenced.
        let mut f = FakeClip::new(FailSpec {
            set_index: Some(1),
            remedial_empty: Some("remedial boom"),
            ..Default::default()
        });
        let r = restore_with(&mut f, &[fmt(1, b"a"), fmt(2, b"b")]);
        let err = r.expect_err("expected SetPartial");
        // Display (thiserror) works + is honest: names cleanup failure + partial data.
        let s = err.to_string();
        assert!(s.contains("cleanup also failed"), "msg: {s}");
        assert!(s.contains("partial data"), "msg: {s}");
        // And the concrete fields (by value now that we've taken to_string):
        match err {
            RestoreError::SetPartial {
                cause,
                cleanup_err,
            } => {
                assert!(cause.contains("set failed"), "cause: {cause}");
                assert!(cleanup_err.contains("remedial boom"), "cleanup_err: {cleanup_err}");
            }
            other => panic!("expected SetPartial, got {other:?}"),
        }
        assert_eq!(f.count("free"), 1, "h_dib freed; h_text system-owned (may persist)");
        // NOTE: we do NOT assert all handles cleared — h_text (system-owned) may legitimately
        // remain on the clipboard after a failed remedial empty. The Drop leak-check only
        // flags app-owned ("ours") handles, which are all gone here.
    }
}
