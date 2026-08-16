//! Tauri command modules. Wire names and JSON are unchanged (plugin-core PR-3).

pub mod history;
pub mod keystore;
pub mod providers;
pub mod settings;
pub mod shortcuts;
pub mod translate;
pub mod vocabulary;
pub mod dict;
pub mod ocr;
pub mod tts;
pub mod external_api;
pub mod updater;
pub mod onboarding;

pub use history::*;
pub use keystore::*;
pub use providers::*;
pub use settings::*;
pub use shortcuts::*;
pub use translate::*;
pub use vocabulary::*;
pub use dict::*;
pub use ocr::*;
pub use tts::*;
pub use external_api::*;
pub use updater::*;
pub use onboarding::*;
