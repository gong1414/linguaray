//! MDict (.mdx) parser — minimal v2 subset.
//!
//! Layout:
//!   magic `MDCT` + BE u32 header size + header bytes (zlib or raw)
//!   BE u32 key count
//!   keys: `key\0` + BE u64 record offset + BE u32 record size
//!   records: raw UTF-8 blobs addressed by offset from the record block start.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MdxError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid MDX magic")]
    InvalidMagic,
    #[error("unsupported MDX version: {0}")]
    UnsupportedVersion(f64),
}

struct MdxKeyEntry {
    key: String,
    record_offset: u64,
    record_size: u32,
}

pub struct MdxParser {
    keys: Vec<MdxKeyEntry>,
    file: File,
    record_block_start: u64,
}

impl MdxParser {
    pub fn open(path: &Path) -> Result<Self, MdxError> {
        let mut file = File::open(path)?;
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        if magic != *b"MDCT" {
            return Err(MdxError::InvalidMagic);
        }
        let mut hdr_size_bytes = [0u8; 4];
        file.read_exact(&mut hdr_size_bytes)?;
        let header_block_size = u32::from_be_bytes(hdr_size_bytes) as usize;
        let mut header_bytes = vec![0u8; header_block_size];
        file.read_exact(&mut header_bytes)?;
        let header_text = decompress_zlib(&header_bytes).unwrap_or(header_bytes);
        let header_str = String::from_utf8_lossy(&header_text);
        let version = parse_version(&header_str);
        if version < 2.0 {
            return Err(MdxError::UnsupportedVersion(version));
        }

        let mut count_bytes = [0u8; 4];
        file.read_exact(&mut count_bytes)?;
        let key_count = u32::from_be_bytes(count_bytes) as usize;
        let mut keys = Vec::with_capacity(key_count);
        for _ in 0..key_count {
            let mut key_bytes = Vec::new();
            loop {
                let mut b = [0u8; 1];
                file.read_exact(&mut b)?;
                if b[0] == 0 {
                    break;
                }
                key_bytes.push(b[0]);
            }
            let mut offset_bytes = [0u8; 8];
            file.read_exact(&mut offset_bytes)?;
            let mut size_bytes = [0u8; 4];
            file.read_exact(&mut size_bytes)?;
            keys.push(MdxKeyEntry {
                key: String::from_utf8_lossy(&key_bytes).to_string(),
                record_offset: u64::from_be_bytes(offset_bytes),
                record_size: u32::from_be_bytes(size_bytes),
            });
        }
        keys.sort_by(|a, b| a.key.cmp(&b.key));
        let record_block_start = file.stream_position()?;
        Ok(Self {
            keys,
            file,
            record_block_start,
        })
    }

    pub fn lookup(&mut self, word: &str) -> Result<Option<String>, MdxError> {
        let found = self.keys.binary_search_by(|e| e.key.as_str().cmp(word));
        let entry = match found {
            Ok(i) => &self.keys[i],
            Err(_) => return Ok(None),
        };
        let record = self.read_record(entry.record_offset, entry.record_size)?;
        Ok(Some(record))
    }

    fn read_record(&mut self, offset: u64, size: u32) -> Result<String, MdxError> {
        self.file
            .seek(SeekFrom::Start(self.record_block_start + offset))?;
        let mut buf = vec![0u8; size as usize];
        self.file.read_exact(&mut buf)?;
        let decompressed = decompress_zlib(&buf).unwrap_or(buf);
        Ok(String::from_utf8_lossy(&decompressed).to_string())
    }
}

fn parse_version(xml: &str) -> f64 {
    xml.split("GeneratedByEngineVersion=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .and_then(|v| v.parse().ok())
        .unwrap_or(2.0)
}

fn decompress_zlib(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decoder = ZlibDecoder::new(data);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out)?;
    Ok(out)
}

/// Write a minimal v2 fixture the parser can open. Used by tests.
pub fn write_minimal_fixture(path: &Path, entries: &[(&str, &str)]) -> Result<(), MdxError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let header = br#"<Dictionary GeneratedByEngineVersion="2.0" Encoding="UTF-8"/>"#;
    let mut header_z = ZlibEncoder::new(Vec::new(), Compression::default());
    header_z.write_all(header)?;
    let header_bytes = header_z.finish()?;

    let mut records = Vec::new();
    let mut key_block = Vec::new();
    key_block.extend_from_slice(&(entries.len() as u32).to_be_bytes());
    for (key, def) in entries {
        key_block.extend_from_slice(key.as_bytes());
        key_block.push(0);
        key_block.extend_from_slice(&(records.len() as u64).to_be_bytes());
        key_block.extend_from_slice(&(def.len() as u32).to_be_bytes());
        records.extend_from_slice(def.as_bytes());
    }

    let mut file = File::create(path)?;
    file.write_all(b"MDCT")?;
    file.write_all(&(header_bytes.len() as u32).to_be_bytes())?;
    file.write_all(&header_bytes)?;
    file.write_all(&key_block)?;
    file.write_all(&records)?;
    Ok(())
}
