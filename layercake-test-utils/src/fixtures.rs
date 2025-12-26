use std::fs;
use std::path::{Path, PathBuf};

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
