pub mod analyzer;
pub mod cli;
pub mod infra;
pub mod report;

pub use analyzer::{
    analyze_path, AnalysisResult, AnalysisRun, AnalyzerRegistry, DataFlow, EntryPoint,
    FunctionInfo, Import,
};
pub use cli::{CodeAnalysisArgs, CodeAnalysisCommand};
pub use report::{markdown::MarkdownReporter, ReportMetadata};
