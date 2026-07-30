//! The §B sentinel clipboard state machine, decoupled from the OS clipboard via a
//! trait so it is unit-testable with a fake. Real wiring (enigo Cmd+C, real
//! clipboard.rs) lives in selection.rs (Task 5).

/// What the engine needs from a clipboard. The real impl wraps `clipboard.rs`;
/// tests use a fake.
pub trait ClipboardLike {
    fn get_text(&self) -> std::result::Result<String, String>;
    fn set_text(&self, s: &str) -> std::result::Result<(), String>;
    /// Monotonic sequence number that advances on ANY write (ours included).
    fn sequence(&self) -> u64;
}

pub enum Capture {
    Selected(String),
    NoSelection,
}

/// Run the §B algorithm. `copy` is the simulated-copy step (Cmd+C/Ctrl+C),
/// injected so the engine stays pure/testable. Returns the selected text or
/// NoSelection (sentinel still present after `copy` ran => nothing was selected).
pub fn capture<C: ClipboardLike, F: FnMut() -> std::result::Result<(), String>>(
    clip: &C,
    mut copy: F,
    timeout_iters: usize,
) -> std::result::Result<Capture, String> {
    // 1. Save current content (best-effort; ignore read errors => empty).
    let saved = clip.get_text().unwrap_or_default();
    // 2. Write a unique sentinel.
    let sentinel = format!("__islandpot_sel_{}__", clip.sequence());
    clip.set_text(&sentinel)?;
    let marker_sequence = clip.sequence();
    // 3. Simulate copy.
    copy()?;
    // 4. Bounded-wait for the sequence to leave the marker (a successful copy
    //    overwrites the sentinel, advancing the sequence).
    let mut waited = 0usize;
    let mut now = clip.sequence();
    while now == marker_sequence && waited < timeout_iters {
        std::thread::sleep(std::time::Duration::from_millis(20));
        now = clip.sequence();
        waited += 1;
    }
    if now == marker_sequence {
        // Copy didn't happen / nothing selected. Restore saved, return NoSelection.
        let _ = clip.set_text(&saved);
        return Ok(Capture::NoSelection);
    }
    // 5. Copy succeeded: read selection, record owned_sequence, restore only if
    //    nothing else wrote since.
    let owned_sequence = clip.sequence();
    let text = clip.get_text()?;
    if clip.sequence() == owned_sequence {
        let _ = clip.set_text(&saved);
    } // else: newer content — don't clobber.
    Ok(Capture::Selected(text))
}
