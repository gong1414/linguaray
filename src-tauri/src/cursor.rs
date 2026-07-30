//! Screen cursor position (spec §C popup anchoring). Tauri 2 has no built-in
//! global cursor-position API, so we use the `mouse_position` crate.
pub fn position() -> (i32, i32) {
    use mouse_position::mouse_position::Mouse;
    match Mouse::get_mouse_position() {
        Mouse::Position { x, y } => (x, y),
        _ => (0, 0),
    }
}
