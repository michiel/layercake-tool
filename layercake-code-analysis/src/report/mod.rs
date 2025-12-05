use crate::analyzer::AnalysisResult;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

pub mod markdown;

#[derive(Debug, Clone)]
pub struct ReportMetadata {
    pub root_path: PathBuf,
    pub files_scanned: usize,
    pub generated_at: DateTime<Utc>,
}

impl ReportMetadata {
    pub fn new(root_path: PathBuf, files_scanned: usize) -> Self {
        Self {
            root_path,
            files_scanned,
            generated_at: Utc::now(),
        }
    }
}

impl ReportMetadata {
    pub fn summary(&self, result: &AnalysisResult) -> String {
        format!(
            "- Path: {}\n- Files scanned: {}\n- Imports: {}\n- Functions: {}\n- Data flows: {}\n- Entry points: {}\n- Generated: {}",
            self.root_path.display(),
            self.files_scanned,
            result.imports.len(),
            result.functions.len(),
            result.data_flows.len(),
            result.entry_points.len(),
            self.generated_at.to_rfc3339()
        )
    }
}
