use super::{
    AnalysisResult, Analyzer, CallEdge, DataFlow, EntryPoint, EnvVarUsage, ExternalCall,
    FunctionInfo, Import,
};
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::path::Path;
use swc_common::comments::SingleThreadedComments;
use swc_common::{sync::Lrc, FileName, SourceMap, Span};
use swc_ecma_ast::EsVersion;
use swc_ecma_ast::{
    AssignExpr, AssignTarget, AssignTargetPat, BinExpr, BinaryOp, CallExpr, Callee, CatchClause,
    ClassDecl, ClassMethod, DoWhileStmt, ExportAll, ExportNamedSpecifier, Expr, FnDecl, FnExpr,
    ForInStmt, ForOfStmt, ForStmt, IfStmt, ImportDecl, MemberProp, ModuleExportName, ObjectPatProp,
    Param, Pat, Program, PropName, SimpleAssignTarget, SwitchStmt, TsEntityName, TsType,
    TsUnionOrIntersectionType, VarDeclarator, WhileStmt,
};
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::{Parser, StringInput, Syntax, TsSyntax};
use swc_ecma_visit::{noop_visit_type, Visit, VisitWith};
use tracing::warn;

#[derive(Default)]
pub struct JavascriptAnalyzer;

impl Analyzer for JavascriptAnalyzer {
    fn supports(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| {
                matches!(
                    ext.to_ascii_lowercase().as_str(),
                    "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs"
                )
            })
            .unwrap_or(false)
    }

    fn analyze(&self, path: &Path) -> Result<AnalysisResult> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read JS/TS file {:?}", path))?;

        let cm: Lrc<SourceMap> = Default::default();
        let fm = cm.new_source_file(FileName::Real(path.to_path_buf()).into(), content.clone());
        let comments = SingleThreadedComments::default();

        let syntax = Syntax::Typescript(TsSyntax {
            tsx: true,
            decorators: true,
            dts: false,
            no_early_errors: false,
            ..Default::default()
        });

        let lexer = Lexer::new(
            syntax,
            EsVersion::Es2022,
            StringInput::from(&*fm),
            Some(&comments),
        );

        let mut parser = Parser::new_from(lexer);
        let module = parser
            .parse_module()
            .map_err(|err| anyhow!("{:?}", err))
            .with_context(|| format!("Failed to parse JS/TS file {:?}", path))?;

        for err in parser.take_errors() {
            warn!("Parser error in {:?}: {:?}", path, err);
        }

        let mut visitor = JsVisitor::new(path, cm.clone());
        Program::Module(module).visit_with(&mut visitor);

        Ok(visitor.into_result())
    }

    fn language(&self) -> &'static str {
        "javascript"
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

struct JsVisitor<'a> {
    file_path: &'a Path,
    cm: Lrc<SourceMap>,
    imports: Vec<Import>,
    import_aliases: HashMap<String, String>,
    functions: Vec<FunctionInfo>,
    data_flows: Vec<DataFlow>,
    call_edges: Vec<CallEdge>,
    entry_points: Vec<EntryPoint>,
    exits: Vec<EntryPoint>,
    external_calls: Vec<ExternalCall>,
    env_vars: Vec<EnvVarUsage>,
    class_stack: Vec<String>,
    function_stack: Vec<String>,
    scope_stack: Vec<HashMap<String, String>>,
    complexity_stack: Vec<usize>,
    current_function: Option<FunctionContext>,
}

impl<'a> JsVisitor<'a> {
    fn new(file_path: &'a Path, cm: Lrc<SourceMap>) -> Self {
        Self {
            file_path,
            cm,
            imports: Vec::new(),
            import_aliases: HashMap::new(),
            functions: Vec::new(),
            data_flows: Vec::new(),
            call_edges: Vec::new(),
            entry_points: Vec::new(),
            exits: Vec::new(),
            external_calls: Vec::new(),
            env_vars: Vec::new(),
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
            exits: self.exits,
            external_calls: self.external_calls,
            env_vars: self.env_vars,
            files: Vec::new(),
            directories: Vec::new(),
            infra: None,
            infra_correlation: None,
        }
    }

    fn file_path_string(&self) -> String {
        self.file_path.to_string_lossy().to_string()
    }

    fn line_for_span(&self, span: Span) -> usize {
        self.cm.lookup_char_pos(span.lo()).line + 1
    }

    fn record_env_var(&self, name: String, span: Span, kind: &str) -> EnvVarUsage {
        EnvVarUsage {
            name,
            file_path: self.file_path_string(),
            line_number: self.line_for_span(span),
            kind: kind.to_string(),
        }
    }

    fn extract_env_var(&self, member: &swc_ecma_ast::MemberExpr) -> Option<EnvVarUsage> {
        // process.env.VAR or process.env["VAR"]
        if let swc_ecma_ast::Expr::Member(base) = &*member.obj {
            if self.is_process_env(base) {
                if let Some(name) = self.prop_to_string(&member.prop) {
                    return Some(self.record_env_var(name, member.span, "process.env"));
                }
            }
        }
        None
    }

    fn is_process_env(&self, member: &swc_ecma_ast::MemberExpr) -> bool {
        if let (swc_ecma_ast::Expr::Ident(obj), swc_ecma_ast::MemberProp::Ident(prop)) =
            (&*member.obj, &member.prop)
        {
            obj.sym == *"process" && prop.sym == *"env"
        } else {
            false
        }
    }

    fn prop_to_string(&self, prop: &MemberProp) -> Option<String> {
        match prop {
            MemberProp::Ident(id) => Some(id.sym.to_string()),
            MemberProp::Computed(c) => match &*c.expr {
                Expr::Lit(swc_ecma_ast::Lit::Str(s)) => Some(s.value.to_string_lossy().to_string()),
                _ => None,
            },
            _ => None,
        }
    }

    fn increment_complexity(&mut self, amount: usize) {
        if let Some(score) = self.complexity_stack.last_mut() {
            *score += amount;
        }
    }

    fn enter_function(&mut self, mut context: FunctionContext) {
        context.name = format!(
            "{}::{}",
            self.file_path_string(),
            self.qualify_name(&context.name)
        );
        self.function_stack.push(
            context
                .name
                .rsplit("::")
                .next()
                .unwrap_or_default()
                .to_string(),
        );
        self.scope_stack.push(HashMap::new());
        self.complexity_stack.push(1);
        self.current_function = Some(context);
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

    fn qualify_name(&self, raw: &str) -> String {
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

    fn get_expr_name(&self, expr: &Expr) -> String {
        match expr {
            Expr::Ident(ident) => {
                if let Some(module) = self.import_aliases.get(&ident.sym.to_string()) {
                    format!("{module}::{}", ident.sym)
                } else {
                    ident.sym.to_string()
                }
            }
            Expr::Member(member) => {
                let obj = match &*member.obj {
                    Expr::Ident(ident) => {
                        if let Some(module) = self.import_aliases.get(&ident.sym.to_string()) {
                            format!("{module}::{}", ident.sym)
                        } else {
                            ident.sym.to_string()
                        }
                    }
                    Expr::This(_) => "this".to_string(),
                    Expr::Member(inner) => self.get_expr_name(&Expr::Member(inner.clone())),
                    other => format!("({})", self.get_expr_name(other)),
                };
                let prop = match &member.prop {
                    MemberProp::Ident(id) => id.sym.to_string(),
                    MemberProp::PrivateName(name) => name.name.to_string(),
                    MemberProp::Computed(comp) => self.get_expr_name(&comp.expr),
                };
                format!("{obj}.{prop}")
            }
            Expr::Call(call) => self.get_callee_name(&call.callee),
            _ => "complex".to_string(),
        }
    }

    fn get_callee_name(&self, callee: &Callee) -> String {
        match callee {
            Callee::Expr(expr) => self.get_expr_name(expr),
            Callee::Super(_) => "super".to_string(),
            Callee::Import(_) => "import".to_string(),
        }
    }

    fn detect_external_call(&self, call: &CallExpr, callee_name: &str) -> Option<ExternalCall> {
        let lc = callee_name.to_ascii_lowercase();

        // HTTP clients
        let is_fetch = lc == "fetch" || lc.ends_with(".fetch");
        let is_axios = lc.contains("axios")
            || lc.ends_with(".get")
            || lc.ends_with(".post")
            || lc.ends_with(".put")
            || lc.ends_with(".delete")
            || lc.ends_with(".patch");
        let is_http_client = lc.contains("httpclient")
            || lc.contains("superagent")
            || lc.contains("got.")
            || lc.contains("node-fetch");

        // Cloud SDKs
        let is_aws = lc.contains("aws.")
            || lc.contains("awssdk")
            || lc.contains("s3.")
            || lc.contains("dynamodb.")
            || lc.contains("lambda.")
            || lc.contains("sqs.")
            || lc.contains("sns.");
        let is_gcp = lc.contains("google.cloud")
            || lc.contains("@google-cloud")
            || lc.contains("storage.")
            || lc.contains("bigquery.");
        let is_azure = lc.contains("@azure/")
            || lc.contains("azure.")
            || lc.contains("blobserviceclient")
            || lc.contains("cosmosclient");

        // Databases
        let is_database = lc.contains("pg.")
            || lc.contains("postgres")
            || lc.contains("mysql")
            || lc.contains("mongodb")
            || lc.contains("redis")
            || lc.contains(".query(")
            || lc.contains(".execute(")
            || lc.contains("prisma.")
            || lc.contains("sequelize.")
            || lc.contains("typeorm.");

        // Message queues & streaming
        let is_messaging = lc.contains("kafka")
            || lc.contains("amqp")
            || lc.contains("rabbitmq")
            || lc.contains("producer.")
            || lc.contains("consumer.")
            || lc.contains("publish(");

        if !(is_fetch
            || is_axios
            || is_http_client
            || is_aws
            || is_gcp
            || is_azure
            || is_database
            || is_messaging)
        {
            return None;
        }

        let path = call.args.first().and_then(|a| match &*a.expr {
            Expr::Lit(swc_ecma_ast::Lit::Str(s)) => Some(s.value.to_string_lossy().into_owned()),
            _ => None,
        });

        let method = if is_fetch || is_axios || is_http_client {
            // Extract HTTP method
            if lc.contains(".get") || lc.ends_with(".get") {
                Some("GET".to_string())
            } else if lc.contains(".post") || lc.ends_with(".post") {
                Some("POST".to_string())
            } else if lc.contains(".put") || lc.ends_with(".put") {
                Some("PUT".to_string())
            } else if lc.contains(".delete") || lc.ends_with(".delete") {
                Some("DELETE".to_string())
            } else if lc.contains(".patch") || lc.ends_with(".patch") {
                Some("PATCH".to_string())
            } else {
                // Check for method in options object (second argument)
                call.args.get(1).and_then(|opt| match &*opt.expr {
                    Expr::Object(obj) => obj.props.iter().find_map(|prop| {
                        if let swc_ecma_ast::PropOrSpread::Prop(p) = prop {
                            if let swc_ecma_ast::Prop::KeyValue(kv) = &**p {
                                if let PropName::Ident(id) = &kv.key {
                                    if id.sym.as_ref() == "method" {
                                        if let Expr::Lit(swc_ecma_ast::Lit::Str(s)) = &*kv.value {
                                            return Some(
                                                s.value.to_string_lossy().to_uppercase(),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        None
                    }),
                    _ => None,
                })
            }
        } else {
            None
        };

        Some(ExternalCall {
            target: callee_name.to_string(),
            method,
            path,
            file_path: self.file_path_string(),
            line_number: self.line_for_span(call.span),
        })
    }

    fn ts_type_to_string(&self, ty: &TsType) -> String {
        match ty {
            TsType::TsKeywordType(keyword) => format!("{:?}", keyword.kind)
                .replace("Ts", "")
                .replace("Keyword", "")
                .to_lowercase(),
            TsType::TsTypeRef(tr) => self.entity_name_to_string(&tr.type_name),
            TsType::TsArrayType(arr) => format!("{}[]", self.ts_type_to_string(&arr.elem_type)),
            TsType::TsUnionOrIntersectionType(TsUnionOrIntersectionType::TsUnionType(u)) => u
                .types
                .iter()
                .map(|t| self.ts_type_to_string(t))
                .collect::<Vec<_>>()
                .join(" | "),
            TsType::TsParenthesizedType(p) => self.ts_type_to_string(&p.type_ann),
            TsType::TsTypeLit(_) => "type".to_string(),
            _ => "any".to_string(),
        }
    }

    fn entity_name_to_string(&self, name: &TsEntityName) -> String {
        match name {
            TsEntityName::Ident(id) => id.sym.to_string(),
            TsEntityName::TsQualifiedName(q) => {
                let left = self.entity_name_to_string(&q.left);
                format!("{left}.{}", q.right.sym)
            }
        }
    }

    fn extract_params(&self, params: &[Param]) -> Vec<(String, String)> {
        let mut result = Vec::new();
        for param in params {
            let names = self.bind_pat(&param.pat);
            let ty = match &param.pat {
                Pat::Ident(ident) => ident
                    .type_ann
                    .as_ref()
                    .map(|ann| self.ts_type_to_string(&ann.type_ann))
                    .unwrap_or_else(|| "any".to_string()),
                _ => "any".to_string(),
            };
            for name in names {
                result.push((name, ty.clone()));
            }
        }
        result
    }

    fn bind_pat(&self, pat: &Pat) -> Vec<String> {
        match pat {
            Pat::Ident(bi) => vec![bi.id.sym.to_string()],
            Pat::Array(arr) => arr
                .elems
                .iter()
                .filter_map(|el| el.as_ref())
                .flat_map(|el| self.bind_pat(el))
                .collect(),
            Pat::Object(obj) => obj
                .props
                .iter()
                .flat_map(|prop| match prop {
                    ObjectPatProp::KeyValue(kv) => self.bind_pat(&kv.value),
                    ObjectPatProp::Assign(assign) => vec![assign.key.sym.to_string()],
                    ObjectPatProp::Rest(rest) => self.bind_pat(&rest.arg),
                })
                .collect(),
            Pat::Assign(assign) => self.bind_pat(&assign.left),
            _ => Vec::new(),
        }
    }

    fn bind_assign_target_pat(&self, pat: &AssignTargetPat) -> Vec<String> {
        match pat {
            AssignTargetPat::Array(arr) => arr
                .elems
                .iter()
                .filter_map(|el| el.as_ref())
                .flat_map(|el| self.bind_pat(el))
                .collect(),
            AssignTargetPat::Object(obj) => obj
                .props
                .iter()
                .flat_map(|prop| match prop {
                    ObjectPatProp::KeyValue(kv) => self.bind_pat(&kv.value),
                    ObjectPatProp::Assign(assign) => vec![assign.key.sym.to_string()],
                    ObjectPatProp::Rest(rest) => self.bind_pat(&rest.arg),
                })
                .collect(),
            AssignTargetPat::Invalid(_) => Vec::new(),
        }
    }

    fn is_entry_point_condition(&self, expr: &Expr) -> bool {
        if let Expr::Bin(bin) = expr {
            if matches!(bin.op, BinaryOp::EqEqEq | BinaryOp::EqEq) {
                let left = self.get_expr_name(&bin.left);
                let right = self.get_expr_name(&bin.right);
                return (left == "require.main" && right == "module")
                    || (right == "require.main" && left == "module");
            }
        }
        false
    }
}

impl<'a> Visit for JsVisitor<'a> {
    fn visit_import_decl(&mut self, n: &ImportDecl) {
        let module = n.src.value.to_string_lossy().into_owned();
        for spec in &n.specifiers {
            match spec {
                swc_ecma_ast::ImportSpecifier::Named(named) => {
                    let local = named.local.sym.to_string();
                    self.import_aliases.insert(local.clone(), module.clone());
                }
                swc_ecma_ast::ImportSpecifier::Default(default) => {
                    let local = default.local.sym.to_string();
                    self.import_aliases.insert(local.clone(), module.clone());
                }
                swc_ecma_ast::ImportSpecifier::Namespace(ns) => {
                    let local = ns.local.sym.to_string();
                    self.import_aliases.insert(local.clone(), module.clone());
                }
            }
        }
        self.imports.push(Import {
            module: module.clone(),
            file_path: self.file_path_string(),
            line_number: self.line_for_span(n.span),
        });
    }

    fn visit_member_expr(&mut self, n: &swc_ecma_ast::MemberExpr) {
        if let Some(env) = self.extract_env_var(n) {
            self.env_vars.push(env);
        }
        n.visit_children_with(self);
    }

    fn visit_export_named_specifier(&mut self, n: &ExportNamedSpecifier) {
        if let Some(src) = &n.exported {
            let module = match src {
                ModuleExportName::Ident(id) => id.sym.to_string(),
                ModuleExportName::Str(s) => s.value.to_string_lossy().into_owned(),
            };
            self.imports.push(Import {
                module,
                file_path: self.file_path_string(),
                line_number: self.line_for_span(n.span),
            });
        }
    }

    fn visit_export_all(&mut self, n: &ExportAll) {
        self.imports.push(Import {
            module: n.src.value.to_string_lossy().into_owned(),
            file_path: self.file_path_string(),
            line_number: self.line_for_span(n.span),
        });
    }

    fn visit_class_decl(&mut self, n: &ClassDecl) {
        self.enter_class(n.ident.sym.to_string());
        n.class.visit_with(self);
        self.exit_class();
    }

    fn visit_fn_decl(&mut self, n: &FnDecl) {
        let name = self.qualify_name(&n.ident.sym.to_string());
        let line = self.line_for_span(n.function.span);
        let args = self.extract_params(&n.function.params);
        let return_type = n
            .function
            .return_type
            .as_ref()
            .map(|ann| self.ts_type_to_string(&ann.type_ann))
            .unwrap_or_else(|| "any".to_string());

        let ctx = FunctionContext {
            name,
            file_path: self.file_path_string(),
            line_number: line,
            args,
            return_type,
            calls: Vec::new(),
        };

        self.enter_function(ctx);
        n.function.visit_with(self);
        self.exit_function();
    }

    fn visit_class_method(&mut self, n: &ClassMethod) {
        let method_name = match &n.key {
            PropName::Ident(id) => id.sym.to_string(),
            PropName::Str(s) => s.value.to_string_lossy().into_owned(),
            PropName::Num(num) => num.value.to_string(),
            PropName::Computed(comp) => self.get_expr_name(&comp.expr),
            PropName::BigInt(bi) => bi.value.to_string(),
        };
        let name = self.qualify_name(&method_name);
        let line = self.line_for_span(n.function.span);
        let args = self.extract_params(&n.function.params);
        let return_type = n
            .function
            .return_type
            .as_ref()
            .map(|ann| self.ts_type_to_string(&ann.type_ann))
            .unwrap_or_else(|| "any".to_string());

        let ctx = FunctionContext {
            name,
            file_path: self.file_path_string(),
            line_number: line,
            args,
            return_type,
            calls: Vec::new(),
        };

        self.enter_function(ctx);
        n.function.visit_with(self);
        self.exit_function();
    }

    fn visit_fn_expr(&mut self, n: &FnExpr) {
        if let Some(id) = &n.ident {
            let name = self.qualify_name(&id.sym.to_string());
            let line = self.line_for_span(n.function.span);
            let args = self.extract_params(&n.function.params);
            let return_type = n
                .function
                .return_type
                .as_ref()
                .map(|ann| self.ts_type_to_string(&ann.type_ann))
                .unwrap_or_else(|| "any".to_string());

            let ctx = FunctionContext {
                name,
                file_path: self.file_path_string(),
                line_number: line,
                args,
                return_type,
                calls: Vec::new(),
            };

            self.enter_function(ctx);
            n.function.visit_with(self);
            self.exit_function();
        } else {
            n.visit_children_with(self);
        }
    }

    fn visit_var_declarator(&mut self, n: &VarDeclarator) {
        let names = self.bind_pat(&n.name);
        let value_source = if let Some(init) = &n.init {
            match init.as_ref() {
                Expr::Call(call) => Some(self.get_callee_name(&call.callee)),
                Expr::Ident(id) => self.resolve_variable(&id.sym.to_string()),
                Expr::Member(mem) => Some(self.get_expr_name(&Expr::Member(mem.clone()))),
                _ => None,
            }
        } else {
            None
        };

        if let Some(source) = value_source {
            for name in names {
                self.define_variable(name, source.clone());
            }
        }

        n.init.visit_with(self);
    }

    fn visit_assign_expr(&mut self, n: &AssignExpr) {
        let names = match &n.left {
            AssignTarget::Simple(SimpleAssignTarget::Ident(id)) => vec![id.id.sym.to_string()],
            AssignTarget::Simple(SimpleAssignTarget::Member(mem)) => {
                vec![self.get_expr_name(&Expr::Member(mem.clone()))]
            }
            AssignTarget::Pat(pat) => self.bind_assign_target_pat(pat),
            _ => Vec::new(),
        };

        let value_source = match n.right.as_ref() {
            Expr::Call(call) => Some(self.get_callee_name(&call.callee)),
            Expr::Ident(id) => self.resolve_variable(&id.sym.to_string()),
            Expr::Member(mem) => Some(self.get_expr_name(&Expr::Member(mem.clone()))),
            _ => None,
        };

        if let Some(source) = value_source {
            for name in names {
                self.define_variable(name, source.clone());
            }
        }

        n.visit_children_with(self);
    }

    fn visit_call_expr(&mut self, n: &CallExpr) {
        let callee_raw = self.get_callee_name(&n.callee);
        let callee = callee_raw.clone();
        for arg in &n.args {
            if let Expr::Ident(id) = &*arg.expr {
                if let Some(source) = self.resolve_variable(&id.sym.to_string()) {
                    self.data_flows.push(DataFlow {
                        source,
                        sink: callee.clone(),
                        variable: Some(id.sym.to_string()),
                        file_path: self.file_path_string(),
                    });
                }
            }
        }

        if let Some(ctx) = self.current_function.as_mut() {
            ctx.calls.push(callee_raw.clone());
            if let Some(caller) = self.function_stack.last().cloned() {
                self.call_edges.push(CallEdge {
                    caller,
                    callee: callee_raw.clone(),
                    file_path: self.file_path_string(),
                });
            }
        }

        // External call detection: fetch/axios/http/SDK
        if let Some(ext) = self.detect_external_call(&n, &callee_raw) {
            self.external_calls.push(ext);
        }

        n.visit_children_with(self);
    }

    fn visit_if_stmt(&mut self, n: &IfStmt) {
        if self.is_entry_point_condition(&n.test) {
            self.entry_points.push(EntryPoint {
                file_path: self.file_path_string(),
                line_number: self.line_for_span(n.span),
                condition: "require.main === module".to_string(),
            });
        }

        self.increment_complexity(1);
        n.visit_children_with(self);
    }

    fn visit_for_stmt(&mut self, n: &ForStmt) {
        self.increment_complexity(1);
        n.visit_children_with(self);
    }

    fn visit_for_of_stmt(&mut self, n: &ForOfStmt) {
        self.increment_complexity(1);
        n.visit_children_with(self);
    }

    fn visit_for_in_stmt(&mut self, n: &ForInStmt) {
        self.increment_complexity(1);
        n.visit_children_with(self);
    }

    fn visit_while_stmt(&mut self, n: &WhileStmt) {
        self.increment_complexity(1);
        n.visit_children_with(self);
    }

    fn visit_do_while_stmt(&mut self, n: &DoWhileStmt) {
        self.increment_complexity(1);
        n.visit_children_with(self);
    }

    fn visit_switch_stmt(&mut self, n: &SwitchStmt) {
        if !n.cases.is_empty() {
            self.increment_complexity(n.cases.len());
        }
        n.visit_children_with(self);
    }

    fn visit_catch_clause(&mut self, n: &CatchClause) {
        self.increment_complexity(1);
        n.visit_children_with(self);
    }

    fn visit_bin_expr(&mut self, n: &BinExpr) {
        if matches!(n.op, BinaryOp::LogicalAnd | BinaryOp::LogicalOr) {
            self.increment_complexity(1);
        }
        n.visit_children_with(self);
    }

    noop_visit_type!();
}
