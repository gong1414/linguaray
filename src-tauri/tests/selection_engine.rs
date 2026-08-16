use linguaray_lib::selection_engine::{capture, Capture, ClipboardLike};
use std::cell::RefCell;

struct Fake {
    text: RefCell<String>,
    seq: RefCell<u64>,
    /// What the simulated copy writes (None = copy does nothing => NoSelection).
    #[allow(dead_code)]
    selection: Option<String>,
    /// If true, get_text() bumps the sequence (models a concurrent writer landing
    /// as we read the selection — the race the §B restore guard protects against).
    get_bumps_seq: bool,
}

impl Fake {
    fn bump(&self) {
        *self.seq.borrow_mut() += 1;
    }
}

impl ClipboardLike for Fake {
    fn get_text(&self) -> Result<String, String> {
        let t = self.text.borrow().clone();
        if self.get_bumps_seq {
            self.bump();
        }
        Ok(t)
    }
    fn set_text(&self, s: &str) -> Result<(), String> {
        *self.text.borrow_mut() = s.to_string();
        self.bump();
        Ok(())
    }
    fn sequence(&self) -> u64 {
        *self.seq.borrow()
    }
}

#[test]
fn success_returns_selection_and_restores_saved() {
    let f = Fake {
        text: RefCell::new("original".into()),
        seq: RefCell::new(5),
        selection: Some("hello".into()),
        get_bumps_seq: false,
    };
    let sel = "hello".to_string();
    let res = capture(&f, || f.set_text(&sel).map(|_| ()), 50).unwrap();
    assert!(matches!(res, Capture::Selected(t) if t == "hello"));
    assert_eq!(f.text.borrow().as_str(), "original", "saved restored");
}

#[test]
fn no_selection_when_copy_does_nothing() {
    let f = Fake {
        text: RefCell::new("original".into()),
        seq: RefCell::new(5),
        selection: None,
        get_bumps_seq: false,
    };
    let res = capture(&f, || Ok(()), 1).unwrap();
    assert!(matches!(res, Capture::NoSelection));
    assert_eq!(
        f.text.borrow().as_str(),
        "original",
        "saved restored on NoSelection"
    );
}

#[test]
fn concurrent_writer_prevents_restore() {
    // get_text() bumps seq: when the engine reads the selection, seq advances,
    // so the final sequence() != owned_sequence => engine does NOT restore.
    let f = Fake {
        text: RefCell::new("original".into()),
        seq: RefCell::new(5),
        selection: Some("hello".into()),
        get_bumps_seq: true,
    };
    let sel = "hello".to_string();
    let res = capture(&f, || f.set_text(&sel).map(|_| ()), 50).unwrap();
    assert!(matches!(res, Capture::Selected(t) if t == "hello"));
    // NOT restored (because get_text bumped seq after owned_sequence was captured)
    assert_eq!(
        f.text.borrow().as_str(),
        "hello",
        "concurrent writer protected — no restore"
    );
}

/// A fake that can inject failures and has a real image slot.
struct FakeImg {
    text: RefCell<String>,
    image: RefCell<Option<(usize, usize, Vec<u8>)>>, // (w, h, bytes)
    seq: RefCell<u64>,
    /// copy() closure-controlled behavior is via the capture() arg; this fake just
    /// tracks state. Set copy_fail / get_text_fail to inject errors.
    copy_fail: bool,
    get_text_fail_on_second: bool,
    /// Simulates an EMPTY original clipboard: real arboard's get_text() returns Err
    /// when there's no text, so Saved::capture's `.ok()` yields None. Used to test the
    /// §B invariant that an empty original (None,None) snapshot still clears the
    /// sentinel on restore (round-11 review P1 #1).
    get_text_always_fails: bool,
    /// Inject a restore_snapshot failure (round-14 review P1 #1: restore errors must
    /// NOT be silenced). When true, restore_snapshot returns Err without changing state.
    restore_fail: bool,
    get_text_calls: RefCell<u32>,
}
impl FakeImg {
    fn bump(&self) { *self.seq.borrow_mut() += 1; }
}
impl ClipboardLike for FakeImg {
    fn get_text(&self) -> Result<String, String> {
        let mut calls = self.get_text_calls.borrow_mut();
        *calls += 1;
        // Empty-clipboard model: get_text always errors (mirrors real arboard on no text).
        if self.get_text_always_fails {
            return Err("no text on clipboard".into());
        }
        // First call = the Saved::capture read; second = the post-copy selection read.
        if *calls == 2 && self.get_text_fail_on_second {
            return Err("injected get_text failure".into());
        }
        Ok(self.text.borrow().clone())
    }
    fn set_text(&self, s: &str) -> Result<(), String> {
        *self.text.borrow_mut() = s.to_string();
        // Mirrors real arboard: each setter clears the other flavor (round-2 P1 #2).
        *self.image.borrow_mut() = None;
        self.bump();
        Ok(())
    }
    fn get_image(&self) -> Result<Option<linguaray_lib::selection_engine::ImageBlob>, String> {
        Ok(self.image.borrow().as_ref().map(|b| linguaray_lib::selection_engine::ImageBlob {
            width: b.0, height: b.1, bytes: b.2.clone(),
        }))
    }
    fn set_image(&self, img: &linguaray_lib::selection_engine::ImageBlob) -> Result<(), String> {
        *self.image.borrow_mut() = Some((img.width, img.height, img.bytes.clone()));
        // Mirrors real arboard: each setter clears the other flavor.
        *self.text.borrow_mut() = String::new();
        self.bump();
        Ok(())
    }
    // Restore both formats in ONE write (single clear + write-both), matching the
    // production restore_snapshot — so the text+image test isn't a false green.
    fn restore_snapshot(
        &self,
        text: Option<&str>,
        image: Option<&linguaray_lib::selection_engine::ImageBlob>,
    ) -> Result<(), String> {
        // Inject a restore failure (round-14 review P1 #1): simulate a platform restore
        // error (e.g. Windows SetPartial / writeObjects fail) WITHOUT changing state, so
        // the caller can prove the error propagates instead of being silenced.
        if self.restore_fail {
            return Err("injected restore failure".into());
        }
        // Single clear, then set BOTH (no per-set clearing here).
        *self.text.borrow_mut() = String::new();
        *self.image.borrow_mut() = None;
        if let Some(img) = image {
            *self.image.borrow_mut() = Some((img.width, img.height, img.bytes.clone()));
        }
        if let Some(t) = text {
            *self.text.borrow_mut() = t.to_string();
        }
        self.bump();
        Ok(())
    }
    fn sequence(&self) -> u64 { *self.seq.borrow() }
}

#[test]
fn copy_failure_restores_saved() {
    // copy() returns Err → saved text must be restored, error propagated.
    let f = FakeImg {
        text: RefCell::new("original".into()), image: RefCell::new(None),
        seq: RefCell::new(5), copy_fail: true, get_text_fail_on_second: false,
        get_text_always_fails: false, restore_fail: false, get_text_calls: RefCell::new(0),
    };
    let err = capture(&f, || if f.copy_fail { Err("copy failed".into()) } else { Ok(()) }, 50).unwrap_err();
    assert_eq!(err, "copy failed");
    // sentinel was written then restored (sequence guard: we just wrote sentinel so
    // owned==current, restore fires). text back to "original".
    assert_eq!(f.text.borrow().as_str(), "original", "saved restored after copy failure");
}

#[test]
fn get_text_failure_restores_saved() {
    // copy succeeds, but the post-copy selection read fails → restore + propagate.
    let f = FakeImg {
        text: RefCell::new("original".into()), image: RefCell::new(None),
        seq: RefCell::new(5), copy_fail: false, get_text_fail_on_second: true,
        get_text_always_fails: false, restore_fail: false, get_text_calls: RefCell::new(0),
    };
    let res = capture(&f, || { f.set_text("hello").map(|_| ()) }, 50);
    assert!(res.is_err(), "get_text failure propagates");
    assert_eq!(f.text.borrow().as_str(), "original", "saved restored after read failure");
}

#[test]
fn image_only_clipboard_restored() {
    // Clipboard had ONLY an image (no text). sentinel write clears it; on success
    // the image must be restored (unwrap_or_default on text would have lost it).
    let f = FakeImg {
        text: RefCell::new(String::new()), image: RefCell::new(Some((1, 1, vec![1, 2, 3, 4]))),
        seq: RefCell::new(5), copy_fail: false, get_text_fail_on_second: false,
        get_text_always_fails: false, restore_fail: false, get_text_calls: RefCell::new(0),
    };
    let res = capture(&f, || { f.set_text("selected").map(|_| ()) }, 50).unwrap();
    assert!(matches!(res, Capture::Selected(t) if t == "selected"));
    assert_eq!(*f.image.borrow(), Some((1, 1, vec![1, 2, 3, 4])), "image restored");
}

#[test]
fn empty_original_clipboard_noselection_clears_sentinel() {
    // §B regression (round-11 review P1 #1): when the original clipboard was EMPTY
    // (no text → get_text errors → Saved.text = None; no image → None), the snapshot
    // is (None, None). The sentinel written during capture MUST be removed on restore,
    // returning the clipboard to empty. The prev production restore_snapshot(None,None)
    // returned Ok(()) without clearing, leaving the sentinel behind.
    //
    // This test models the empty original via get_text_always_fails (real arboard
    // errors on an empty clipboard → Saved.text = None) + image = None. The copy() here
    // does NOT advance the sequence (nothing selected) → NoSelection. restore_if_owned
    // fires (we still own the post-sentinel sequence) and must clear the sentinel.
    let f = FakeImg {
        text: RefCell::new(String::new()), image: RefCell::new(None),
        seq: RefCell::new(5), copy_fail: false, get_text_fail_on_second: false,
        get_text_always_fails: true, restore_fail: false, get_text_calls: RefCell::new(0),
    };
    let res = capture(&f, || { Ok(()) }, 50).unwrap(); // copy does nothing → NoSelection
    assert!(matches!(res, Capture::NoSelection), "nothing selected");
    // The Fake's restore_snapshot(None,None) clears BOTH fields. If restore had been a
    // no-op (the bug), the sentinel "__linguaray_sel_*__" would still be in `text`.
    assert!(
        !f.text.borrow().contains("__linguaray_sel_"),
        "sentinel must be cleared on empty-original restore (got {:?})",
        f.text.borrow()
    );
    assert!(
        f.image.borrow().is_none(),
        "image must be None after clearing the empty snapshot"
    );
}

#[test]
fn ax_first_short_circuits_copy_fallback() {
    // When the injected AX reader returns Some(text), capture_selection_with_ax
    // must return it directly and NOT touch the clipboard (no copy path).
    use linguaray_lib::selection::capture_selection_with_ax;
    use linguaray_lib::selection_engine::Capture;
    // The owner value is cfg'd (Phase 4 Task 2b M3): capture_selection_with_ax takes a
    // third `owner: OwnerHwnd` arg on ALL targets (raw HWND on Windows, () elsewhere).
    // The AX-Some path short-circuits before the owner is used, so null_mut is safe on
    // Windows — restore_snapshot is never reached.
    #[cfg(target_os = "windows")]
    let owner: linguaray_lib::selection::OwnerHwnd = std::ptr::null_mut();
    #[cfg(not(target_os = "windows"))]
    let owner: linguaray_lib::selection::OwnerHwnd = ();
    let res = capture_selection_with_ax(|| Some("ax-text".into()), 1, owner).unwrap();
    assert!(matches!(res, Capture::Selected(t) if t == "ax-text"));
}

// (The "AX-None routes to copy-fallback" behavior is a trivial 2-line branch in
// capture_selection_with_ax; it's not unit-testable without a real clipboard
// because capture_selection_with_ax's copy path uses the real OsClipboard. The
// copy-fallback ITSELF is covered by the Fake-based capture() tests above; the
// AX-Some short-circuit is covered by the test above. Runtime E2E covers the
// AX-None → copy path on a real machine.)

#[test]
fn text_and_image_both_restored_when_present() {
    // Review P1 #1: the old `else if` restored only the image when both text+image
    // were present. Now BOTH should be restored. With this Fake, set_image clears
    // text then set_text sets text — so after restore, text is present and the
    // image is also present (FakeImg tracks both independently).
    let f = FakeImg {
        text: RefCell::new("hello".into()),
        image: RefCell::new(Some((2, 2, vec![0; 16]))), // 2x2 RGBA
        seq: RefCell::new(5), copy_fail: false, get_text_fail_on_second: false,
        get_text_always_fails: false, restore_fail: false, get_text_calls: RefCell::new(0),
    };
    let res = capture(&f, || { f.set_text("selected").map(|_| ()) }, 50).unwrap();
    assert!(matches!(res, Capture::Selected(t) if t == "selected"));
    // Per the Fake's set semantics, restore writes image then text — both fields set.
    assert_eq!(f.text.borrow().as_str(), "hello", "text restored");
    assert_eq!(*f.image.borrow(), Some((2, 2, vec![0; 16])), "image restored");
}

#[test]
fn restore_failure_on_success_path_is_not_silenced() {
    // Round-14 review P1 #1: a restore_snapshot failure during the success path must
    // NOT be silenced. The capture SUCCEEDED (text was read), but restoring the user's
    // prior clipboard failed → their prior content is lost → capture returns Err
    // (carrying the restore error), NOT Ok(Selected(...)).
    let f = FakeImg {
        text: RefCell::new("original".into()), image: RefCell::new(None),
        seq: RefCell::new(5), copy_fail: false, get_text_fail_on_second: false,
        get_text_always_fails: false, restore_fail: true, get_text_calls: RefCell::new(0),
    };
    let res = capture(&f, || { f.set_text("selected").map(|_| ()) }, 50);
    let err = res.expect_err("restore failure must propagate, not be silenced");
    assert!(
        err.contains("injected restore failure"),
        "err must surface the restore cause; got: {err}"
    );
    assert!(
        !err.contains("additionally"),
        "no original error to combine on the success path; got: {err}"
    );
}

#[test]
fn restore_failure_combines_with_original_error() {
    // When the ORIGINAL operation also failed (here: get_text fails on the 2nd call),
    // the restore error is COMBINED with it — both surface, neither hides the other.
    let f = FakeImg {
        text: RefCell::new("original".into()), image: RefCell::new(None),
        seq: RefCell::new(5), copy_fail: false, get_text_fail_on_second: true,
        get_text_always_fails: false, restore_fail: true, get_text_calls: RefCell::new(0),
    };
    let res = capture(&f, || { f.set_text("hello").map(|_| ()) }, 50);
    let err = res.expect_err("combined error must propagate");
    assert!(
        err.contains("injected get_text failure"),
        "must surface the ORIGINAL cause; got: {err}"
    );
    assert!(
        err.contains("injected restore failure"),
        "must ALSO surface the restore cause; got: {err}"
    );
    assert!(
        err.contains("additionally"),
        "must use the combine form when both errors present; got: {err}"
    );
}

#[test]
fn restore_failure_on_noselection_path_is_not_silenced() {
    // The NoSelection path (copy did nothing) also must propagate a restore failure
    // rather than returning Ok(NoSelection) silently.
    let f = FakeImg {
        text: RefCell::new("original".into()), image: RefCell::new(None),
        seq: RefCell::new(5), copy_fail: false, get_text_fail_on_second: false,
        get_text_always_fails: false, restore_fail: true, get_text_calls: RefCell::new(0),
    };
    // copy() does nothing → sequence never advances → NoSelection path → restore fires.
    let res = capture(&f, || Ok(()), 50);
    let err = res.expect_err("NoSelection restore failure must propagate");
    assert!(
        err.contains("injected restore failure"),
        "err must surface the restore cause; got: {err}"
    );
}
