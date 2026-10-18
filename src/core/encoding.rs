use std::fs;
use std::io::{Error, ErrorKind};
use std::path::Path;

pub const UTF8_BOM: &[u8; 3] = &[0xEF, 0xBB, 0xBF];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineEnding {
    #[default]
    LF,
    CRLF,
}

impl LineEnding {
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::LF => "\n",
            LineEnding::CRLF => "\r\n",
        }
    }

    pub fn detect(content: &str) -> Self {
        let crlf_count = content.matches("\r\n").count();
        let total_lf = content.matches('\n').count();
        let lone_lf_count = total_lf.saturating_sub(crlf_count);

        if crlf_count > lone_lf_count {
            LineEnding::CRLF
        } else if lone_lf_count > 0 {
            LineEnding::LF
        } else {
            #[cfg(windows)]
            {
                LineEnding::CRLF
            }
            #[cfg(not(windows))]
            {
                LineEnding::LF
            }
        }
    }

    pub fn normalize(&self, content: &str) -> String {
        let lf_only = content.replace("\r\n", "\n").replace('\r', "\n");
        match self {
            LineEnding::LF => lf_only,
            LineEnding::CRLF => lf_only.replace('\n', "\r\n"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedDocument {
    pub content: String,
    pub has_bom: bool,
    pub line_ending: LineEnding,
}

pub fn load_from_bytes(bytes: &[u8]) -> Result<LoadedDocument, Error> {
    let (content_bytes, has_bom) = if bytes.starts_with(UTF8_BOM) {
        (&bytes[3..], true)
    } else {
        (bytes, false)
    };

    let content = std::str::from_utf8(content_bytes)
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("UTF-8 decode error: {e}")))?
        .to_string();

    let line_ending = LineEnding::detect(&content);

    Ok(LoadedDocument {
        content,
        has_bom,
        line_ending,
    })
}

pub fn encode_to_bytes(content: &str, has_bom: bool, line_ending: LineEnding) -> Vec<u8> {
    let normalized = line_ending.normalize(content);
    let mut bytes = Vec::with_capacity((if has_bom { 3 } else { 0 }) + normalized.len());
    if has_bom {
        bytes.extend_from_slice(UTF8_BOM);
    }
    bytes.extend_from_slice(normalized.as_bytes());
    bytes
}

pub fn load_file<P: AsRef<Path>>(path: P) -> Result<LoadedDocument, Error> {
    let raw = fs::read(path)?;
    load_from_bytes(&raw)
}

pub fn save_file<P: AsRef<Path>>(
    path: P,
    content: &str,
    has_bom: bool,
    line_ending: LineEnding,
) -> Result<(), Error> {
    let bytes = encode_to_bytes(content, has_bom, line_ending);
    fs::write(path, bytes)
}
