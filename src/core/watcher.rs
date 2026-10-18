use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Default)]
pub struct DiskWatcher {
    path: Option<PathBuf>,
    last_modified: Option<SystemTime>,
}

impl DiskWatcher {
    pub fn new<P: AsRef<Path>>(path: Option<P>) -> Self {
        let mut watcher = Self {
            path: path.map(|p| p.as_ref().to_path_buf()),
            last_modified: None,
        };
        watcher.update_timestamp();
        watcher
    }

    pub fn for_path<P: AsRef<Path>>(path: P) -> Self {
        Self::new(Some(path))
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn set_path<P: AsRef<Path>>(&mut self, path: Option<P>) {
        self.path = path.map(|p| p.as_ref().to_path_buf());
        self.update_timestamp();
    }

    pub fn last_modified(&self) -> Option<SystemTime> {
        self.last_modified
    }

    pub fn update_timestamp(&mut self) {
        self.last_modified = self.path.as_ref().and_then(|p| {
            fs::metadata(p).ok().and_then(|meta| meta.modified().ok())
        });
    }

    pub fn check_for_external_change(&mut self) -> bool {
        let Some(path) = &self.path else {
            return false;
        };

        let current_modified = fs::metadata(path).ok().and_then(|m| m.modified().ok());

        match (self.last_modified, current_modified) {
            (Some(last), Some(curr)) => {
                if curr > last {
                    self.last_modified = Some(curr);
                    true
                } else {
                    false
                }
            }
            (None, Some(curr)) => {
                self.last_modified = Some(curr);
                true
            }
            _ => false,
        }
    }
}
