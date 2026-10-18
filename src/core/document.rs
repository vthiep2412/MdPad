use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

use crate::core::encoding::{load_file, save_file, LineEnding};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseDecision {
    Save,
    DontSave,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DocumentStats {
    pub line_count: usize,
    pub word_count: usize,
    pub char_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    path: Option<PathBuf>,
    content: String,
    is_dirty: bool,
    has_bom: bool,
    line_ending: LineEnding,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Document {
    pub fn new() -> Self {
        Self {
            path: None,
            content: String::new(),
            is_dirty: false,
            has_bom: false,
            line_ending: LineEnding::default(),
        }
    }

    pub fn with_content(content: String) -> Self {
        let line_ending = LineEnding::detect(&content);
        Self {
            path: None,
            content,
            is_dirty: false,
            has_bom: false,
            line_ending,
        }
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path_buf = path.as_ref().to_path_buf();
        let loaded = load_file(&path_buf)?;

        Ok(Self {
            path: Some(path_buf),
            content: loaded.content,
            is_dirty: false,
            has_bom: loaded.has_bom,
            line_ending: loaded.line_ending,
        })
    }

    pub fn save(&mut self) -> Result<(), Error> {
        let Some(ref path) = self.path else {
            return Err(Error::new(
                ErrorKind::NotFound,
                "Cannot save document without an associated path",
            ));
        };

        save_file(path, &self.content, self.has_bom, self.line_ending)?;
        self.is_dirty = false;
        Ok(())
    }

    pub fn save_as<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let path_buf = path.as_ref().to_path_buf();
        save_file(&path_buf, &self.content, self.has_bom, self.line_ending)?;
        self.path = Some(path_buf);
        self.is_dirty = false;
        Ok(())
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn set_content(&mut self, new_content: String) {
        if self.content != new_content {
            self.content = new_content;
            self.is_dirty = true;
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.is_dirty = dirty;
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn set_path(&mut self, path: Option<PathBuf>) {
        self.path = path;
    }

    pub fn file_name(&self) -> &str {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
    }

    pub fn window_title(&self) -> String {
        let name = self.file_name();
        let prefix = if self.is_dirty { "*" } else { "" };
        format!("{prefix}{name} - MdPad")
    }

    pub fn has_bom(&self) -> bool {
        self.has_bom
    }

    pub fn set_has_bom(&mut self, has_bom: bool) {
        if self.has_bom != has_bom {
            self.has_bom = has_bom;
            self.is_dirty = true;
        }
    }

    pub fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    pub fn set_line_ending(&mut self, line_ending: LineEnding) {
        if self.line_ending != line_ending {
            self.line_ending = line_ending;
            self.is_dirty = true;
        }
    }

    pub fn stats(&self) -> DocumentStats {
        calculate_document_stats(&self.content)
    }

    pub fn cursor_position(&self, byte_offset: usize) -> (usize, usize) {
        calculate_cursor_position(&self.content, byte_offset)
    }
}

pub fn calculate_document_stats(text: &str) -> DocumentStats {
    let line_count = if text.is_empty() {
        1
    } else {
        text.split('\n').count()
    };
    let word_count = text.split_whitespace().count();
    let char_count = text.chars().count();

    DocumentStats {
        line_count,
        word_count,
        char_count,
    }
}

pub fn calculate_cursor_position(text: &str, byte_offset: usize) -> (usize, usize) {
    let offset = byte_offset.min(text.len());
    let prefix = &text[..offset];
    let line = prefix.matches('\n').count() + 1;
    let last_newline_pos = prefix.rfind('\n');
    let col_slice = match last_newline_pos {
        Some(pos) => &prefix[pos + 1..],
        None => prefix,
    };
    let col_chars = col_slice.trim_matches('\r').chars().count();
    let column = col_chars + 1;
    (line, column)
}
