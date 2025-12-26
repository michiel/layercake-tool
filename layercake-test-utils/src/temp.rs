use std::fs;
use std::path::{Path, PathBuf};

use uuid::Uuid;

pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn new() -> std::io::Result<Self> {
        let mut path = std::env::temp_dir();
        path.push(format!("layercake-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
