use std::collections::HashSet;

use crate::analyzer::AnalysisResult;

use super::graph::InfrastructureGraph;
use super::model::CorrelationMatch;

#[derive(Debug, Default, Clone)]
pub struct CorrelationReport {
    pub matches: Vec<CorrelationMatch>,
    pub unresolved: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn correlate_code_infra(
    code: &AnalysisResult,
    infra: &InfrastructureGraph,
) -> CorrelationReport {
    let mut report = CorrelationReport::default();
    let files: HashSet<String> = code.files.iter().cloned().collect();
    let file_names: HashSet<String> = files
        .iter()
        .filter_map(|f| std::path::Path::new(f).file_name().and_then(|n| n.to_str()))
        .map(|s| s.to_ascii_lowercase())
        .collect();
    let functions: HashSet<String> = code.functions.iter().map(|f| f.name.clone()).collect();
    let functions_lower: HashSet<String> =
        functions.iter().map(|f| f.to_ascii_lowercase()).collect();
    let mut functions_by_file: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for func in &code.functions {
        functions_by_file
            .entry(func.file_path.to_ascii_lowercase())
            .or_default()
            .push(func.name.clone());
    }
    let env_vars: HashSet<String> = code.env_vars.iter().map(|e| e.name.clone()).collect();
    let env_vars_lower: HashSet<String> = env_vars.iter().map(|e| e.to_ascii_lowercase()).collect();

    for resource in infra.resources.values() {
        let mut matched = false;
        for (k, v) in &resource.properties {
            let v_lower = v.to_ascii_lowercase();
            let k_lower = k.to_ascii_lowercase();
            // File match: property contains a code file path or filename
            if files.iter().any(|f| v.contains(f))
                || file_names.iter().any(|name| v_lower.contains(name))
            {
                for f in files
                    .iter()
                    .filter(|f| v.contains(*f) || v_lower.contains(&f.to_ascii_lowercase()))
                {
                    report.matches.push(CorrelationMatch {
                        code_node: f.clone(),
                        infra_node: resource.id.clone(),
                        reason: format!("property references file {f}"),
                    });
                    matched = true;
                }
            }
            // Function/handler match by name (supports handler strings like module.func)
            if functions_lower.iter().any(|func| v_lower.contains(func)) {
                for func in functions.iter().filter(|func| {
                    let lower = func.to_ascii_lowercase();
                    v_lower.contains(&lower)
                        || v_lower
                            .split(|c| c == ':' || c == '/' || c == '.')
                            .any(|part| part == lower)
                }) {
                    report.matches.push(CorrelationMatch {
                        code_node: func.clone(),
                        infra_node: resource.id.clone(),
                        reason: format!("property references handler/function {func}"),
                    });
                    matched = true;
                }
            }
            // Handler match combining file + function (e.g., "src/app.lambda_handler")
            if v_lower.contains('/') && v_lower.contains('.') {
                let parts: Vec<&str> = v_lower.split('.').collect();
                if let Some((path_part, func_part)) = parts.split_last().and_then(|(last, rest)| {
                    let func = *last;
                    let path = rest.join(".");
                    Some((path, func))
                }) {
                    let path_clean = path_part.replace(':', "");
                    if file_names.iter().any(|n| path_clean.contains(n))
                        || files
                            .iter()
                            .any(|f| f.to_ascii_lowercase().contains(&path_clean))
                    {
                        if functions_lower.contains(func_part) {
                            if let Some(func_original) = functions
                                .iter()
                                .find(|f| f.to_ascii_lowercase() == func_part)
                            {
                                let qualified =
                                    format!("{}::{}", path_part.replace('\\', "/"), func_original);
                                report.matches.push(CorrelationMatch {
                                    code_node: qualified,
                                    infra_node: resource.id.clone(),
                                    reason: format!("handler maps to {path_part}.{func_part}"),
                                });
                                matched = true;
                            }
                        }
                    }
                    // Try by file association
                    for (file, funcs) in &functions_by_file {
                        if file.contains(&path_clean) {
                            for f in funcs {
                                report.matches.push(CorrelationMatch {
                                    code_node: format!("{}::{}", path_part, f),
                                    infra_node: resource.id.clone(),
                                    reason: format!("handler references file {path_part}"),
                                });
                                matched = true;
                            }
                        }
                    }
                }
            }
            // Env var match by value
            if env_vars_lower.iter().any(|env| v_lower.contains(env)) {
                for env in env_vars.iter().filter(|env| {
                    let lower = env.to_ascii_lowercase();
                    v_lower.contains(&lower)
                        || v_lower
                            .split(|c| c == ':' || c == '/' || c == '.')
                            .any(|part| part == lower)
                }) {
                    report.matches.push(CorrelationMatch {
                        code_node: env.clone(),
                        infra_node: resource.id.clone(),
                        reason: format!("property references env var {env}"),
                    });
                    matched = true;
                }
            }
            // Env var match by key
            if env_vars_lower.contains(&k_lower) {
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

    if report.matches.is_empty() && !infra.resources.is_empty() {
        report
            .warnings
            .push("Infra resources detected but no code correlations found".to_string());
    }

    report
}
