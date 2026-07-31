//! The §B sentinel clipboard state machine, decoupled from the OS clipboard via a
//! trait so it is unit-testable with a fake. Real wiring (enigo Cmd+C, real
//! clipboard.rs) lives in selection.rs (Task 5).

/// An RGBA image blob for save/restore (width + height needed to round-trip).
#[derive(Debug, Clone)]
pub struct ImageBlob {
    pub width: usize,
    pub height: usize,
    pub bytes: Vec<u8>, // RGBA, row-major
}

/// What the engine needs from a clipboard. The real impl wraps `clipboard.rs`;
/// tests use a fake.
pub trait ClipboardLike {
    fn get_text(&self) -> std::result::Result<String, String>;
    fn set_text(&self, s: &str) -> std::result::Result<(), String>;
    /// Save + restore the image on the clipboard (best-effort; spec §B image
    /// promise). None if there's no image. Errors are non-fatal.
    fn get_image(&self) -> std::result::Result<Option<ImageBlob>, String> {
        Ok(None)
    }
    fn set_image(&self, _img: &ImageBlob) -> std::result::Result<(), String> {
        Ok(())
    }
    /// Restore BOTH text and image in a single platform-level write. Round-2
    /// review P1 #2: arboard's set_text/set_image each clearContents first, so
    /// calling them in sequence leaves only the last flavor. The real impl clears
    /// ONCE then writes both formats atomically (macOS: set text + image on one
    /// NSPasteboard; Windows: one SetClipboardData sequence). Default impl falls
    /// back to sequential writes (lossy) for fakes that don't override.
    fn restore_snapshot(
        &self,
        text: Option<&str>,
        image: Option<&ImageBlob>,
    ) -> std::result::Result<(), String> {
        if let Some(img) = image {
            self.set_image(img)?;
        }
        if let Some(t) = text {
            self.set_text(t)?;
        }
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
/// image is present). restore_if_owned replays whichever were present (the OS
/// clipboard can hold both flavors; restoring both honors the original state).
struct Saved {
    text: Option<String>, // None if there was no text
    image: Option<ImageBlob>,
    owned_sequence: u64, // sequence right after we wrote the sentinel
}

impl Saved {
    fn capture<C: ClipboardLike>(clip: &C) -> std::result::Result<Self, String> {
        // Text: None if absent/error (distinguish from empty-string text presence).
        let text = clip.get_text().ok();
        // Image: best-effort; errors → None.
        let image = clip.get_image().unwrap_or(None);
        Ok(Saved { text, image, owned_sequence: 0 })
    }

    /// Restore the snapshot ONLY if the clipboard sequence still equals owned. Per
    /// §B: if a newer writer landed since, do NOT clobber. Restores BOTH text and
    /// image (if present) via a single platform-level snapshot write — NOT two
    /// sequential set_text/set_image calls (which each clearContents and lose the
    /// other flavor on real arboard; round-2 review P1 #2). Errors are best-effort.
    fn restore_if_owned<C: ClipboardLike>(&self, clip: &C) {
        if clip.sequence() != self.owned_sequence {
            return; // newer writer — don't clobber
        }
        let _ = clip.restore_snapshot(self.text.as_deref(), self.image.as_ref());
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
    let sentinel = format!("__linguaray_sel_{}__", clip.sequence());
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
