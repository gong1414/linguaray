//! The §B sentinel clipboard state machine, decoupled from the OS clipboard via a
//! trait so it is unit-testable with a fake. Real wiring (enigo Cmd+C, real
//! clipboard.rs) lives in selection.rs (Task 5).

/// What the engine needs from a clipboard. The real impl wraps `clipboard.rs`;
/// tests use a fake.
pub trait ClipboardLike {
    fn get_text(&self) -> std::result::Result<String, String>;
    fn set_text(&self, s: &str) -> std::result::Result<(), String>;
    /// Save + restore the image on the clipboard (best-effort; spec §B image
    /// promise). None if there's no image. Errors are non-fatal — see comments.
    fn get_image(&self) -> std::result::Result<Option<Vec<u8>>, String> {
        Ok(None)
    }
    fn set_image(&self, _img: &[u8]) -> std::result::Result<(), String> {
        Ok(())
    }
    /// Monotonic sequence number that advances on ANY write (ours included).
    fn sequence(&self) -> u64;
}

#[derive(Debug)]
pub enum Capture {
    Selected(String),
    NoSelection,
}

/// Saved clipboard snapshot for restoration. `unwrap_or_default()` on text would
/// lose an image-only clipboard, so we capture BOTH (text may be empty while an
/// image is present). restore_if_owned replays whichever was present.
struct Saved {
    text: Option<String>, // None if there was no text
    image: Option<Vec<u8>>,
    owned_sequence: u64, // sequence right after we wrote the sentinel
}

impl Saved {
    fn capture<C: ClipboardLike>(clip: &C) -> std::result::Result<Self, String> {
        // Text: None if absent/error (distinguish from empty-string text presence).
        let text = match clip.get_text() {
            Ok(t) => Some(t),
            Err(_) => None,
        };
        // Image: best-effort; errors → None.
        let image = clip.get_image().unwrap_or(None);
        Ok(Saved { text, image, owned_sequence: 0 })
    }

    /// Restore the snapshot ONLY if the clipboard sequence still equals owned. Per
    /// §B: if a newer writer landed since, do NOT clobber. Restore text and/or image
    /// depending on what was present; both being None means we just clear our
    /// sentinel. Errors here are best-effort (ignored) — we never want restore
    /// failure to mask the real selection result.
    fn restore_if_owned<C: ClipboardLike>(&self, clip: &C) {
        if clip.sequence() != self.owned_sequence {
            return; // newer writer — don't clobber
        }
        if let Some(img) = &self.image {
            let _ = clip.set_image(img);
        } else if let Some(t) = &self.text {
            let _ = clip.set_text(t);
        }
    }
}

/// Run the §B algorithm. `copy` is the simulated-copy step (Cmd+C/Ctrl+C),
/// injected so the engine stays pure/testable. Returns the selected text or
/// NoSelection (sentinel still present after `copy` ran => nothing was selected).
///
/// Every exit path restores the saved clipboard if still owned — copy() failure,
/// get_text() failure, NoSelection, success — via a single cleanup discipline.
pub fn capture<C: ClipboardLike, F: FnMut() -> std::result::Result<(), String>>(
    clip: &C,
    mut copy: F,
    timeout_iters: usize,
) -> std::result::Result<Capture, String> {
    // 1. Save current content (text + image, each optional).
    let mut saved = Saved::capture(clip)?;
    // 2. Write a unique sentinel.
    let sentinel = format!("__islandpot_sel_{}__", clip.sequence());
    clip.set_text(&sentinel)?;
    saved.owned_sequence = clip.sequence(); // the state WE left the clipboard in
    // 3. Simulate copy. On failure: restore (if still owned), propagate.
    if let Err(e) = copy() {
        saved.restore_if_owned(clip);
        return Err(e);
    }
    // 4. Bounded-wait for the sequence to leave the marker (a successful copy
    //    overwrites the sentinel, advancing the sequence).
    let mut waited = 0usize;
    let mut now = clip.sequence();
    while now == saved.owned_sequence && waited < timeout_iters {
        std::thread::sleep(std::time::Duration::from_millis(20));
        now = clip.sequence();
        waited += 1;
    }
    if now == saved.owned_sequence {
        // Copy didn't happen / nothing selected. Restore saved, return NoSelection.
        saved.restore_if_owned(clip);
        return Ok(Capture::NoSelection);
    }
    // Copy succeeded: re-baseline owned_sequence to the post-copy state. Anything
    // that advances the sequence AFTER this point is a newer writer (don't clobber).
    saved.owned_sequence = now;
    // 5. Copy succeeded: read selection. On get_text failure, restore + propagate.
    let text = match clip.get_text() {
        Ok(t) => t,
        Err(e) => {
            saved.restore_if_owned(clip);
            return Err(e);
        }
    };
    // 6. Restore the saved snapshot if nothing else wrote since (don't clobber a
    //    newer writer). Then return the selection.
    saved.restore_if_owned(clip);
    Ok(Capture::Selected(text))
}
