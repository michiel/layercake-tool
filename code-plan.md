# Code Analysis Feature - Design and Implementation Plan

## Overview

Implement a code analysis capability for the Layercake tool that parses codebases and generates rich datasets in Layercake format. The initial implementation will focus on Python, with extensibility for other languages.

## Goals

- Create a new `layercake-code-analysis` crate in the workspace
- Implement Python static analysis using RustPython AST parser
- Extract structural, complexity, and data-flow metrics
- Generate markdown reports with embedded CSV datasets
- Provide CLI interface: `layercake code-analysis report PATH -o report.md` (alias: `ca`)

## Architecture

### Component Structure

```
layercake-code-analysis/
├── src/
│   ├── lib.rs              # Public API and common types
│   ├── cli.rs              # CLI command implementation
│   ├── analyzer/
│   │   ├── mod.rs          # Analyzer trait definition
│   │   ├── python.rs       # Python-specific analyzer
│   │   └── javascript.rs   # Placeholder for future JS support
│   ├── metrics/
│   │   ├── mod.rs          # Metric types and structures
│   │   ├── complexity.rs   # Cyclomatic complexity calculation
│   │   └── dataflow.rs     # Data flow tracking
│   └── report/
│       ├── mod.rs          # Report generation
│       └── markdown.rs     # Markdown formatter with CSV output
└── tests/
    └── integration_tests.rs
```

### Key Data Structures

```rust
// Core analysis result
#[derive(Debug, Default, Clone)]
pub struct AnalysisResult {
    pub imports: Vec<Import>,
    pub functions: Vec<FunctionInfo>,
    pub data_flows: Vec<DataFlow>,
    pub entry_points: Vec<EntryPoint>,
}

// Import metadata
#[derive(Debug, Clone)]
pub struct Import {
    pub module: String,
    pub file_path: String,
    pub line_number: usize,
}

// Function metadata
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub file_path: String,
    pub line_number: usize,
    pub args: Vec<(String, String)>,  // (name, type_hint)
    pub return_type: String,
    pub complexity: usize,
    pub calls: Vec<String>,
}

// Data flow tracking
#[derive(Debug, Clone)]
pub struct DataFlow {
    pub source: String,
    pub sink: String,
    pub variable: Option<String>,
    pub file_path: String,
}
```

## Implementation Stages

### Stage 1: Project Setup and Core Infrastructure
**Goal**: Create the crate structure and basic CLI integration
**Success Criteria**:
- Crate compiles and integrates with main CLI
- `layercake ca --help` shows command documentation
- Basic argument parsing works (PATH, -o flag)

**Tasks**:
1. Create `layercake-code-analysis` crate in workspace
2. Add dependencies to `Cargo.toml`:
   - `rustpython-parser = "0.3.0"`
   - `rustpython-ast = { version = "0.3.0", features = ["visitor"] }`
   - `walkdir = "2.4"`
   - `ignore = "0.4"` (for .gitignore support)
3. Define `Analyzer` trait in `src/analyzer/mod.rs`
4. Implement conversion traits/helpers to transform analysis structs into Layercake dataset records for downstream persistence
5. Implement CLI command structure in `src/cli.rs`
6. Integrate with main CLI using clap subcommand

**Tests**:
- CLI argument parsing
- Help text generation
- Invalid path handling
- Trait round-trip: analysis structs convert to Layercake dataset formats without loss

**Status**: Completed

---

### Stage 2: Python AST Parser and Basic Visitor
**Goal**: Parse Python files and traverse AST
**Success Criteria**:
- Successfully parse valid Python files
- Gracefully handle syntax errors
- Basic visitor pattern implemented and tested

**Tasks**:
1. Implement file discovery with `.gitignore` support
2. Create `PythonAnalyzer` struct with visitor implementation
3. Implement basic visitor methods:
   - `visit_stmt_import` / `visit_stmt_import_from`
   - `visit_stmt_function_def`
   - `visit_stmt_class_def`
4. Add error handling for parse failures
5. Create helper method `get_expr_name` for expression resolution
6. Detect entry points (e.g., `if __name__ == "__main__"`) and record file/line metadata

**Tests**:
- Parse simple Python file with imports and functions
- Handle syntax errors without crashing
- Correctly identify nested functions
- Process multiple files in directory
- Entry point detection captured with correct line numbers

**Status**: Completed

---

### Stage 3: Scope Management and Variable Tracking
**Goal**: Implement scope stack for accurate variable resolution
**Success Criteria**:
- Variables correctly resolved within nested scopes
- Variable aliasing tracked through assignments
- Scope isolation works for methods in different classes

**Tasks**:
1. Implement scope stack (`Vec<HashMap<String, String>>`)
2. Add `enter_scope()` and `exit_scope()` methods
3. Implement `define_variable()` and `resolve_variable()` methods
4. Update `visit_stmt_function_def` to manage scope lifecycle
5. Implement `visit_stmt_assign` with:
   - Direct assignment tracking (`x = func()`)
   - Variable aliasing (`y = x`)
6. Track class context with class stack
7. Ensure fully-qualified function/method names (including class and nested contexts) are used in variable sources and call records to avoid collisions across scopes

**Tests**:
- Variable resolution in nested functions
- Aliasing chain: `a = func(); b = a; c = b; call(c)`
- Same variable name in different functions
- Method resolution with class context
- Aliasing and call tracking for methods on different classes with same method name

**Status**: Completed

---

### Stage 4: Data Flow Analysis
**Goal**: Track data flow from source functions to sink functions
**Success Criteria**:
- Correctly identify source-to-sink flows through variables
- Handle multi-hop aliasing
- Capture flows across function boundaries

**Tasks**:
1. Implement `visit_expr_call` to check argument variables
2. Build data flow records when variable has known source
3. Track flows through assignment chains
4. Store flows with file location context
5. Preserve fully-qualified caller/callee names in flow records, including class context

**Tests**:
- Simple flow: `x = source(); sink(x)`
- Multi-hop: `x = source(); y = x; sink(y)`
- Multiple flows from same source
- Flows within different functions
- Flows across methods with same short name in different classes

**Status**: Completed

---

### Stage 5: Type Hints and Complexity Metrics
**Goal**: Extract type information and calculate cyclomatic complexity
**Success Criteria**:
- Type hints correctly parsed for all forms (List[int], Dict[str, Any], etc.)
- Cyclomatic complexity matches McCabe's formula
- Complexity correctly isolated per function

**Tasks**:
1. Enhance `get_expr_name` to handle:
   - `Expr::Subscript` (generic types)
   - `Expr::Tuple` (tuple types)
   - `Expr::Constant` (string literals in types)
2. Parse function argument type hints from `annotation` field
3. Parse return type hints from `returns` field
4. Implement complexity stack (`Vec<usize>`)
5. Add complexity increment logic in:
   - `visit_stmt_if`
   - `visit_stmt_for`
   - `visit_stmt_while`
   - `visit_excepthandler`
   - `visit_expr_bool_op` (for and/or chains)
   - Explicitly handle or ignore `match`/`case` with a documented decision and test fixture
6. Capture complexity score on scope exit

**Tests**:
- Type parsing: `def func(x: List[Dict[str, int]]) -> Optional[str]`
- Simple function: complexity = 1
- Function with if/for/while: verify counts
- Nested functions don't affect parent complexity
- Boolean chains: `a and b or c` complexity calculation
- `match`/`case` behavior documented and tested for stability

**Status**: Completed

---

### Stage 6: Report Generation and CSV Export
**Goal**: Generate markdown report with embedded CSV datasets
**Success Criteria**:
- Markdown report includes summary statistics
- CSV datasets embedded in code fences
- Output to file or STDOUT

**Tasks**:
1. Design CSV schema for each dataset:
   - Functions: `file,name,line,complexity,return_type,args`
   - Data Flows: `source,sink,variable,file`
   - Imports: `file,module,line`
   - Entry Points: `file,line,condition`
2. Implement `MarkdownReporter` struct
3. Generate summary section with counts and statistics
4. Format CSV datasets with proper escaping
5. Handle `-o` flag for file output vs STDOUT
6. Add project metadata section (path, file count, timestamp)
7. Ensure deterministic ordering (sort) before rendering to keep reports and tests stable

**Tests**:
- Generate report for sample Python project
- Verify CSV format validity
- Test STDOUT vs file output
- Handle special characters in CSV fields
- Deterministic ordering verified across runs

**Status**: Completed

---

### Stage 7: Parallel Processing and Performance
**Goal**: Process large codebases efficiently using parallelism
**Success Criteria**:
- Files processed in parallel using rayon
- Results correctly merged without data loss
- Performance improvement on multi-core systems

**Tasks**:
1. Add `rayon = "1.7"` dependency
2. Refactor file processing to use `par_iter()`
3. Implement `AnalysisResult::merge()` method
4. Ensure thread-safe error handling
5. Add progress indicator for large codebases

**Tests**:
- Process 100+ files and verify result completeness
- Compare sequential vs parallel results for consistency
- Benchmark performance improvement

**Status**: Completed

---

### Stage 8: JavaScript Placeholder and Extensibility
**Goal**: Create framework for future language support
**Success Criteria**:
- `Analyzer` trait allows multiple language implementations
- JavaScript analyzer stub with `unimplemented!()` macros
- Clear documentation for adding new languages

**Tasks**:
1. Document `Analyzer` trait contract
2. Create `src/analyzer/javascript.rs` with skeleton
3. Add language detection logic (by file extension)
4. Update CLI to handle multiple language types
5. Document extension process in README

**Tests**:
- Verify Python analyzer still works
- JavaScript files trigger appropriate unimplemented message

**Status**: Completed

---

## Technical Considerations

### Error Handling
- Use `anyhow::Result` for error propagation
- Gracefully handle parse errors (log and continue to next file)
- Provide clear error messages with file context

### Testing Strategy
- Unit tests for each visitor method
- Integration tests with sample Python projects
- Regression tests for edge cases (deeply nested code, complex types)

### Performance
- Use parallel processing for multi-file analysis
- Minimize allocations in hot paths
- Consider caching parsed ASTs for watch mode integration

### Cross-Platform Compatibility
- Use `std::path::PathBuf` for all file paths
- Test on Linux, macOS, and Windows
- Ensure CSV line endings are consistent

## Future Extensions

1. **Control Flow Graph (CFG)**: Build directed graph for dead code detection
2. **Call Graph Visualisation**: Generate Mermaid/DOT diagrams
3. **Incremental Analysis**: Cache results and only re-analyse changed files
4. **Custom Metrics**: Allow user-defined metrics via plugin system
5. **Python 3.12+ Support**: Upgrade parser for pattern matching support
6. **JavaScript/TypeScript**: Full implementation using swc or tree-sitter
7. **Integration with Layercake Pipeline**: Use analysis results as dataset source

## Dependencies

```toml
[dependencies]
rustpython-parser = "0.3.0"
rustpython-ast = { version = "0.3.0", features = ["visitor"] }
walkdir = "2.4"
ignore = "0.4"
rayon = "1.7"
anyhow = "1.0"
clap = { version = "4.0", features = ["derive"] }
csv = "1.3"
```

## Definition of Done

- [ ] All 8 stages completed with tests passing
- [ ] CLI command `layercake ca report <path> -o report.md` works end-to-end
- [ ] Documentation added to docs/ directory
- [ ] Example reports generated for sample projects
- [ ] Code follows project style guidelines
- [ ] All tests pass on Linux, macOS, and Windows
- [ ] Performance benchmarks documented

## Notes

- This design is based on the proof-of-concept code in `docs/code-analysis-background.md`
- Implementation should be incremental with each stage fully tested before proceeding
- Consider using `tracing` for debug logging during development
- The design prioritises correctness and maintainability over premature optimisation
