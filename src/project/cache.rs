use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

use super::blocks::Block;
use super::encoding;
use crate::error::HmsError;

struct Entry {
    modified: SystemTime,
    blocks: Vec<Block>,
}

/// Caché de archivos de proyecto, invalidada por la fecha de modificación.
#[derive(Default)]
pub struct BlockCache {
    inner: Mutex<HashMap<PathBuf, Entry>>,
}

impl BlockCache {
    pub fn load(&self, path: &Path) -> Result<Vec<Block>, HmsError> {
        let modified = std::fs::metadata(path)?.modified()?;
        let mut guard = self.inner.lock().unwrap_or_else(|err| err.into_inner());
        if let Some(entry) = guard.get(path) {
            if entry.modified == modified {
                return Ok(entry.blocks.clone());
            }
        }
        let blocks = super::blocks::parse(&encoding::read_text(path)?);
        guard.insert(
            path.to_path_buf(),
            Entry {
                modified,
                blocks: blocks.clone(),
            },
        );
        Ok(blocks)
    }

    pub fn forget(&self, path: &Path) {
        let mut guard = self.inner.lock().unwrap_or_else(|err| err.into_inner());
        guard.remove(path);
    }
}
