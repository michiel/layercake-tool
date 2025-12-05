use layercake_code_analysis::analyzer::analyze_path;
use std::fs;

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
