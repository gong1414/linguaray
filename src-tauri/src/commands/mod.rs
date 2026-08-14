//! Tauri command modules. Wire names and JSON are unchanged (plugin-core PR-3).

pub mod history;
pub mod keystore;
pub mod settings;
pub mod shortcuts;

pub use history::*;
pub use keystore::*;
pub use settings::*;
pub use shortcuts::*;
