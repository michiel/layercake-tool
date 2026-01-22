use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

pub fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("resources")
        .join("test-fixtures")
}

pub fn load_fixture(relative_path: &str) -> std::io::Result<Vec<u8>> {
    let path = fixtures_root().join(relative_path);
    fs::read(path)
}

pub fn golden_root() -> PathBuf {
    fixtures_root().join("golden")
}

pub fn load_golden(relative_path: &str) -> io::Result<Vec<u8>> {
    let path = golden_root().join(relative_path);
    fs::read(path)
}

pub fn load_golden_json<T: DeserializeOwned>(relative_path: &str) -> io::Result<T> {
    let bytes = load_golden(relative_path)?;
    serde_json::from_slice(&bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn write_golden(relative_path: &str, bytes: &[u8]) -> io::Result<PathBuf> {
    let path = golden_root().join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, bytes)?;
    Ok(path)
}
