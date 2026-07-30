//! Wires the §B engine to the real OS clipboard + enigo keystroke simulation.
use crate::clipboard;
use crate::selection_engine::{self, Capture, ClipboardLike};

struct OsClipboard;
impl ClipboardLike for OsClipboard {
    fn get_text(&self) -> Result<String, String> { clipboard::get_text() }
    fn set_text(&self, s: &str) -> Result<(), String> { clipboard::set_text(s) }
    fn sequence(&self) -> u64 { clipboard::sequence() }
}

/// Simulate the platform copy keystroke: Cmd+C on macOS, Ctrl+C elsewhere.
fn simulate_copy() -> Result<(), String> {
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    let modifier = Key::Meta;
    #[cfg(not(target_os = "macos"))]
    let modifier = Key::Control;
    // The draft used Layout('c'); enigo 0.2.1 has no Layout variant — Unicode('c')
    // routes through the layout/keymap (keysym), which is what we want.
    enigo
        .key(modifier, Direction::Press)
        .map_err(|e| e.to_string())?;
    enigo
        .key(Key::Unicode('c'), Direction::Click)
        .map_err(|e| e.to_string())?;
    enigo
        .key(modifier, Direction::Release)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Capture the current selection via the §B algorithm. ~timeout_ms total.
pub fn capture_selection(timeout_ms: u64) -> Result<Capture, String> {
    let iters = (timeout_ms / 20) as usize;
    selection_engine::capture(&OsClipboard, || simulate_copy(), iters)
}
