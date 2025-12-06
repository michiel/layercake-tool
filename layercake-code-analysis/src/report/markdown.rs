use crate::analyzer::AnalysisResult;
use crate::infra::{CorrelationReport, InfrastructureGraph};
use crate::report::ReportMetadata;
use anyhow::Result;
use csv::WriterBuilder;
use std::path::Path;
use tokei::{Config, Languages};

#[derive(Default)]
pub struct MarkdownReporter;

impl MarkdownReporter {
    pub fn render(&self, result: &AnalysisResult, metadata: &ReportMetadata) -> Result<String> {
        self.render_with_infra(result, metadata, None, None)
    }

    pub fn render_with_infra(
        &self,
        result: &AnalysisResult,
        metadata: &ReportMetadata,
        infra: Option<&InfrastructureGraph>,
        correlation: Option<&CorrelationReport>,
    ) -> Result<String> {
        let mut output = String::new();
        output.push_str("# Code Analysis Report\n");
        output.push_str(&format!("{}\n\n", metadata.summary(result)));

        if let Some(stats) = codebase_stats(&metadata.root_path) {
            output.push_str("## Codebase stats\n");
            output.push_str(&stats);
            output.push_str("\n\n");
        }

        if let Some(infra_graph) = infra {
            output.push_str("## Infrastructure summary\n");
            let counts = infra_counts(infra_graph);
            output.push_str(&counts);
            output.push('\n');
            if !infra_graph.diagnostics.is_empty() {
                output.push_str("\n### Infra diagnostics\n");
                for diag in &infra_graph.diagnostics {
                    output.push_str(&format!("- {diag}\n"));
                }
            }
            output.push_str("\n");
        }

        if let Some(corr) = correlation {
            output.push_str("## Correlation summary\n");
            output.push_str(&format!("- Matches: {}\n", corr.matches.len()));
            output.push_str(&format!("- Unresolved: {}\n", corr.unresolved.len()));
            if !corr.matches.is_empty() {
                output.push_str("\n### Matches\n");
                for m in &corr.matches {
                    output.push_str(&format!(
                        "- Code `{}` ↔ Infra `{}` ({})\n",
                        m.code_node, m.infra_node, m.reason
                    ));
                }
            }
            if !corr.unresolved.is_empty() {
                output.push_str("\n### Unresolved\n");
                for u in &corr.unresolved {
                    output.push_str(&format!("- {u}\n"));
                }
            }
            output.push('\n');
        }

        for (name, headers, rows) in datasets(result) {
            output.push_str(&format!("## {}\n", name));
            let csv = to_csv(&headers, &rows)?;
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

pub fn strip_csv_blocks(markdown: &str) -> String {
    let mut cleaned = String::new();
    let mut in_csv = false;
    for line in markdown.lines() {
        if line.trim_start().starts_with("```CSV") {
            in_csv = true;
            continue;
        }
        if in_csv && line.trim_start().starts_with("```") {
            in_csv = false;
            continue;
        }
        if !in_csv {
            cleaned.push_str(line);
            cleaned.push('\n');
        }
    }
    cleaned.trim_end().to_string()
}

fn datasets(result: &AnalysisResult) -> Vec<(&'static str, Vec<String>, Vec<Vec<String>>)> {
    let mut imports_rows = Vec::with_capacity(result.imports.len());
    for import in &result.imports {
        imports_rows.push(vec![
            import.file_path.clone(),
            import.module.clone(),
            import.line_number.to_string(),
        ]);
    }

    let mut function_rows = Vec::with_capacity(result.functions.len());
    for function in &result.functions {
        let args = function
            .args
            .iter()
            .map(|(name, ty)| format!("{name}:{ty}"))
            .collect::<Vec<_>>()
            .join(";");
        let calls = function.calls.join(";");
        function_rows.push(vec![
            function.file_path.clone(),
            function.name.clone(),
            function.line_number.to_string(),
            function.complexity.to_string(),
            function.return_type.clone(),
            args,
            calls,
        ]);
    }

    let mut data_flow_rows = Vec::with_capacity(result.data_flows.len());
    for flow in &result.data_flows {
        data_flow_rows.push(vec![
            flow.source.clone(),
            flow.sink.clone(),
            flow.variable.clone().unwrap_or_default(),
            flow.file_path.clone(),
        ]);
    }

    let mut entry_rows = Vec::with_capacity(result.entry_points.len());
    for entry in &result.entry_points {
        entry_rows.push(vec![
            entry.file_path.clone(),
            entry.line_number.to_string(),
            entry.condition.clone(),
        ]);
    }

    let mut env_rows = Vec::with_capacity(result.env_vars.len());
    for env in &result.env_vars {
        env_rows.push(vec![
            env.file_path.clone(),
            env.name.clone(),
            env.kind.clone(),
            env.line_number.to_string(),
        ]);
    }

    vec![
        (
            "imports",
            vec!["file".into(), "module".into(), "line".into()],
            imports_rows,
        ),
        (
            "functions",
            vec![
                "file".into(),
                "name".into(),
                "line".into(),
                "complexity".into(),
                "return_type".into(),
                "args".into(),
                "calls".into(),
            ],
            function_rows,
        ),
        (
            "data_flows",
            vec![
                "source".into(),
                "sink".into(),
                "variable".into(),
                "file".into(),
            ],
            data_flow_rows,
        ),
        (
            "entry_points",
            vec!["file".into(), "line".into(), "condition".into()],
            entry_rows,
        ),
        (
            "env_vars",
            vec!["file".into(), "name".into(), "kind".into(), "line".into()],
            env_rows,
        ),
    ]
}

fn to_csv(headers: &[String], rows: &[Vec<String>]) -> Result<String> {
    let mut buffer = Vec::new();
    {
        let mut writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(&mut buffer);
        writer.write_record(headers)?;
        for row in rows {
            writer.write_record(row)?;
        }
        writer.flush()?;
    }
    Ok(String::from_utf8(buffer)?)
}

fn codebase_stats(root: &Path) -> Option<String> {
    let mut languages = Languages::new();
    let config = Config::default();
    languages.get_statistics(&[root.to_path_buf()], &[], &config);

    let total = languages.total();
    let total_lines = total.lines();
    if total_lines == 0 {
        return None;
    }

    let mut by_lang: Vec<(String, usize)> = languages
        .iter()
        .map(|(lang, stats)| (lang.to_string(), stats.lines()))
        .collect();
    by_lang.sort_by(|a, b| b.1.cmp(&a.1));
    let top = by_lang.into_iter().take(5).collect::<Vec<_>>();

    let mut out = String::new();
    out.push_str(&format!("- Total code lines: {total_lines}\n",));
    if !top.is_empty() {
        out.push_str("- Top languages by code lines: ");
        let parts: Vec<String> = top
            .into_iter()
            .map(|(name, code)| format!("{name} ({code})"))
            .collect();
        out.push_str(&parts.join(", "));
        out.push('\n');
    }
    Some(out.trim_end().to_string())
}

fn infra_counts(graph: &InfrastructureGraph) -> String {
    let mut counts = std::collections::HashMap::<String, usize>::new();
    for node in graph.resources.values() {
        let kind = format!("{:?}", node.resource_type);
        *counts.entry(kind).or_insert(0) += 1;
    }
    if counts.is_empty() {
        return "- No infrastructure resources detected.\n".to_string();
    }
    let mut parts: Vec<String> = counts
        .into_iter()
        .map(|(kind, count)| format!("{kind}: {count}"))
        .collect();
    parts.sort();
    format!(
        "- Resource counts: {}\n- Edges: {}\n",
        parts.join(", "),
        graph.edges.len()
    )
}
