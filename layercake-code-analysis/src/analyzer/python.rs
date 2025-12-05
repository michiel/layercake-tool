use super::CallEdge;
use super::{AnalysisResult, Analyzer, DataFlow, EntryPoint, FunctionInfo, Import};
use anyhow::{Context, Result};
use rustpython_ast::{self as ast, Visitor};
use rustpython_parser::{parse, Mode};
use std::collections::HashMap;
use std::path::Path;

#[derive(Default)]
pub struct PythonAnalyzer;

impl Analyzer for PythonAnalyzer {
    fn supports(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("py"))
            .unwrap_or(false)
    }

    fn analyze(&self, path: &Path) -> Result<AnalysisResult> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read python file {:?}", path))?;
        let line_index = build_line_index(&content);
        let module = parse(&content, Mode::Module, path.to_string_lossy().as_ref())
            .with_context(|| format!("Failed to parse python file {:?}", path))?;

        let mut visitor = PythonVisitor::new(path, &line_index);
        match module {
            ast::Mod::Module(module) => {
                for stmt in module.body {
                    visitor.visit_stmt(stmt);
                }
            }
            ast::Mod::Interactive(module) => {
                for stmt in module.body {
                    visitor.visit_stmt(stmt);
                }
            }
            ast::Mod::Expression(expr) => {
                visitor.visit_expr(*expr.body);
            }
            _ => {}
        }

        Ok(visitor.into_result())
    }

    fn language(&self) -> &'static str {
        "python"
    }
}

struct FunctionContext {
    name: String,
    file_path: String,
    line_number: usize,
    args: Vec<(String, String)>,
    return_type: String,
    calls: Vec<String>,
}

struct PythonVisitor<'a> {
    file_path: &'a Path,
    line_index: &'a [usize],
    imports: Vec<Import>,
    functions: Vec<FunctionInfo>,
    data_flows: Vec<DataFlow>,
    call_edges: Vec<CallEdge>,
    entry_points: Vec<EntryPoint>,
    class_stack: Vec<String>,
    function_stack: Vec<String>,
    scope_stack: Vec<HashMap<String, String>>,
    complexity_stack: Vec<usize>,
    current_function: Option<FunctionContext>,
}

impl<'a> PythonVisitor<'a> {
    fn new(file_path: &'a Path, line_index: &'a [usize]) -> Self {
        Self {
            file_path,
            line_index,
            imports: Vec::new(),
            functions: Vec::new(),
            data_flows: Vec::new(),
            call_edges: Vec::new(),
            entry_points: Vec::new(),
            class_stack: Vec::new(),
            function_stack: Vec::new(),
            scope_stack: vec![HashMap::new()],
            complexity_stack: Vec::new(),
            current_function: None,
        }
    }

    fn into_result(self) -> AnalysisResult {
        AnalysisResult {
            imports: self.imports,
            functions: self.functions,
            data_flows: self.data_flows,
            call_edges: self.call_edges,
            entry_points: self.entry_points,
            files: Vec::new(),
            directories: Vec::new(),
        }
    }

    fn file_path_string(&self) -> String {
        self.file_path.to_string_lossy().to_string()
    }

    fn enter_function(&mut self, context: FunctionContext) {
        self.function_stack.push(
            context
                .name
                .rsplit('.')
                .next()
                .unwrap_or_default()
                .to_string(),
        );
        self.scope_stack.push(HashMap::new());
        self.complexity_stack.push(1);
        self.current_function = Some(context);
        self.call_edges.push(CallEdge {
            caller: self.function_stack.join("."),
            callee: self.function_stack.join("."),
            file_path: self.file_path_string(),
        });
    }

    fn exit_function(&mut self) {
        let complexity = self.complexity_stack.pop().unwrap_or(1);
        self.scope_stack.pop();
        self.function_stack.pop();

        if let Some(context) = self.current_function.take() {
            self.functions.push(FunctionInfo {
                name: context.name,
                file_path: context.file_path,
                line_number: context.line_number,
                args: context.args,
                return_type: context.return_type,
                complexity,
                calls: context.calls,
            });
        }
    }

    fn increment_complexity(&mut self, amount: usize) {
        if let Some(score) = self.complexity_stack.last_mut() {
            *score += amount;
        }
    }

    fn enter_class(&mut self, name: String) {
        self.class_stack.push(name);
    }

    fn exit_class(&mut self) {
        self.class_stack.pop();
    }

    fn define_variable(&mut self, name: String, source: String) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.insert(name, source);
        }
    }

    fn resolve_variable(&self, name: &str) -> Option<String> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(source) = scope.get(name) {
                return Some(source.clone());
            }
        }
        None
    }

    fn get_expr_name(&self, expr: &ast::Expr) -> String {
        match expr {
            ast::Expr::Name(ast::ExprName { id, .. }) => id.to_string(),
            ast::Expr::Attribute(ast::ExprAttribute { value, attr, .. }) => {
                format!("{}.{}", self.get_expr_name(value), attr)
            }
            ast::Expr::Constant(ast::ExprConstant { value, .. }) => match value {
                ast::Constant::Str(s) => s.clone(),
                ast::Constant::Bool(b) => b.to_string(),
                ast::Constant::Int(i) => i.to_string(),
                ast::Constant::Float(f) => f.to_string(),
                _ => "literal".to_string(),
            },
            ast::Expr::Subscript(ast::ExprSubscript { value, slice, .. }) => format!(
                "{}[{}]",
                self.get_expr_name(value),
                self.get_expr_name(slice)
            ),
            ast::Expr::Tuple(ast::ExprTuple { elts, .. }) => {
                let parts = elts
                    .iter()
                    .map(|elt| self.get_expr_name(elt))
                    .collect::<Vec<_>>();
                format!("({})", parts.join(", "))
            }
            _ => "Any".to_string(),
        }
    }

    fn line_for_range(&self, range: rustpython_parser::text_size::TextRange) -> usize {
        let offset: usize = range.start().into();
        match self.line_index.binary_search(&offset) {
            Ok(idx) => idx + 1,
            Err(idx) => idx.max(1),
        }
    }

    fn build_function_name(&self, raw: &str) -> String {
        let mut segments = Vec::new();
        if !self.class_stack.is_empty() {
            segments.extend(self.class_stack.clone());
        }
        if !self.function_stack.is_empty() {
            segments.extend(self.function_stack.clone());
        }
        segments.push(raw.to_string());
        segments.join(".")
    }

    fn extract_args(&self, args: &ast::Arguments) -> Vec<(String, String)> {
        let mut all_args = Vec::new();
        for arg in args
            .posonlyargs
            .iter()
            .chain(args.args.iter())
            .chain(args.kwonlyargs.iter())
        {
            let name = arg.def.arg.to_string();
            let ty = arg
                .def
                .annotation
                .as_ref()
                .map(|expr| self.get_expr_name(expr))
                .unwrap_or_else(|| "Any".to_string());
            all_args.push((name, ty));
        }
        if let Some(vararg) = &args.vararg {
            all_args.push((
                format!("*{}", vararg.arg),
                vararg
                    .annotation
                    .as_ref()
                    .map(|expr| self.get_expr_name(expr))
                    .unwrap_or_else(|| "Any".to_string()),
            ));
        }
        if let Some(kwarg) = &args.kwarg {
            all_args.push((
                format!("**{}", kwarg.arg),
                kwarg
                    .annotation
                    .as_ref()
                    .map(|expr| self.get_expr_name(expr))
                    .unwrap_or_else(|| "Any".to_string()),
            ));
        }
        all_args
    }

    fn is_entry_point(&self, test: &ast::Expr) -> bool {
        match test {
            ast::Expr::Compare(ast::ExprCompare {
                left,
                ops: _,
                comparators,
                ..
            }) => {
                if let ast::Expr::Name(ast::ExprName { id, .. }) = left.as_ref() {
                    if id.as_str() == "__name__" {
                        if let Some(ast::Expr::Constant(ast::ExprConstant { value, .. })) =
                            comparators.first()
                        {
                            return matches!(value, ast::Constant::Str(s) if s == "__main__");
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn qualify_callee(&self, callee: &str) -> String {
        if callee.contains('.') {
            return callee.to_string();
        }
        let mut segments = Vec::new();
        if !self.class_stack.is_empty() {
            segments.extend(self.class_stack.clone());
        }
        if !self.function_stack.is_empty() {
            segments.extend(self.function_stack.clone());
        }
        segments.push(callee.to_string());
        if segments.len() > 1 {
            segments.join(".")
        } else {
            callee.to_string()
        }
    }
}

impl<'a> Visitor for PythonVisitor<'a> {
    fn visit_stmt_import(&mut self, node: ast::StmtImport) {
        let line = self.line_for_range(node.range);
        for alias in node.names {
            self.imports.push(Import {
                module: alias.name.to_string(),
                file_path: self.file_path_string(),
                line_number: line,
            });
        }
    }

    fn visit_stmt_import_from(&mut self, node: ast::StmtImportFrom) {
        let line = self.line_for_range(node.range);
        if let Some(module) = node.module {
            self.imports.push(Import {
                module: module.to_string(),
                file_path: self.file_path_string(),
                line_number: line,
            });
        }
    }

    fn visit_stmt_class_def(&mut self, node: ast::StmtClassDef) {
        self.enter_class(node.name.to_string());
        self.generic_visit_stmt_class_def(node);
        self.exit_class();
    }

    fn visit_stmt_function_def(&mut self, node: ast::StmtFunctionDef) {
        let full_name = self.build_function_name(&node.name.to_string());
        let line = self.line_for_range(node.range);
        let args = self.extract_args(&node.args);
        let return_type = node
            .returns
            .as_ref()
            .map(|expr| self.get_expr_name(expr))
            .unwrap_or_else(|| "None".to_string());

        let context = FunctionContext {
            name: full_name,
            file_path: self.file_path_string(),
            line_number: line,
            args,
            return_type,
            calls: Vec::new(),
        };

        self.enter_function(context);
        self.generic_visit_stmt_function_def(node);
        self.exit_function();
    }

    fn visit_stmt_async_function_def(&mut self, node: ast::StmtAsyncFunctionDef) {
        let full_name = self.build_function_name(&node.name.to_string());
        let line = self.line_for_range(node.range);
        let args = self.extract_args(&node.args);
        let return_type = node
            .returns
            .as_ref()
            .map(|expr| self.get_expr_name(expr))
            .unwrap_or_else(|| "None".to_string());

        let context = FunctionContext {
            name: full_name,
            file_path: self.file_path_string(),
            line_number: line,
            args,
            return_type,
            calls: Vec::new(),
        };

        self.enter_function(context);
        self.generic_visit_stmt_async_function_def(node);
        self.exit_function();
    }

    fn visit_stmt_assign(&mut self, node: ast::StmtAssign) {
        let value_source = match node.value.as_ref() {
            ast::Expr::Call(call) => {
                let callee = self.get_expr_name(&call.func);
                Some(self.qualify_callee(&callee))
            }
            ast::Expr::Name(ast::ExprName { id, .. }) => self.resolve_variable(id),
            _ => None,
        };

        if let Some(source) = value_source {
            for target in node.targets.iter() {
                if let ast::Expr::Name(ast::ExprName { id, .. }) = target {
                    self.define_variable(id.to_string(), source.clone());
                }
            }
        }

        self.generic_visit_stmt_assign(node);
    }

    fn visit_expr_call(&mut self, node: ast::ExprCall) {
        let callee_raw = self.get_expr_name(&node.func);
        let callee = self.qualify_callee(&callee_raw);

        for arg in node.args.iter() {
            if let ast::Expr::Name(ast::ExprName { id, .. }) = arg {
                if let Some(source) = self.resolve_variable(id) {
                    self.data_flows.push(DataFlow {
                        source,
                        sink: callee.clone(),
                        variable: Some(id.to_string()),
                        file_path: self.file_path_string(),
                    });
                }
            }
        }

        if let Some(context) = self.current_function.as_mut() {
            context.calls.push(callee);
            if let Some(caller) = self.function_stack.last().cloned() {
                self.call_edges.push(CallEdge {
                    caller,
                    callee: callee_raw,
                    file_path: self.file_path_string(),
                });
            }
        }

        self.generic_visit_expr_call(node);
    }

    fn visit_stmt_if(&mut self, node: ast::StmtIf) {
        if self.is_entry_point(&node.test) {
            let line = self.line_for_range(node.range);
            self.entry_points.push(EntryPoint {
                file_path: self.file_path_string(),
                line_number: line,
                condition: "__name__ == \"__main__\"".to_string(),
            });
        }

        self.increment_complexity(1);
        self.generic_visit_stmt_if(node);
    }

    fn visit_stmt_for(&mut self, node: ast::StmtFor) {
        self.increment_complexity(1);
        self.generic_visit_stmt_for(node);
    }

    fn visit_stmt_async_for(&mut self, node: ast::StmtAsyncFor) {
        self.increment_complexity(1);
        self.generic_visit_stmt_async_for(node);
    }

    fn visit_stmt_while(&mut self, node: ast::StmtWhile) {
        self.increment_complexity(1);
        self.generic_visit_stmt_while(node);
    }

    fn visit_stmt_match(&mut self, node: ast::StmtMatch) {
        self.increment_complexity(1);
        self.generic_visit_stmt_match(node);
    }

    fn visit_excepthandler(&mut self, node: ast::ExceptHandler) {
        self.increment_complexity(1);
        self.generic_visit_excepthandler(node);
    }

    fn visit_expr_bool_op(&mut self, node: ast::ExprBoolOp) {
        if node.values.len() > 1 {
            self.increment_complexity(node.values.len() - 1);
        }
        self.generic_visit_expr_bool_op(node);
    }
}

fn build_line_index(content: &str) -> Vec<usize> {
    let mut offsets = vec![0];
    for (idx, byte) in content.as_bytes().iter().enumerate() {
        if *byte == b'\n' {
            offsets.push(idx + 1);
        }
    }
    offsets
}
