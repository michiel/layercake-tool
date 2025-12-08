use anyhow::Result;
use hcl::{Body, Value as HclValue};
use ignore::WalkBuilder;
use rayon::prelude::*;
use rustpython_ast::Visitor;
use rustpython_parser::{parse, Mode};
use serde_yaml::Value as YamlValue;
use std::collections::HashMap;
use std::path::Path;
use swc_common::{sync::Lrc, FileName, SourceMap};
use swc_ecma_ast::{CallExpr, Expr, NewExpr, PropName};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};
use swc_ecma_visit::{noop_visit_type, Visit, VisitWith};
use tracing::warn;

use super::graph::{slugify_id, InfrastructureGraph};
use super::model::{EdgeType, GraphEdge, ResourceNode, ResourceType};

#[derive(Debug, Default)]
pub struct InfraScanResult {
    pub resources: Vec<ResourceNode>,
    pub edges: Vec<GraphEdge>,
    pub diagnostics: Vec<String>,
}

pub fn analyze_infra(path: &Path) -> Result<InfrastructureGraph> {
    let root = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mut graph = InfrastructureGraph::new("infra");
    let mut diagnostics = Vec::new();

    let walker = WalkBuilder::new(path)
        .hidden(false)
        .parents(true)
        .ignore(true)
        .git_ignore(true)
        .git_exclude(true)
        .git_global(true)
        .build();

    let entries: Vec<_> = walker
        .filter_map(|entry| match entry {
            Ok(e) if e.file_type().map(|t| t.is_file()).unwrap_or(false) => Some(e.into_path()),
            Ok(_) => None,
            Err(err) => {
                warn!("Skipping infra entry: {err}");
                None
            }
        })
        .collect();

    let scans: Vec<_> = entries
        .par_iter()
        .map(|entry_path| {
            let ext = entry_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();

            let relative = entry_path
                .strip_prefix(&root)
                .unwrap_or_else(|_| entry_path.as_path())
                .to_string_lossy()
                .to_string();

            let scan = match ext.as_str() {
                "tf" => parse_terraform(entry_path, &relative),
                "yaml" | "yml" => parse_cloudformation(entry_path, &relative),
                "bicep" => parse_bicep(entry_path, &relative),
                "ts" | "tsx" => parse_cdk_typescript(entry_path, &relative),
                "py" => parse_cdk_python(entry_path, &relative),
                _ => InfraScanResult::default(),
            };
            (relative, scan)
        })
        .collect();

    for (relative, scan) in scans {
        diagnostics.extend(scan.diagnostics);
        for mut resource in scan.resources {
            if resource.belongs_to.is_none() {
                // Use directory as partition when available
                if let Some(parent) = Path::new(&relative).parent() {
                    let label = parent.to_string_lossy().to_string();
                    let partition = graph.ensure_partition(label, None, None);
                    resource.belongs_to = Some(partition);
                }
            }
            graph.add_resource(resource);
        }
        for edge in scan.edges {
            graph.add_edge(edge);
        }
    }

    // Infer reference edges when resource properties mention other resources by id/name
    let mut inferred = Vec::new();
    let resources_snapshot: Vec<(String, String)> = graph
        .resources
        .values()
        .map(|r| (r.id.clone(), r.name.clone()))
        .collect();
    let mut seen = std::collections::HashSet::new();
    for resource in graph.resources.values() {
        for val in resource.properties.values() {
            let lower = val.to_ascii_lowercase();
            for (other_id, other_name) in &resources_snapshot {
                if other_id == &resource.id {
                    continue;
                }
                if lower.contains(&other_id.to_ascii_lowercase())
                    || lower.contains(&other_name.to_ascii_lowercase())
                {
                    let key = (resource.id.clone(), other_id.clone());
                    if seen.insert(key.clone()) {
                        inferred.push(GraphEdge {
                            from: resource.id.clone(),
                            to: other_id.clone(),
                            edge_type: EdgeType::References,
                            label: Some("inferred_ref".into()),
                        });
                    }
                }
            }
        }
    }
    for edge in inferred {
        graph.add_edge(edge);
    }

    graph.validate_edges();
    graph.diagnostics.extend(diagnostics);
    Ok(graph)
}

fn parse_terraform(path: &Path, relative: &str) -> InfraScanResult {
    let mut result = InfraScanResult::default();
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(err) => {
            result
                .diagnostics
                .push(format!("Failed to read {relative}: {err}"));
            return result;
        }
    };

    let body: Body = match hcl::from_str(&content) {
        Ok(b) => b,
        Err(err) => {
            result
                .diagnostics
                .push(format!("Failed to parse HCL {relative}: {err}"));
            return result;
        }
    };

    for block in body.blocks() {
        if block.identifier() != "resource" {
            continue;
        }
        let labels = block.labels();
        if labels.len() < 2 {
            continue;
        }
        let provider_type = labels[0].as_str().to_string();
        let logical_name = labels[1].as_str().to_string();
        let id = slugify_id(&format!("{provider_type}.{logical_name}"));
        let mut node = ResourceNode::new(
            id.clone(),
            ResourceType::from_raw(&provider_type),
            &logical_name,
            relative,
        );
        let mut props = HashMap::new();
        for attr in block.body().attributes() {
            let key = attr.key().to_string();
            if key == "depends_on" {
                if let Some(deps) = extract_strings(attr.expr()) {
                    for dep in deps {
                        result.edges.push(GraphEdge {
                            from: id.clone(),
                            to: slugify_id(&dep),
                            edge_type: EdgeType::DependsOn,
                            label: Some("depends_on".into()),
                        });
                    }
                }
                continue;
            }
            if let Some(val) = extract_value(attr.expr()) {
                props.insert(key, val);
            }
        }
        // Handler hint for Lambda functions
        if provider_type.contains("aws_lambda_function") {
            if let Some(handler) = props.get("handler").cloned() {
                let code = props
                    .get("filename")
                    .or_else(|| props.get("source_code_hash"))
                    .cloned()
                    .unwrap_or_default();
                let clean = format!("{}/{}", code.trim_end_matches('/'), handler);
                props.insert("handler_path".into(), clean);
            }
        }
        node.properties = props;
        result.resources.push(node);
    }

    result
}

fn parse_cloudformation(path: &Path, relative: &str) -> InfraScanResult {
    let mut result = InfraScanResult::default();
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(err) => {
            result
                .diagnostics
                .push(format!("Failed to read {relative}: {err}"));
            return result;
        }
    };

    let value: YamlValue = match serde_yaml::from_str(&content) {
        Ok(v) => v,
        Err(err) => {
            result
                .diagnostics
                .push(format!("Failed to parse YAML {relative}: {err}"));
            return result;
        }
    };

    let resources = match value.get("Resources").and_then(|v| v.as_mapping()) {
        Some(map) => map,
        None => return result,
    };

    for (name, body) in resources {
        let name_str = name.as_str().unwrap_or_default();
        let res_type = body
            .get(&YamlValue::from("Type"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let id = slugify_id(&format!("{res_type}.{name_str}"));

        let mut node = ResourceNode::new(
            id.clone(),
            ResourceType::from_raw(res_type),
            name_str,
            relative,
        );
        let mut props = HashMap::new();
        if let Some(props_map) = body
            .get(&YamlValue::from("Properties"))
            .and_then(|v| v.as_mapping())
        {
            for (k, v) in props_map {
                if let Some(key) = k.as_str() {
                    props.insert(key.to_string(), yaml_to_string(v));
                }
            }

            // If we have CodeUri + Handler, stitch a handler_path hint to help correlation
            if let Some(handler) = props_map
                .get(&YamlValue::from("Handler"))
                .and_then(|v| v.as_str())
            {
                if let Some(code_uri) = props_map
                    .get(&YamlValue::from("CodeUri"))
                    .and_then(|v| v.as_str())
                {
                    let clean = format!("{}/{}", code_uri.trim_end_matches('/'), handler);
                    props.insert("handler_path".into(), clean);
                }
            }
        }
        node.properties = props;

        if let Some(depends_on) = body.get(&YamlValue::from("DependsOn")) {
            match depends_on {
                YamlValue::String(dep) => {
                    result.edges.push(GraphEdge {
                        from: id.clone(),
                        to: slugify_id(dep),
                        edge_type: EdgeType::DependsOn,
                        label: Some("DependsOn".into()),
                    });
                }
                YamlValue::Sequence(seq) => {
                    for dep in seq {
                        if let Some(dep_str) = dep.as_str() {
                            result.edges.push(GraphEdge {
                                from: id.clone(),
                                to: slugify_id(dep_str),
                                edge_type: EdgeType::DependsOn,
                                label: Some("DependsOn".into()),
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        result.resources.push(node);
    }

    result
}

fn parse_bicep(path: &Path, relative: &str) -> InfraScanResult {
    let mut result = InfraScanResult::default();
    let source = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(err) => {
            result
                .diagnostics
                .push(format!("Failed to read {relative}: {err}"));
            return result;
        }
    };

    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("resource") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 3 {
                let name = parts[1].trim_matches('"').trim_matches('\'');
                let rtype = parts[2].trim_matches('"').trim_matches('\'');
                let id = slugify_id(&format!("{rtype}.{name}"));
                let node = ResourceNode::new(id, ResourceType::from_raw(rtype), name, relative);
                result.resources.push(node);
            }
        }
    }

    result
}

fn enrich_construct_props(construct: &str, props: &mut HashMap<String, String>) {
    let lc = construct.to_ascii_lowercase();
    if lc.contains("function") {
        if let Some(handler) = props.get("handler").cloned() {
            let base = props
                .get("code")
                .or_else(|| props.get("entry"))
                .cloned()
                .unwrap_or_default();
            let hint = if base.is_empty() {
                handler.clone()
            } else {
                format!("{}/{}", base.trim_end_matches('/'), handler)
            };
            props.entry("handler_path".into()).or_insert(hint);
        }
    }
    if lc.contains("table") {
        if let Some(pk) = props.get("partitionKey").cloned() {
            props.entry("key.partition".into()).or_insert(pk);
        }
        if let Some(sk) = props.get("sortKey").cloned() {
            props.entry("key.sort".into()).or_insert(sk);
        }
    }
}

fn parse_cdk_python(path: &Path, relative: &str) -> InfraScanResult {
    let mut result = InfraScanResult::default();
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(err) => {
            result
                .diagnostics
                .push(format!("Failed to read {relative}: {err}"));
            return result;
        }
    };

    let module = match parse(&content, Mode::Module, relative) {
        Ok(m) => m,
        Err(err) => {
            result
                .diagnostics
                .push(format!("Failed to parse CDK python {relative}: {err}"));
            return result;
        }
    };

    let mut visitor = CdkPyVisitor::new(relative);
    match module {
        rustpython_ast::Mod::Module(m) => {
            for stmt in m.body {
                visitor.visit_stmt(stmt);
            }
        }
        _ => {}
    }

    result.resources.extend(visitor.resources);
    result
}

fn parse_cdk_typescript(path: &Path, relative: &str) -> InfraScanResult {
    let mut result = InfraScanResult::default();
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(err) => {
            result
                .diagnostics
                .push(format!("Failed to read {relative}: {err}"));
            return result;
        }
    };

    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(FileName::Custom(relative.to_string()).into(), content);
    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: relative.ends_with(".tsx"),
            decorators: true,
            ..Default::default()
        }),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );
    let mut parser = Parser::new_from(lexer);
    let module = match parser.parse_module() {
        Ok(m) => m,
        Err(err) => {
            result
                .diagnostics
                .push(format!("Failed to parse TS {relative}: {err:?}"));
            return result;
        }
    };

    let mut visitor = CdkTsVisitor::new(relative.to_string());
    module.visit_with(&mut visitor);
    result.resources.extend(visitor.resources);
    result
}

fn _basic_resource(
    resource_type: &str,
    name: &str,
    file: &str,
    belongs_to: Option<String>,
) -> ResourceNode {
    let mut node = ResourceNode::new(
        slugify_id(&format!("{resource_type}_{name}")),
        ResourceType::from_raw(resource_type),
        name,
        file,
    );
    node.belongs_to = belongs_to;
    node
}

fn _basic_edge(from: &str, to: &str, edge_type: EdgeType) -> GraphEdge {
    GraphEdge {
        from: slugify_id(from),
        to: slugify_id(to),
        edge_type,
        label: None,
    }
}

fn extract_strings(expr: &hcl::Expression) -> Option<Vec<String>> {
    match HclValue::from(expr.clone()) {
        HclValue::Array(arr) => {
            let mut values = Vec::new();
            for v in arr {
                if let Some(s) = v.as_str() {
                    values.push(s.to_string());
                }
            }
            Some(values)
        }
        HclValue::String(s) => Some(vec![s]),
        _ => None,
    }
}

fn extract_value(expr: &hcl::Expression) -> Option<String> {
    Some(value_to_string(&HclValue::from(expr.clone())))
}

fn value_to_string(val: &HclValue) -> String {
    match val {
        HclValue::String(s) => s.to_string(),
        HclValue::Number(n) => n.to_string(),
        HclValue::Bool(b) => b.to_string(),
        HclValue::Array(arr) => arr
            .iter()
            .map(value_to_string)
            .collect::<Vec<_>>()
            .join(","),
        HclValue::Object(map) => serde_json::to_string(map).unwrap_or_default(),
        _ => format!("{val:?}"),
    }
}

fn yaml_to_string(val: &YamlValue) -> String {
    match val {
        YamlValue::String(s) => s.clone(),
        YamlValue::Number(n) => n.to_string(),
        YamlValue::Bool(b) => b.to_string(),
        YamlValue::Sequence(seq) => seq.iter().map(yaml_to_string).collect::<Vec<_>>().join(","),
        YamlValue::Mapping(_) => serde_yaml::to_string(val).unwrap_or_default(),
        _ => format!("{val:?}"),
    }
}

struct CdkPyVisitor<'a> {
    file: &'a str,
    resources: Vec<ResourceNode>,
}

impl<'a> CdkPyVisitor<'a> {
    fn new(file: &'a str) -> Self {
        Self {
            file,
            resources: Vec::new(),
        }
    }

    fn push_resource(
        &mut self,
        module: String,
        construct: String,
        name: String,
        mut props: HashMap<String, String>,
    ) {
        let id = slugify_id(&format!("{module}.{name}"));
        let mut node =
            ResourceNode::new(id, ResourceType::from_raw(&module), name.clone(), self.file);
        node.properties
            .insert("construct".into(), construct.clone());
        enrich_construct_props(&construct, &mut props);
        node.properties.extend(props.drain());
        self.resources.push(node);
    }
}

impl<'a> rustpython_ast::Visitor for CdkPyVisitor<'a> {
    fn visit_expr_call(&mut self, node: rustpython_ast::ExprCall) {
        if let rustpython_ast::Expr::Attribute(ref attr) = *node.func {
            let construct = attr.attr.to_string();
            let base = match *attr.value.clone() {
                rustpython_ast::Expr::Name(ref name) => name.id.to_string(),
                rustpython_ast::Expr::Attribute(ref inner) => inner.attr.to_string(),
                _ => "cdk".to_string(),
            };
            let is_cdk_construct = matches!(
                construct.as_str(),
                "Bucket"
                    | "Table"
                    | "Function"
                    | "Queue"
                    | "Topic"
                    | "Api"
                    | "Stack"
                    | "RestApi"
                    | "HttpApi"
                    | "StateMachine"
            );
            if is_cdk_construct {
                let mut props = HashMap::new();
                for kw in &node.keywords {
                    if let Some(key) = kw.arg.as_ref().map(|s| s.to_string()) {
                        if let rustpython_ast::Expr::Constant(c) = &kw.value {
                            if let rustpython_ast::Constant::Str(s) = &c.value {
                                props.insert(key, s.to_string());
                            }
                        }
                    }
                }
                let name = node
                    .args
                    .get(1)
                    .and_then(|arg| match arg {
                        rustpython_ast::Expr::Constant(c) => match &c.value {
                            rustpython_ast::Constant::Str(s) => Some(s.to_string()),
                            _ => None,
                        },
                        _ => None,
                    })
                    .unwrap_or_else(|| construct.clone());
                self.push_resource(base.clone(), construct.to_string(), name, props);
            }
        }
        self.generic_visit_expr_call(node);
    }
}

struct CdkTsVisitor {
    file: String,
    resources: Vec<ResourceNode>,
}

impl CdkTsVisitor {
    fn new(file: String) -> Self {
        Self {
            file,
            resources: Vec::new(),
        }
    }

    fn record(
        &mut self,
        type_name: String,
        construct: String,
        name: String,
        mut props: HashMap<String, String>,
    ) {
        let id = slugify_id(&format!("{type_name}.{name}"));
        let mut node = ResourceNode::new(
            id,
            ResourceType::from_raw(&type_name),
            name.clone(),
            &self.file,
        );
        node.properties
            .insert("construct".into(), construct.clone());
        enrich_construct_props(&construct, &mut props);
        node.properties.extend(props.drain());
        self.resources.push(node);
    }

    fn member_to_string(&self, expr: &swc_ecma_ast::MemberExpr) -> String {
        let mut parts = Vec::new();
        let mut current = expr;
        loop {
            if let swc_ecma_ast::MemberProp::Ident(id) = &current.prop {
                parts.push(id.sym.to_string());
            }
            match &*current.obj {
                Expr::Member(inner) => current = inner,
                Expr::Ident(ident) => {
                    parts.push(ident.sym.to_string());
                    break;
                }
                _ => break,
            }
        }
        parts.into_iter().rev().collect::<Vec<_>>().join(".")
    }

    fn extract_name(&self, args: &[swc_ecma_ast::ExprOrSpread]) -> Option<String> {
        if args.len() > 1 {
            if let Some(name) = literal_to_string(&args[1].expr) {
                return Some(name);
            }
        }
        if let Some(first) = args.first() {
            return literal_to_string(&first.expr);
        }
        None
    }

    fn extract_props(&self, args: &[swc_ecma_ast::ExprOrSpread]) -> HashMap<String, String> {
        // Typical CDK new Construct(this, 'Id', { ...props })
        if args.len() < 3 {
            return HashMap::new();
        }
        fn flatten_object(expr: &Expr, map: &mut HashMap<String, String>, prefix: Option<&str>) {
            if let Expr::Object(obj) = expr {
                for prop in &obj.props {
                    if let swc_ecma_ast::PropOrSpread::Prop(p) = prop {
                        if let swc_ecma_ast::Prop::KeyValue(kv) = &**p {
                            if let Some(key) = match &kv.key {
                                PropName::Ident(id) => Some(id.sym.to_string()),
                                PropName::Str(s) => Some(s.value.to_string_lossy().into_owned()),
                                _ => None,
                            } {
                                let full_key = if let Some(pref) = prefix {
                                    format!("{pref}.{key}")
                                } else {
                                    key
                                };
                                if let Some(val) = literal_to_string(&kv.value) {
                                    map.insert(full_key, val);
                                } else if let Expr::Object(_) = &*kv.value {
                                    flatten_object(&kv.value, map, Some(&full_key));
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut map = HashMap::new();
        flatten_object(&args[2].expr, &mut map, None);
        map
    }
}

impl Visit for CdkTsVisitor {
    noop_visit_type!();

    fn visit_new_expr(&mut self, node: &NewExpr) {
        let callee = &node.callee;
        if let Expr::Member(member) = &**callee {
            let construct = if let swc_ecma_ast::MemberProp::Ident(id) = &member.prop {
                id.sym.to_string()
            } else {
                "".into()
            };
            let type_name = self.member_to_string(member);
            let is_construct = matches!(
                construct.as_str(),
                "Bucket"
                    | "Function"
                    | "Table"
                    | "Queue"
                    | "Topic"
                    | "Stack"
                    | "Api"
                    | "RestApi"
                    | "HttpApi"
                    | "StateMachine"
                    | "Distribution"
                    | "UserPool"
                    | "GraphqlApi"
            );
            if is_construct {
                let name = node
                    .args
                    .as_ref()
                    .and_then(|args| self.extract_name(args))
                    .unwrap_or_else(|| construct.clone());
                let props = node
                    .args
                    .as_ref()
                    .map(|args| self.extract_props(args))
                    .unwrap_or_default();
                self.record(type_name, construct, name, props);
            }
        }
        node.visit_children_with(self);
    }

    fn visit_call_expr(&mut self, node: &CallExpr) {
        node.visit_children_with(self);
    }
}

fn literal_to_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(swc_ecma_ast::Lit::Str(s)) => Some(s.value.to_string_lossy().to_string()),
        Expr::Lit(swc_ecma_ast::Lit::Num(n)) => Some(n.value.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn parses_basic_terraform_resource() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("main.tf");
        fs::write(
            &file,
            r#"
resource "aws_s3_bucket" "main" {
  bucket = "example"
  depends_on = ["aws_iam_role.role"]
}
"#,
        )
        .unwrap();
        let result = parse_terraform(&file, "main.tf");
        assert_eq!(result.resources.len(), 1);
        assert_eq!(result.edges.len(), 1);
    }

    #[test]
    fn parses_cloudformation_resource() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("template.yml");
        fs::write(
            &file,
            r#"
Resources:
  MyBucket:
    Type: AWS::S3::Bucket
    DependsOn:
      - Other
"#,
        )
        .unwrap();
        let result = parse_cloudformation(&file, "template.yml");
        assert_eq!(result.resources.len(), 1);
        assert_eq!(result.edges.len(), 1);
    }
}
