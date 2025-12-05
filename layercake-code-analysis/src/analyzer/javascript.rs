use super::{AnalysisResult, Analyzer};
use anyhow::{anyhow, Result};
use std::path::Path;

#[derive(Default)]
pub struct JavascriptAnalyzer;

impl Analyzer for JavascriptAnalyzer {
    fn supports(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| {
                matches!(
                    ext.to_ascii_lowercase().as_str(),
                    "js" | "jsx" | "ts" | "tsx"
                )
            })
            .unwrap_or(false)
    }

    fn analyze(&self, _path: &Path) -> Result<AnalysisResult> {
        Err(anyhow!(
            "JavaScript analysis is not implemented yet for this version"
        ))
    }

    fn language(&self) -> &'static str {
        "javascript"
    }
}
