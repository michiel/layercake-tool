use anyhow::Result;
use ignore::WalkBuilder;
use std::path::Path;
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

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                warn!("Skipping infra entry: {err}");
                continue;
            }
        };

        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }

        let ext = entry
            .path()
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        let relative = entry
            .path()
            .strip_prefix(&root)
            .unwrap_or_else(|_| entry.path())
            .to_string_lossy()
            .to_string();

        let scan = match ext.as_str() {
            "tf" => parse_terraform(entry.path(), &relative),
            "yaml" | "yml" => parse_cloudformation(entry.path(), &relative),
            "bicep" => parse_bicep(entry.path(), &relative),
            "ts" | "tsx" => parse_cdk_typescript(entry.path(), &relative),
            "py" => parse_cdk_python(entry.path(), &relative),
            _ => InfraScanResult::default(),
        };

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

    graph.validate_edges();
    graph.diagnostics.extend(diagnostics);
    Ok(graph)
}

fn parse_terraform(_path: &Path, relative: &str) -> InfraScanResult {
    InfraScanResult {
        diagnostics: vec![format!("Terraform parsing not implemented for {relative}")],
        ..Default::default()
    }
}

fn parse_cloudformation(_path: &Path, relative: &str) -> InfraScanResult {
    InfraScanResult {
        diagnostics: vec![format!(
            "CloudFormation parsing not implemented for {relative}"
        )],
        ..Default::default()
    }
}

fn parse_bicep(_path: &Path, relative: &str) -> InfraScanResult {
    InfraScanResult {
        diagnostics: vec![format!("Bicep parsing not implemented for {relative}")],
        ..Default::default()
    }
}

fn parse_cdk_python(_path: &Path, relative: &str) -> InfraScanResult {
    InfraScanResult {
        diagnostics: vec![format!("CDK Python parsing not implemented for {relative}")],
        ..Default::default()
    }
}

fn parse_cdk_typescript(_path: &Path, relative: &str) -> InfraScanResult {
    InfraScanResult {
        diagnostics: vec![format!(
            "CDK TypeScript parsing not implemented for {relative}"
        )],
        ..Default::default()
    }
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
