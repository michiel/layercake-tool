use std::collections::HashSet;

use crate::analyzer::AnalysisResult;

use super::graph::InfrastructureGraph;
use super::model::CorrelationMatch;

#[derive(Debug, Default, Clone)]
pub struct CorrelationReport {
    pub matches: Vec<CorrelationMatch>,
    pub unresolved: Vec<String>,
}

pub fn correlate_code_infra(
    code: &AnalysisResult,
    infra: &InfrastructureGraph,
) -> CorrelationReport {
    let mut report = CorrelationReport::default();
    let files: HashSet<String> = code.files.iter().cloned().collect();
    let functions: HashSet<String> = code.functions.iter().map(|f| f.name.clone()).collect();
    let env_vars: HashSet<String> = code.env_vars.iter().map(|e| e.name.clone()).collect();

    for resource in infra.resources.values() {
        let mut matched = false;
        for (k, v) in &resource.properties {
            // File match: property contains a code file path
            if files.iter().any(|f| v.contains(f)) {
                for f in files.iter().filter(|f| v.contains(*f)) {
                    report.matches.push(CorrelationMatch {
                        code_node: f.clone(),
                        infra_node: resource.id.clone(),
                        reason: format!("property references file {f}"),
                    });
                    matched = true;
                }
            }
            // Function/handler match by name
            if functions.iter().any(|func| v.contains(func)) {
                for func in functions.iter().filter(|func| v.contains(*func)) {
                    report.matches.push(CorrelationMatch {
                        code_node: func.clone(),
                        infra_node: resource.id.clone(),
                        reason: format!("property references handler/function {func}"),
                    });
                    matched = true;
                }
            }
            // Env var match by value
            if env_vars.iter().any(|env| v.contains(env)) {
                for env in env_vars.iter().filter(|env| v.contains(*env)) {
                    report.matches.push(CorrelationMatch {
                        code_node: env.clone(),
                        infra_node: resource.id.clone(),
                        reason: format!("property references env var {env}"),
                    });
                    matched = true;
                }
            }
            // Env var match by key
            if env_vars.contains(k) {
                report.matches.push(CorrelationMatch {
                    code_node: k.clone(),
                    infra_node: resource.id.clone(),
                    reason: format!("property key matches env var {k}"),
                });
                matched = true;
            }
        }

        if !matched {
            report
                .unresolved
                .push(format!("No code correlation for resource {}", resource.id));
        }
    }

    report
}
