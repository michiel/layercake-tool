mod javascript;
mod python;

use anyhow::Result;
use ignore::WalkBuilder;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::warn;

pub use javascript::JavascriptAnalyzer;
pub use python::PythonAnalyzer;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Import {
    pub module: String,
    pub file_path: String,
    pub line_number: usize,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub file_path: String,
    pub line_number: usize,
    pub args: Vec<(String, String)>,
    pub return_type: String,
    pub complexity: usize,
    pub calls: Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DataFlow {
    pub source: String,
    pub sink: String,
    pub variable: Option<String>,
    pub file_path: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CallEdge {
    pub caller: String,
    pub callee: String,
    pub file_path: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EntryPoint {
    pub file_path: String,
    pub line_number: usize,
    pub condition: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub imports: Vec<Import>,
    pub functions: Vec<FunctionInfo>,
    pub data_flows: Vec<DataFlow>,
    pub call_edges: Vec<CallEdge>,
    pub entry_points: Vec<EntryPoint>,
    pub exits: Vec<EntryPoint>,
    pub external_calls: Vec<ExternalCall>,
    pub env_vars: Vec<EnvVarUsage>,
    pub files: Vec<String>,
    pub directories: Vec<String>,
    pub infra: Option<crate::infra::InfrastructureGraph>,
    pub infra_correlation: Option<crate::infra::CorrelationReport>,
}

impl AnalysisResult {
    pub fn merge(mut self, other: AnalysisResult) -> AnalysisResult {
        self.imports.extend(other.imports);
        self.functions.extend(other.functions);
        self.data_flows.extend(other.data_flows);
        self.call_edges.extend(other.call_edges);
        self.entry_points.extend(other.entry_points);
        self.exits.extend(other.exits);
        self.external_calls.extend(other.external_calls);
        self.env_vars.extend(other.env_vars);
        self.files.extend(other.files);
        self.directories.extend(other.directories);
        if self.infra.is_none() {
            self.infra = other.infra;
        }
        if self.infra_correlation.is_none() {
            self.infra_correlation = other.infra_correlation;
        }
        self
    }

    pub fn sort_deterministic(&mut self) {
        self.imports.sort_by(|a, b| {
            (a.file_path.as_str(), a.module.as_str(), a.line_number).cmp(&(
                b.file_path.as_str(),
                b.module.as_str(),
                b.line_number,
            ))
        });

        self.functions.sort_by(|a, b| {
            (
                a.file_path.as_str(),
                a.line_number,
                a.name.as_str(),
                a.complexity,
            )
                .cmp(&(
                    b.file_path.as_str(),
                    b.line_number,
                    b.name.as_str(),
                    b.complexity,
                ))
        });

        self.data_flows.sort_by(|a, b| {
            (
                a.file_path.as_str(),
                a.source.as_str(),
                a.sink.as_str(),
                a.variable.as_deref().unwrap_or(""),
            )
                .cmp(&(
                    b.file_path.as_str(),
                    b.source.as_str(),
                    b.sink.as_str(),
                    b.variable.as_deref().unwrap_or(""),
                ))
        });

        self.call_edges.sort_by(|a, b| {
            (a.file_path.as_str(), a.caller.as_str(), a.callee.as_str()).cmp(&(
                b.file_path.as_str(),
                b.caller.as_str(),
                b.callee.as_str(),
            ))
        });

        self.entry_points.sort_by(|a, b| {
            (a.file_path.as_str(), a.line_number, a.condition.as_str()).cmp(&(
                b.file_path.as_str(),
                b.line_number,
                b.condition.as_str(),
            ))
        });

        self.env_vars.sort_by(|a, b| {
            (
                a.file_path.as_str(),
                a.line_number,
                a.name.as_str(),
                a.kind.as_str(),
            )
                .cmp(&(
                    b.file_path.as_str(),
                    b.line_number,
                    b.name.as_str(),
                    b.kind.as_str(),
                ))
        });

        self.files.sort();
        self.files.dedup();
        self.directories.sort();
        self.directories.dedup();
    }
}

pub trait GraphConvertible<G> {
    fn to_graph(&self, annotation: Option<String>) -> G;
}

pub trait Analyzer: Send + Sync {
    fn supports(&self, path: &Path) -> bool;
    fn analyze(&self, path: &Path) -> Result<AnalysisResult>;
    fn language(&self) -> &'static str;
}

pub struct AnalyzerRegistry {
    analyzers: Vec<Box<dyn Analyzer>>,
}

impl AnalyzerRegistry {
    pub fn new(analyzers: Vec<Box<dyn Analyzer>>) -> Self {
        Self { analyzers }
    }

    pub fn find_for_path(&self, path: &Path) -> Option<&dyn Analyzer> {
        self.analyzers
            .iter()
            .find(|analyzer| analyzer.supports(path))
            .map(|analyzer| analyzer.as_ref())
    }
}

impl Default for AnalyzerRegistry {
    fn default() -> Self {
        Self {
            analyzers: vec![
                Box::new(PythonAnalyzer::default()),
                Box::new(JavascriptAnalyzer::default()),
            ],
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AnalysisRun {
    pub result: AnalysisResult,
    pub files_scanned: usize,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EnvVarUsage {
    pub name: String,
    pub file_path: String,
    pub line_number: usize,
    pub kind: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ExternalCall {
    pub target: String,
    pub method: Option<String>,
    pub path: Option<String>,
    pub file_path: String,
    pub line_number: usize,
}

pub fn analyze_path(path: &Path) -> Result<AnalysisRun> {
    let registry = AnalyzerRegistry::default();
    let root = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let walker = WalkBuilder::new(path)
        .hidden(false)
        .parents(true)
        .ignore(true)
        .git_ignore(true)
        .git_exclude(true)
        .git_global(true)
        .build();

    let mut files = Vec::new();
    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                warn!("Skipping entry: {err}");
                continue;
            }
        };

        if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            files.push(entry.into_path());
        }
    }

    let supported_files: Vec<PathBuf> = files
        .into_iter()
        .filter(|path| registry.find_for_path(path).is_some())
        .collect();

    #[derive(Default, Clone)]
    struct PartialResult {
        result: AnalysisResult,
    }

    let aggregated = supported_files
        .par_iter()
        .filter_map(|file_path| {
            let analyzer = registry.find_for_path(file_path)?;
            let relative = file_path
                .strip_prefix(&root)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string();
            let dirs = collect_directories(&relative);

            match analyzer.analyze(file_path) {
                Ok(mut result) => {
                    normalize_paths(&mut result, &relative);
                    result.files.push(relative.clone());
                    result.directories.extend(dirs);
                    Some(PartialResult { result })
                }
                Err(err) => {
                    warn!(
                        "Failed to analyze {:?} with {}: {}",
                        file_path,
                        analyzer.language(),
                        err
                    );
                    None
                }
            }
        })
        .reduce(PartialResult::default, |left, right| PartialResult {
            result: left.result.merge(right.result),
        })
        .result;

    let mut result = aggregated;
    result.sort_deterministic();

    // Default infra parsing & correlation
    match crate::infra::analyze_infra(&root) {
        Ok(infra_graph) => {
            let correlation = crate::infra::correlate_code_infra(&result, &infra_graph);
            result.infra = Some(infra_graph);
            result.infra_correlation = Some(correlation);
        }
        Err(err) => {
            warn!("Infra analysis failed: {err}");
        }
    }

    Ok(AnalysisRun {
        result,
        files_scanned: supported_files.len(),
    })
}

fn normalize_paths(result: &mut AnalysisResult, relative: &str) {
    for import in &mut result.imports {
        import.file_path = relative.to_string();
    }
    for function in &mut result.functions {
        function.file_path = relative.to_string();
    }
    for flow in &mut result.data_flows {
        flow.file_path = relative.to_string();
    }
    for entry in &mut result.entry_points {
        entry.file_path = relative.to_string();
    }
}

fn collect_directories(file: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let path = Path::new(file);
    if let Some(parent) = path.parent() {
        let mut current = PathBuf::new();
        for part in parent.iter() {
            current.push(part);
            if let Some(rel) = current.to_str() {
                parts.push(rel.to_string());
            }
        }
    }
    parts
}
