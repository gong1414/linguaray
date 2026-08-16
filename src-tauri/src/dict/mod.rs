//! Dictionary subsystem: StarDict/MDX parsers, hardened package install, unified lookup.

use std::path::{Path, PathBuf};

pub mod lookup;
pub mod mdx;
pub mod package;
pub mod stardict;

pub use lookup::{lookup, DictLookupResult};
pub use package::{install_package, list_packages, DictPackageInfo};

/// App-owned dictionary root: sibling of the SQLite file.
pub fn dictionaries_dir(db_path: &Path) -> PathBuf {
    db_path.parent().unwrap_or(db_path).join("dictionaries")
}
