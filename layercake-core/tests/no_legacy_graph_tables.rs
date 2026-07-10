//! Phase 0 cutover guard.
//!
//! The legacy graph tables (`graphs`, `graph_nodes`, `graph_edges`,
//! `dataset_graph_nodes`, `dataset_graph_edges`, `dataset_graph_layers`) were
//! dropped by migration `m20251215_000001_drop_legacy_graph_tables`. The unified
//! `graph_data` schema is the single source of truth. This test fails if any
//! source file outside `migrations/` reintroduces an ORM reference to one of
//! those dropped tables, so the incomplete-cutover class of bug (see
//! `plans/20260710-phase0-graph-data-cutover.md`) cannot silently return.
//!
//! Note: `graph_layers` is intentionally NOT guarded yet — the per-graph
//! layer-editing surface is still being migrated to `project_layers` (WS3
//! deferred item). Add it here once that work lands.

use std::fs;
use std::path::{Path, PathBuf};

/// Entity modules for tables dropped in the graph_data cutover.
const DROPPED_TABLES: &[&str] = &[
    "graphs",
    "graph_nodes",
    "graph_edges",
    "dataset_graph_nodes",
    "dataset_graph_edges",
    "dataset_graph_layers",
];

/// SeaORM item suffixes that indicate a real code reference (not prose).
const ENTITY_SUFFIXES: &[&str] = &["Entity", "Model", "Column", "ActiveModel"];

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is <workspace>/layercake-core at compile time.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("layercake-core has a parent workspace directory")
        .to_path_buf()
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip the migrations tree: it legitimately names dropped tables.
            if path.file_name().is_some_and(|n| n == "migrations") {
                continue;
            }
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn no_references_to_dropped_graph_tables() {
    let root = workspace_root();
    let scan_dirs = [
        "layercake-core/src",
        "layercake-server/src",
        "layercake-cli/src",
        "layercake-projections/src",
    ];

    // Precompute the forbidden needles, e.g. "graphs::Entity", "graph_nodes::Model".
    let needles: Vec<String> = DROPPED_TABLES
        .iter()
        .flat_map(|table| {
            ENTITY_SUFFIXES
                .iter()
                .map(move |suffix| format!("{table}::{suffix}"))
        })
        .collect();

    let mut violations: Vec<String> = Vec::new();

    for rel in scan_dirs {
        let dir = root.join(rel);
        let mut files = Vec::new();
        collect_rs_files(&dir, &mut files);

        for file in files {
            let Ok(contents) = fs::read_to_string(&file) else {
                continue;
            };
            for (idx, line) in contents.lines().enumerate() {
                for needle in &needles {
                    // `dataset_graph_nodes::Entity` legitimately ends with
                    // `graph_nodes::Entity`; that is still a dropped-table
                    // reference, so a substring match is exactly what we want.
                    if line.contains(needle.as_str()) {
                        violations.push(format!(
                            "{}:{}: references dropped table entity `{}`",
                            file.strip_prefix(&root).unwrap_or(&file).display(),
                            idx + 1,
                            needle
                        ));
                    }
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Found references to legacy graph tables dropped in the graph_data cutover.\n\
         Use the unified graph_data / project_layers entities instead.\n{}",
        violations.join("\n")
    );
}
