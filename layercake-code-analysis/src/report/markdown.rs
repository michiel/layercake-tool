use crate::analyzer::{AnalysisResult, LayercakeDatasetConvertible};
use crate::report::ReportMetadata;
use anyhow::Result;

#[derive(Default)]
pub struct MarkdownReporter;

impl MarkdownReporter {
    pub fn render(&self, result: &AnalysisResult, metadata: &ReportMetadata) -> Result<String> {
        let mut output = String::new();
        output.push_str("# Code Analysis Report\n");
        output.push_str(&format!("{}\n\n", metadata.summary(result)));

        for dataset in result.to_layercake_datasets() {
            output.push_str(&format!("## {}\n", dataset.name));
            let csv = dataset.to_csv()?;
            output.push_str("```CSV\n");
            output.push_str(&csv);
            if !csv.ends_with('\n') {
                output.push('\n');
            }
            output.push_str("```\n\n");
        }

        Ok(output)
    }
}
