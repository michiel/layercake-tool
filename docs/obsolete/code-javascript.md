# JavaScript/TypeScript Code Analysis Options

## Parser/AST Options

- **tree-sitter-javascript / tree-sitter-typescript**  
  - Pros: tolerant of syntax errors; preserves comments/whitespace; great for editor-like tooling and incremental parsing.  
  - Cons: CST rather than AST; more verbose; type info limited; additional binding/build step.
- **swc (via `swc_ecma_parser`)**  
  - Pros: fast Rust-native parser; outputs ESTree-like AST; supports TS, JSX, decorators; good error reporting.  
  - Cons: less tolerant than tree-sitter for broken code; limited comment retention unless configured; API surface is large.
- **oxc**  
  - Pros: Rust implementation of ESTree/TypeScript with strong performance and lint-oriented APIs; good semantic helpers.  
  - Cons: ecosystem less mature than swc; APIs still evolving; fewer examples.
- **Rslint parser (`rslint_parser`)**  
  - Pros: Good error recovery; ESTree-ish AST; lighter dependency footprint than swc.  
  - Cons: Project is less active; TS/JSX coverage lags behind swc/oxc.

## Recommended Stack

- **Parser:** `swc_ecma_parser` with `Syntax::Typescript { tsx: true, decorators: true }` to cover JS/TS/JSX.  
- **AST Traversal:** `swc_ecma_visit` (`Visit`/`VisitMut`) for walking and collecting metrics.  
- **Comments:** Enable `parse_comments` and thread `SingleThreadedComments` through if we need import doc extraction.  
- **Project Walk:** Reuse `ignore`/`.gitignore` handling from Python analyzer; filter on `js/ts/jsx/tsx/mjs/cjs`.  
- **Parallelism:** Same rayon map-reduce pattern per file.  
- **Determinism:** Sort results before reporting (as with Python).

## Tradeoffs

- **tree-sitter vs swc:** tree-sitter wins on error tolerance and comment fidelity; swc wins on semantic richness (bindings, TypeScript) and speed.  
- **oxc vs swc:** oxc has emerging semantic helpers but less stability; swc has broader adoption and examples.  
- **Error recovery:** swc recovers reasonably but fails on severely broken code; tree-sitter would be needed for editor/live mode.  
- **JSX/TS support:** swc/oxc both support; rslint coverage is partial.  
- **Maintenance:** swc is actively maintained and kept in sync with modern TC39 proposals; reduces maintenance burden.

## Implementation Plan (JS/TS Analyzer)

1) **Crate deps**  
   - Add to `layercake-code-analysis`:  
     - `swc_common = "0.141"` (SourceMaps, comments)  
     - `swc_ecma_parser = "0.141"`  
     - `swc_ecma_visit = "0.141"`  
   - Feature-guard if size is a concern.
2) **File discovery**  
   - Extend analyzer registry to accept JS/TS extensions (`js`, `jsx`, `ts`, `tsx`, `mjs`, `cjs`).  
   - Reuse `ignore` walker.
3) **Parsing**  
   - Configure `Syntax::Typescript { tsx: true, decorators: true, dts: false }`, `EsConfig` for import assertions/top-level await.  
   - Enable `parse_comments: true`.  
   - Map parser errors to warnings and continue.
4) **Traversal** (mirror Python metrics)  
   - Collect imports (`ImportDecl`, `NamedExport` re-exports).  
   - Functions/methods (`FnDecl`, `ClassMethod`, `MethodProp`, `ArrowExpr` where named).  
   - Compute cyclomatic complexity: `IfStmt`, `For/While/DoWhile`, `SwitchCase` (case count-1), logical `&&/||`, `CatchClause`.  
   - Calls: record callee string; capture data-flow via simple alias map (`var_sources`), handling `let/const/var` and destructuring identifiers.  
   - Entry points: `export default` functions? For parity, detect package-level `main` via `if (require.main === module)` and top-level `new App().start()` heuristic (documented).
5) **Scopes and qualification**  
   - Track class stack and function stack for fully-qualified names; scope stack per function/block for variable sources; reset at function boundaries.  
   - Handle `this.method` qualifications in calls.  
   - For destructuring, map each identifier to the source call name when RHS is a call.
6) **Type info**  
   - From TypeScript AST: extract parameter/return type annotations (`TsTypeAnn`). For JS, fallback to `"any"`.  
   - Keep as strings using `swc_ecma_codegen::text_writer::WriteJs` or manual printers for types.
7) **Data model & output**  
   - Reuse existing `AnalysisResult` structs; ensure deterministic sorting.  
   - Add JS analyzer to registry; leave existing JS stub removed/replaced.
8) **Tests**  
   - Integration tests for:  
     - Basic TS file with imports, class methods, JSX function.  
     - Complexity with `switch` and logical ops.  
     - Data-flow aliasing: `const x = get(); const y = x; sink(y);`.  
     - Entry-point detection for `require.main === module`.
9) **Docs**  
   - Update `code-plan.md` and README to mark JS implemented; note crate choices and error-tolerance limits.
