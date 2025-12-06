use crate::analyzer::AnalysisResult;

use super::graph::InfrastructureGraph;
use super::model::CorrelationMatch;

#[derive(Debug, Default, Clone)]
pub struct CorrelationReport {
    pub matches: Vec<CorrelationMatch>,
    pub unresolved: Vec<String>,
}

pub fn correlate_code_infra(
    _code: &AnalysisResult,
    _infra: &InfrastructureGraph,
) -> CorrelationReport {
    // Placeholder: future implementations will match env vars, handlers, and imports.
    CorrelationReport::default()
}
