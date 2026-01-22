use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Deserialize)]
struct Manifest {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    name: String,
    path: String,
    description: Option<String>,
}

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("resources")
        .join("test-fixtures")
        .join("golden")
        .join("errors")
        .join("manifest.json")
}

#[test]
fn graphql_error_baselines_are_valid_json() {
    let manifest_path = manifest_path();
    let manifest_bytes = fs::read(&manifest_path)
        .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", manifest_path, e));
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .unwrap_or_else(|e| panic!("Invalid manifest JSON: {}", e));

    for case in manifest.cases {
        let payload_path = manifest_path.parent().unwrap().join(&case.path);
        let payload = fs::read(&payload_path).unwrap_or_else(|e| {
            panic!(
                "Missing baseline {} at {:?}: {}",
                case.name, payload_path, e
            )
        });
        let _: serde_json::Value = serde_json::from_slice(&payload).unwrap_or_else(|e| {
            panic!(
                "Baseline {} ({:?}) is not valid JSON: {}",
                case.name, payload_path, e
            )
        });
        let _ = case.description;
    }
}
