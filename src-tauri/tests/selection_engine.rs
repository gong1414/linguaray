use islandpot_lib::selection_engine::{capture, Capture, ClipboardLike};
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
    image: RefCell<Option<Vec<u8>>>,
    seq: RefCell<u64>,
    /// copy() closure-controlled behavior is via the capture() arg; this fake just
    /// tracks state. Set copy_fail / get_text_fail to inject errors.
    copy_fail: bool,
    get_text_fail_on_second: bool,
    get_text_calls: RefCell<u32>,
}
impl FakeImg {
    fn bump(&self) { *self.seq.borrow_mut() += 1; }
}
impl ClipboardLike for FakeImg {
    fn get_text(&self) -> Result<String, String> {
        let mut calls = self.get_text_calls.borrow_mut();
        *calls += 1;
        // First call = the Saved::capture read; second = the post-copy selection read.
        if *calls == 2 && self.get_text_fail_on_second {
            return Err("injected get_text failure".into());
        }
        Ok(self.text.borrow().clone())
    }
    fn set_text(&self, s: &str) -> Result<(), String> {
        *self.text.borrow_mut() = s.to_string();
        self.bump();
        Ok(())
    }
    fn get_image(&self) -> Result<Option<Vec<u8>>, String> {
        Ok(self.image.borrow().clone())
    }
    fn set_image(&self, img: &[u8]) -> Result<(), String> {
        *self.image.borrow_mut() = Some(img.to_vec());
        // Setting an image replaces text on this fake (mirrors a real clipboard where
        // an image write clears the text flavor).
        *self.text.borrow_mut() = String::new();
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
        get_text_calls: RefCell::new(0),
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
        get_text_calls: RefCell::new(0),
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
        text: RefCell::new(String::new()), image: RefCell::new(Some(vec![1, 2, 3])),
        seq: RefCell::new(5), copy_fail: false, get_text_fail_on_second: false,
        get_text_calls: RefCell::new(0),
    };
    let res = capture(&f, || { f.set_text("selected").map(|_| ()) }, 50).unwrap();
    assert!(matches!(res, Capture::Selected(t) if t == "selected"));
    assert_eq!(*f.image.borrow(), Some(vec![1, 2, 3]), "image restored");
}
