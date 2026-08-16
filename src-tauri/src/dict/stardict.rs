//! StarDict parser (.ifo / .idx / .dict + optional .dict.dz).

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct StarDictInfo {
    pub bookname: String,
    pub word_count: usize,
    pub sametypesequence: Option<String>,
}

#[derive(Debug, Error)]
pub enum StarDictError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("no .ifo file found in {0}")]
    NoIfo(PathBuf),
    #[error("invalid .ifo: {0}")]
    InvalidIfo(String),
    #[error("invalid .idx entry")]
    InvalidIdx,
}

struct IdxEntry {
    word: String,
    offset: u32,
    size: u32,
}

pub struct StarDictParser {
    info: StarDictInfo,
    idx_entries: Vec<IdxEntry>,
    dict_path: PathBuf,
    dict_dz: bool,
}

impl StarDictParser {
    /// Open a StarDict directory. Finds the .ifo, .idx, and .dict (or .dict.dz).
    pub fn open(dir: &Path) -> Result<Self, StarDictError> {
        let ifo_path =
            find_file(dir, ".ifo").ok_or_else(|| StarDictError::NoIfo(dir.to_path_buf()))?;
        let ifo_content = std::fs::read_to_string(&ifo_path)?;
        let info = parse_ifo(&ifo_content)?;

        let idx_path = find_file(dir, ".idx")
            .ok_or_else(|| StarDictError::InvalidIfo("no .idx file".into()))?;
        let idx_data = std::fs::read(&idx_path)?;
        let idx_entries = parse_idx(&idx_data)?;

        let (dict_path, dict_dz) = if let Some(p) = find_file(dir, ".dict.dz") {
            (p, true)
        } else if let Some(p) = find_file(dir, ".dict") {
            (p, false)
        } else {
            return Err(StarDictError::InvalidIfo(
                "no .dict or .dict.dz file".into(),
            ));
        };

        Ok(Self {
            info,
            idx_entries,
            dict_path,
            dict_dz,
        })
    }

    pub fn info(&self) -> &StarDictInfo {
        &self.info
    }

    /// Look up a word. Returns the definition text or None.
    pub fn lookup(&self, word: &str) -> Result<Option<String>, StarDictError> {
        let found = self
            .idx_entries
            .binary_search_by(|e| e.word.as_str().cmp(word));
        let entry = match found {
            Ok(i) => &self.idx_entries[i],
            Err(_) => return Ok(None),
        };
        let bytes = self.read_definition(entry)?;
        if bytes.is_empty() {
            return Ok(Some(String::new()));
        }
        let text = match &self.info.sametypesequence {
            Some(seq) if seq.as_bytes().contains(&bytes[0]) => {
                String::from_utf8_lossy(&bytes[1..]).to_string()
            }
            _ => String::from_utf8_lossy(&bytes).to_string(),
        };
        Ok(Some(text.trim_end_matches('\0').to_string()))
    }

    fn read_definition(&self, entry: &IdxEntry) -> Result<Vec<u8>, StarDictError> {
        let offset = entry.offset as u64;
        let size = entry.size as usize;
        if self.dict_dz {
            let file = File::open(&self.dict_path)?;
            let mut decoder = GzDecoder::new(file);
            let mut all = Vec::new();
            decoder.read_to_end(&mut all)?;
            let start = offset as usize;
            if start
                .checked_add(size)
                .map(|end| end > all.len())
                .unwrap_or(true)
            {
                return Err(StarDictError::InvalidIdx);
            }
            Ok(all[start..start + size].to_vec())
        } else {
            let mut file = File::open(&self.dict_path)?;
            file.seek(SeekFrom::Start(offset))?;
            let mut buf = vec![0u8; size];
            file.read_exact(&mut buf)?;
            Ok(buf)
        }
    }
}

fn find_file(dir: &Path, suffix: &str) -> Option<PathBuf> {
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

fn parse_ifo(content: &str) -> Result<StarDictInfo, StarDictError> {
    let mut bookname = String::new();
    let mut word_count = 0usize;
    let mut sametypesequence = None;
    for line in content.lines() {
        if let Some(val) = line.strip_prefix("bookname=") {
            bookname = val.to_string();
        } else if let Some(val) = line.strip_prefix("wordcount=") {
            word_count = val.parse().unwrap_or(0);
        } else if let Some(val) = line.strip_prefix("sametypesequence=") {
            sametypesequence = Some(val.to_string());
        }
    }
    if bookname.is_empty() {
        return Err(StarDictError::InvalidIfo("missing bookname".into()));
    }
    Ok(StarDictInfo {
        bookname,
        word_count,
        sametypesequence,
    })
}

fn parse_idx(data: &[u8]) -> Result<Vec<IdxEntry>, StarDictError> {
    let mut entries = Vec::new();
    let mut pos = 0;
    while pos < data.len() {
        let null_pos = data[pos..]
            .iter()
            .position(|&b| b == 0)
            .ok_or(StarDictError::InvalidIdx)?;
        let word = String::from_utf8_lossy(&data[pos..pos + null_pos]).to_string();
        pos += null_pos + 1;
        if pos + 8 > data.len() {
            return Err(StarDictError::InvalidIdx);
        }
        let offset = u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        let size = u32::from_be_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]]);
        pos += 8;
        entries.push(IdxEntry { word, offset, size });
    }
    Ok(entries)
}
