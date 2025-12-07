use layercake_code_analysis::analyzer::analyze_path;
use std::fs;
use std::path::Path;

#[test]
fn analyzes_python_project_and_reports_metrics() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let file_path = temp_dir.path().join("example.py");
    let content = r#"
import os
from typing import List

def fetch(value: int) -> int:
    return value + 1

class Runner:
    def process(self, data: int):
        if data:
            return data
        return 0

    def main(self):
        value = fetch(1)
        alias = value
        self.process(alias)

if __name__ == "__main__":
    Runner().main()
"#;
    fs::write(&file_path, content).expect("write sample python");

    let run = analyze_path(temp_dir.path()).expect("analysis run");
    assert_eq!(run.files_scanned, 1);
    let result = run.result;

    assert_eq!(result.imports.len(), 2);
    assert_eq!(result.entry_points.len(), 1);
    assert_eq!(result.data_flows.len(), 1);

    let function_names: Vec<_> = result.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(function_names.contains(&"fetch"));
    assert!(function_names.contains(&"Runner.process"));
    assert!(function_names.contains(&"Runner.main"));

    let main_fn = result
        .functions
        .iter()
        .find(|f| f.name == "Runner.main")
        .expect("main function exists");
    assert_eq!(main_fn.complexity, 1);
}

#[test]
fn analyzes_javascript_project() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let file_path = temp_dir.path().join("example.js");
    let content = r#"
import fs from 'fs';

function fetch() {
  return apiCall();
}

class Runner {
  process(data) {
    if (data && data.ok) {
      return data;
    }
    return null;
  }

  main() {
    const value = fetch();
    const alias = value;
    this.process(alias);
  }
}

if (require.main === module) {
  new Runner().main();
}
"#;
    fs::write(&file_path, content).expect("write sample js");

    let run = analyze_path(temp_dir.path()).expect("analysis run");
    assert_eq!(run.files_scanned, 1);
    let result = run.result;

    assert!(!result.functions.is_empty());
    assert_eq!(result.entry_points.len(), 1);
    assert!(!result.data_flows.is_empty());

    let imports: Vec<_> = result.imports.iter().map(|i| i.module.as_str()).collect();
    assert!(imports.contains(&"fs"));
}

#[test]
fn analyzes_agentcore_onboarding_reference_project() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");
    let sample_path =
        repo_root.join("resources/reference-codebases/sample-amazon-bedrock-agentcore-onboarding");
    assert!(
        sample_path.exists(),
        "expected sample project at {:?}",
        sample_path
    );

    let run = analyze_path(&sample_path).expect("analysis run");
    assert!(
        run.files_scanned >= 10,
        "should scan many files, got {}",
        run.files_scanned
    );

    let result = run.result;
    assert!(
        !result.functions.is_empty(),
        "should discover functions in reference project"
    );
    assert!(
        !result.imports.is_empty(),
        "should discover imports in reference project"
    );
    assert!(
        !result.data_flows.is_empty() || !result.call_edges.is_empty(),
        "should discover flows/calls in reference project"
    );

    let has_dir = |needle: &str| result.directories.iter().any(|d| d.contains(needle));
    for dir in [
        "01_code_interpreter",
        "02_runtime",
        "03_identity",
        "04_gateway",
        "05_observability",
        "06_memory",
    ] {
        assert!(
            has_dir(dir),
            "expected directory '{}' to be captured in analysis",
            dir
        );
    }

    let has_func_in_dir = |needle: &str| {
        result
            .functions
            .iter()
            .any(|f| f.file_path.contains(needle))
    };

    assert!(
        has_func_in_dir("01_code_interpreter"),
        "code interpreter functions should be present"
    );
    assert!(
        has_func_in_dir("02_runtime"),
        "runtime functions should be present"
    );
    assert!(
        has_func_in_dir("03_identity"),
        "identity functions should be present"
    );
    assert!(
        has_func_in_dir("04_gateway"),
        "gateway functions should be present"
    );
    assert!(
        has_func_in_dir("05_observability"),
        "observability functions should be present"
    );
    assert!(
        has_func_in_dir("06_memory"),
        "memory functions should be present"
    );
}
