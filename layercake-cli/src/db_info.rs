//! `layercake db info` — report the database file location and size.
//!
//! Filesystem-only by default (no DB connection), so it is safe to run against
//! a database that a server currently has open. Useful for agents that want to
//! inspect or back up the SQLite file directly.

use anyhow::Result;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct DbInfo {
    path: String,
    absolute_path: Option<String>,
    exists: bool,
    size_bytes: u64,
    size_mb: String,
}

/// Print database file info as human-readable text or JSON.
pub fn run(database: &str, json: bool) -> Result<()> {
    let path = Path::new(database);
    let exists = path.exists();
    let size_bytes = if exists {
        std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };
    let absolute_path = std::fs::canonicalize(path)
        .ok()
        .map(|p| p.display().to_string());

    let info = DbInfo {
        path: database.to_string(),
        absolute_path,
        exists,
        size_bytes,
        size_mb: format!("{:.2}", size_bytes as f64 / (1024.0 * 1024.0)),
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
        println!("Database:  {}", info.path);
        if let Some(abs) = &info.absolute_path {
            println!("Location:  {}", abs);
        }
        println!("Exists:    {}", info.exists);
        if info.exists {
            println!("Size:      {} MB ({} bytes)", info.size_mb, info.size_bytes);
        }
    }

    Ok(())
}
