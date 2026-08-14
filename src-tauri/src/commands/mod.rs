//! Tauri command modules. Wire names and JSON are unchanged (plugin-core PR-3).

pub mod history;
pub mod keystore;
pub mod providers;
pub mod settings;
pub mod shortcuts;
pub mod translate;
pub mod vocabulary;
pub mod dict;

pub use history::*;
pub use keystore::*;
pub use providers::*;
pub use settings::*;
pub use shortcuts::*;
pub use translate::*;
pub use vocabulary::*;
pub use dict::*;
