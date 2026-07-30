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
