mod javascript;
mod python;

use anyhow::Result;
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use tracing::warn;

pub use javascript::JavascriptAnalyzer;
pub use python::PythonAnalyzer;

#[derive(Debug, Default, Clone)]
pub struct Import {
    pub module: String,
    pub file_path: String,
    pub line_number: usize,
}

#[derive(Debug, Default, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub file_path: String,
    pub line_number: usize,
    pub args: Vec<(String, String)>,
    pub return_type: String,
    pub complexity: usize,
    pub calls: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct DataFlow {
    pub source: String,
    pub sink: String,
    pub variable: Option<String>,
    pub file_path: String,
}

#[derive(Debug, Default, Clone)]
pub struct EntryPoint {
    pub file_path: String,
    pub line_number: usize,
    pub condition: String,
}

#[derive(Debug, Default, Clone)]
pub struct AnalysisResult {
    pub imports: Vec<Import>,
    pub functions: Vec<FunctionInfo>,
    pub data_flows: Vec<DataFlow>,
    pub entry_points: Vec<EntryPoint>,
}

impl AnalysisResult {
    pub fn merge(mut self, other: AnalysisResult) -> AnalysisResult {
        self.imports.extend(other.imports);
        self.functions.extend(other.functions);
        self.data_flows.extend(other.data_flows);
        self.entry_points.extend(other.entry_points);
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

        self.entry_points.sort_by(|a, b| {
            (a.file_path.as_str(), a.line_number, a.condition.as_str()).cmp(&(
                b.file_path.as_str(),
                b.line_number,
                b.condition.as_str(),
            ))
        });
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

#[derive(Debug, Default, Clone)]
pub struct AnalysisRun {
    pub result: AnalysisResult,
    pub files_scanned: usize,
}

pub fn analyze_path(path: &Path) -> Result<AnalysisRun> {
    let registry = AnalyzerRegistry::default();
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

    let aggregated = supported_files
        .par_iter()
        .filter_map(|file_path| {
            let analyzer = registry.find_for_path(file_path)?;
            match analyzer.analyze(file_path) {
                Ok(result) => Some(result),
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
        .reduce(AnalysisResult::default, |left, right| left.merge(right));

    let mut result = aggregated;
    result.sort_deterministic();

    Ok(AnalysisRun {
        result,
        files_scanned: supported_files.len(),
    })
}
