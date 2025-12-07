use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::model::{GraphEdge, ResourceNode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureGraph {
    pub root_id: String,
    pub root_label: String,
    pub partitions: HashMap<String, PartitionNode>,
    pub resources: HashMap<String, ResourceNode>,
    pub edges: Vec<GraphEdge>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionNode {
    pub id: String,
    pub label: String,
    pub parent: Option<String>,
    pub comment: Option<String>,
}

impl InfrastructureGraph {
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();
        let root_id = slugify_id(&label);
        let mut partitions = HashMap::new();
        partitions.insert(
            root_id.clone(),
            PartitionNode {
                id: root_id.clone(),
                label: label.clone(),
                parent: None,
                comment: None,
            },
        );

        Self {
            root_id,
            root_label: label,
            partitions,
            resources: HashMap::new(),
            edges: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn ensure_partition(
        &mut self,
        label: impl AsRef<str>,
        parent: Option<String>,
        comment: Option<String>,
    ) -> String {
        let base = format!("partition_{}", label.as_ref());
        let id = slugify_id(&base);
        if let Some(existing) = self.partitions.get(&id) {
            return existing.id.clone();
        }

        let parent_id = parent.unwrap_or_else(|| self.root_id.clone());
        self.partitions.insert(
            id.clone(),
            PartitionNode {
                id: id.clone(),
                label: label.as_ref().to_string(),
                parent: Some(parent_id),
                comment,
            },
        );
        id
    }

    pub fn add_resource(&mut self, mut node: ResourceNode) {
        if node.belongs_to.is_none() {
            node.belongs_to = Some(self.root_id.clone());
        }

        let id = slugify_id(&node.id);
        node.id = id.clone();
        self.resources.insert(id, node);
    }

    pub fn add_edge(&mut self, edge: GraphEdge) {
        self.edges.push(edge);
    }

    pub fn validate_edges(&mut self) {
        let resource_ids: HashSet<&String> = self.resources.keys().collect();
        let valid_partitions: HashSet<&String> = self.partitions.keys().collect();
        self.edges.retain(|edge| {
            let from_ok =
                resource_ids.contains(&edge.from) || valid_partitions.contains(&edge.from);
            let to_ok = resource_ids.contains(&edge.to) || valid_partitions.contains(&edge.to);
            if !from_ok || !to_ok {
                self.diagnostics.push(format!(
                    "Dropped edge {} -> {} (missing endpoint)",
                    edge.from, edge.to
                ));
                return false;
            }
            true
        });
    }
}

pub fn slugify_id(input: &str) -> String {
    let mut slug = String::with_capacity(input.len());
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else {
            slug.push('_');
        }
    }
    while slug.contains("__") {
        slug = slug.replace("__", "_");
    }
    slug.trim_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::model::{EdgeType, GraphEdge, ResourceNode, ResourceType};

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify_id("My-Resource.ID"), "my_resource_id");
        assert_eq!(slugify_id("__weird__id__"), "weird_id");
    }

    #[test]
    fn graph_adds_partitions() {
        let mut graph = InfrastructureGraph::new("Infra Root");
        let part = graph.ensure_partition("services/api", None, None);
        let mut node = ResourceNode::new(
            "aws_s3_bucket_main",
            ResourceType::Aws("aws_s3_bucket".into()),
            "main",
            "path.tf",
        );
        node.belongs_to = Some(part.clone());
        graph.add_resource(node);
        graph.add_edge(GraphEdge {
            from: part.clone(),
            to: "aws_s3_bucket_main".into(),
            edge_type: EdgeType::DependsOn,
            label: None,
        });
        graph.validate_edges();
        assert!(graph.partitions.contains_key(&part));
        assert!(graph.resources.contains_key("aws_s3_bucket_main"));
        assert_eq!(graph.edges.len(), 1);
    }
}
