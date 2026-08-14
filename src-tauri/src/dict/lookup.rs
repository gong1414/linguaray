//! Unified dictionary lookup: macOS system dict + offline packages.

use std::path::Path;

use serde::Serialize;
use thiserror::Error;

use crate::db::Database;
use crate::dict::mdx::MdxParser;
use crate::dict::package::{self as pkg, DictPackageInfo};
use crate::dict::stardict::StarDictParser;

#[derive(Debug, Clone, Serialize)]
pub struct DictLookupResult {
    pub definition: String,
    pub source: String,
}

#[derive(Debug, Error)]
pub enum LookupError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("db: {0}")]
    Db(#[from] crate::db::DbError),
    #[error("stardict: {0}")]
    StarDict(#[from] crate::dict::stardict::StarDictError),
}

const MACOS_SOURCE: &str = "macOS System Dictionary";

/// macOS system dictionary via DCSCopyTextDefinition.
#[cfg(target_os = "macos")]
pub fn macos_system_lookup(word: &str) -> Option<String> {
    use core_foundation::base::{CFRange, TCFType};
    use core_foundation::string::{CFString, CFStringRef};

    #[link(name = "CoreServices", kind = "framework")]
    extern "C" {
        fn DCSCopyTextDefinition(
            dict: *const std::ffi::c_void,
            text: CFStringRef,
            range: CFRange,
        ) -> CFStringRef;
    }

    unsafe {
        let cf_word = CFString::new(word);
        let range = CFRange {
            location: 0,
            length: cf_word.char_len(),
        };
        let result = DCSCopyTextDefinition(std::ptr::null(), cf_word.as_concrete_TypeRef(), range);
        if result.is_null() {
            return None;
        }
        let def = CFString::wrap_under_create_rule(result).to_string();
        if def.is_empty() {
            None
        } else {
            Some(def)
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn macos_system_lookup(_word: &str) -> Option<String> {
    None
}

/// Unified lookup: try macOS system dict, then installed StarDict/MDX packages.
pub fn lookup(
    db: &Database,
    dict_dir: &Path,
    word: &str,
) -> Result<Option<DictLookupResult>, LookupError> {
    if let Some(def) = macos_system_lookup(word) {
        return Ok(Some(DictLookupResult {
            definition: def,
            source: MACOS_SOURCE.to_string(),
        }));
    }

    let packages: Vec<DictPackageInfo> = db.with_conn(|conn| pkg::list_packages(conn))?;
    for pkg_info in &packages {
        let pkg_dir = dict_dir.join(&pkg_info.package_id);
        if !pkg_dir.exists() {
            continue;
        }
        if let Ok(parser) = StarDictParser::open(&pkg_dir) {
            if let Ok(Some(def)) = parser.lookup(word) {
                return Ok(Some(DictLookupResult {
                    definition: def,
                    source: pkg_info.name.clone(),
                }));
            }
        }
        if let Some(mdx_path) = find_suffix(&pkg_dir, ".mdx") {
            if let Ok(mut parser) = MdxParser::open(&mdx_path) {
                if let Ok(Some(def)) = parser.lookup(word) {
                    return Ok(Some(DictLookupResult {
                        definition: def,
                        source: pkg_info.name.clone(),
                    }));
                }
            }
        }
    }
    Ok(None)
}

fn find_suffix(dir: &Path, suffix: &str) -> Option<std::path::PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .find_map(|e| {
            let path = e.path();
            let matches = path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(suffix));
            matches.then_some(path)
        })
}
