use anyhow::{anyhow, Result as AnyResult};
use async_graphql::*;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::graph::Graph;

// Transform Node Configuration
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize)]
#[graphql(input_name = "TransformNodeConfigInput")]
pub struct TransformNodeConfig {
    pub transforms: Vec<GraphTransform>,
}

/// Custom deserializer that handles migration from legacy schema v1 to current schema v2.
///
/// Supports three input formats:
/// 1. Current (v2): `{ transforms: [...] }` - Array of transforms
/// 2. Legacy (v1): `{ transformType: "...", transformConfig: {...} }` - Single transform with config
/// 3. Empty: `{}` - No transforms (defaults to empty array)
///
/// See docs/NODE_CONFIG_MIGRATION.md for detailed migration logic and examples.
impl<'de> Deserialize<'de> for TransformNodeConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = TransformNodeConfigWire::deserialize(deserializer)?;

        if let Some(transforms) = wire.transforms {
            // Current schema v2: array of transforms
            Ok(TransformNodeConfig {
                transforms: transforms
                    .into_iter()
                    .map(GraphTransform::with_default_enabled)
                    .collect(),
            })
        } else if let Some(transform_type) = wire.transform_type {
            // Legacy schema v1: migrate single transform to array
            let legacy_config = wire.transform_config.unwrap_or_default();
            let transforms = legacy_config.into_graph_transforms(transform_type);
            Ok(TransformNodeConfig { transforms })
        } else {
            // Empty config: no transforms
            Ok(TransformNodeConfig {
                transforms: Vec::new(),
            })
        }
    }
}

#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "GraphTransformInput")]
pub struct GraphTransform {
    pub kind: GraphTransformKind,
    #[serde(default)]
    #[graphql(default)]
    pub params: GraphTransformParams,
}

impl GraphTransform {
    fn with_default_enabled(self) -> Self {
        let mut transform = self;
        if transform.params.enabled.is_none() {
            transform.params.enabled = Some(true);
        }
        transform
    }

    pub fn is_enabled(&self) -> bool {
        self.params.enabled.unwrap_or(true)
    }

    pub fn apply_to(&self, graph: &mut Graph) -> AnyResult<Option<String>> {
        if matches!(self.kind, GraphTransformKind::AggregateEdges) {
            if self.is_enabled() {
                let before = graph.edges.len();
                graph.aggregate_edges();
                let msg = format!(
                    "### Transform: Aggregate Edges\n- Before: {} edges\n- After: {} edges",
                    before,
                    graph.edges.len()
                );
                return Ok(Some(msg));
            }
            return Ok(None);
        }

        if !self.is_enabled() {
            return Ok(None);
        }

        let before_nodes = graph.nodes.len();
        let before_edges = graph.edges.len();

        let annotation = match self.kind {
            GraphTransformKind::PartitionDepthLimit => {
                let depth = self.params.max_partition_depth.ok_or_else(|| {
                    anyhow!("PartitionDepthLimit transform requires max_partition_depth")
                })?;
                if depth <= 0 {
                    return Err(anyhow!("max_partition_depth must be greater than zero"));
                }
                let synthesized = graph.ensure_partition_hierarchy();
                graph
                    .modify_graph_limit_partition_depth(depth)
                    .map_err(|e| anyhow!(e))?;
                if synthesized {
                    info!("PartitionDepthLimit synthesized hierarchy because no partitions were defined in the source graph");
                }
                Some(format!(
                    "### Transform: Partition Depth Limit\n- Max depth: {}\n- Nodes after: {}\n- Edges after: {}",
                    depth,
                    graph.nodes.len(),
                    graph.edges.len()
                ))
            }
            GraphTransformKind::PartitionWidthLimit => {
                let width = self.params.max_partition_width.ok_or_else(|| {
                    anyhow!("PartitionWidthLimit transform requires max_partition_width")
                })?;
                if width <= 0 {
                    return Err(anyhow!("max_partition_width must be greater than zero"));
                }
                let synthesized = graph.ensure_partition_hierarchy();
                let summary = graph
                    .modify_graph_limit_partition_width(width)
                    .map_err(|e| anyhow!(e))?;
                if synthesized {
                    info!("PartitionWidthLimit synthesized hierarchy because no partitions were defined in the source graph");
                }

                let annotation = if summary.is_empty() {
                    format!(
                        "### Transform: Partition Width Limit\n- Max width: {}\n- No aggregation necessary",
                        width
                    )
                } else {
                    let mut table = String::from("| Parent | Retained | Aggregated | New aggregate |\n| --- | --- | --- | --- |\n");
                    let mut details = String::new();
                    for entry in &summary {
                        table.push_str(&format!(
                            "| {} ({}) | {} | {} | {} ({}) |\n",
                            entry.parent_id,
                            entry.parent_label,
                            entry.retained_count,
                            entry.aggregated_nodes.len(),
                            entry.aggregate_node_id,
                            entry.aggregate_node_label
                        ));

                        let aggregated_list = entry
                            .aggregated_nodes
                            .iter()
                            .map(|(id, label)| format!("{}:{}", id, label))
                            .collect::<Vec<_>>()
                            .join(", ");
                        details.push_str(&format!(
                            "- **{} ({})** aggregated: {}\n",
                            entry.aggregate_node_id, entry.aggregate_node_label, aggregated_list
                        ));
                    }

                    format!(
                        "### Transform: Partition Width Limit\n- Max width: {}\n- Parents aggregated: {}\n\n{}\n\n#### Aggregated nodes\n{}",
                        width,
                        summary.len(),
                        table,
                        details
                    )
                };

                Some(annotation)
            }
            GraphTransformKind::DropUnconnectedNodes => {
                let removed = graph.drop_unconnected_nodes();
                Some(format!(
                    "### Transform: Drop Unconnected Nodes\n- Removed: {} nodes\n- Nodes after: {}\n- Edges after: {}",
                    removed,
                    graph.nodes.len(),
                    graph.edges.len()
                ))
            }
            GraphTransformKind::NodeLabelMaxLength => {
                let length = self.params.node_label_max_length.ok_or_else(|| {
                    anyhow!("NodeLabelMaxLength transform requires node_label_max_length")
                })?;
                if length == 0 {
                    return Err(anyhow!("node_label_max_length must be greater than zero"));
                }
                graph.truncate_node_labels(length);
                Some(format!(
                    "### Transform: Node Label Max Length\n- Max length: {}\n- Nodes after: {}",
                    length,
                    graph.nodes.len()
                ))
            }
            GraphTransformKind::NodeLabelInsertNewlines => {
                let wrap_at = self.params.node_label_insert_newlines_at.ok_or_else(|| {
                    anyhow!(
                        "NodeLabelInsertNewlines transform requires node_label_insert_newlines_at"
                    )
                })?;
                if wrap_at == 0 {
                    return Err(anyhow!(
                        "node_label_insert_newlines_at must be greater than zero"
                    ));
                }
                graph.insert_newlines_in_node_labels(wrap_at);
                Some(format!(
                    "### Transform: Node Label Insert Newlines\n- Wrap column: {}\n- Nodes after: {}",
                    wrap_at,
                    graph.nodes.len()
                ))
            }
            GraphTransformKind::EdgeLabelMaxLength => {
                let length = self.params.edge_label_max_length.ok_or_else(|| {
                    anyhow!("EdgeLabelMaxLength transform requires edge_label_max_length")
                })?;
                if length == 0 {
                    return Err(anyhow!("edge_label_max_length must be greater than zero"));
                }
                graph.truncate_edge_labels(length);
                Some(format!(
                    "### Transform: Edge Label Max Length\n- Max length: {}\n- Edges after: {}",
                    length,
                    graph.edges.len()
                ))
            }
            GraphTransformKind::EdgeLabelInsertNewlines => {
                let wrap_at = self.params.edge_label_insert_newlines_at.ok_or_else(|| {
                    anyhow!(
                        "EdgeLabelInsertNewlines transform requires edge_label_insert_newlines_at"
                    )
                })?;
                if wrap_at == 0 {
                    return Err(anyhow!(
                        "edge_label_insert_newlines_at must be greater than zero"
                    ));
                }
                graph.insert_newlines_in_edge_labels(wrap_at);
                Some(format!(
                    "### Transform: Edge Label Insert Newlines\n- Wrap column: {}\n- Edges after: {}",
                    wrap_at,
                    graph.edges.len()
                ))
            }
            GraphTransformKind::InvertGraph => {
                *graph = graph.invert_graph().map_err(|e| anyhow!(e))?;
                Some(format!(
                    "### Transform: Invert Graph\n- Nodes before: {}\n- Edges before: {}\n- Nodes after: {}\n- Edges after: {}",
                    before_nodes,
                    before_edges,
                    graph.nodes.len(),
                    graph.edges.len()
                ))
            }
            GraphTransformKind::GenerateHierarchy => {
                graph.generate_hierarchy();
                Some(format!(
                    "### Transform: Generate Hierarchy\n- Nodes after: {}\n- Edges after: {}",
                    graph.nodes.len(),
                    graph.edges.len()
                ))
            }
            GraphTransformKind::AggregateLayerNodes => {
                let threshold = self.params.layer_connections_threshold.unwrap_or(3);
                if threshold == 0 {
                    return Err(anyhow!(
                        "AggregateLayerNodes transform requires layerConnectionsThreshold greater than zero"
                    ));
                }
                let summary = graph
                    .aggregate_nodes_by_layer(threshold)
                    .map_err(|e| anyhow!(e))?;
                let annotation = if summary.is_empty() {
                    format!(
                        "### Transform: Aggregate Nodes by Layer\n- Threshold: {}\n- No aggregations performed",
                        threshold
                    )
                } else {
                    let mut table = String::from(
                        "| Layer | Common node | Aggregated | New node |\n| --- | --- | --- | --- |\n",
                    );
                    let mut details = String::new();
                    for entry in &summary {
                        let anchor = entry
                            .anchor_node_label
                            .as_deref()
                            .unwrap_or(&entry.anchor_node_id);
                        table.push_str(&format!(
                            "| {} | {} | {} | {} |\n",
                            entry.layer_id,
                            anchor,
                            entry.aggregated_nodes.len(),
                            entry.aggregate_node_label
                        ));
                        let aggregated_list = entry
                            .aggregated_nodes
                            .iter()
                            .map(|(id, label)| format!("{}:{}", id, label))
                            .collect::<Vec<_>>()
                            .join(", ");
                        details.push_str(&format!(
                            "- **{}** condensed {} nodes via {}: {}\n",
                            entry.aggregate_node_label,
                            entry.aggregated_nodes.len(),
                            anchor,
                            aggregated_list
                        ));
                    }

                    format!(
                        "### Transform: Aggregate Nodes by Layer\n- Threshold: {}\n- Groups aggregated: {}\n\n{}\n\n{}",
                        threshold,
                        summary.len(),
                        table,
                        details
                    )
                };
                Some(annotation)
            }
            GraphTransformKind::AggregateEdges => {
                unreachable!("AggregateEdges should have been handled earlier")
            }
        };

        Ok(annotation)
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum GraphTransformKind {
    PartitionDepthLimit,
    PartitionWidthLimit,
    DropUnconnectedNodes,
    NodeLabelMaxLength,
    NodeLabelInsertNewlines,
    EdgeLabelMaxLength,
    EdgeLabelInsertNewlines,
    InvertGraph,
    GenerateHierarchy,
    AggregateLayerNodes,
    AggregateEdges,
}

#[derive(SimpleObject, InputObject, Clone, Debug, Default, Serialize, Deserialize)]
#[graphql(input_name = "GraphTransformParamsInput")]
#[serde(rename_all = "camelCase")]
pub struct GraphTransformParams {
    #[serde(alias = "max_partition_depth")]
    pub max_partition_depth: Option<i32>,
    #[serde(alias = "max_partition_width")]
    pub max_partition_width: Option<i32>,
    #[serde(alias = "node_label_max_length")]
    pub node_label_max_length: Option<usize>,
    #[serde(alias = "node_label_insert_newlines_at")]
    pub node_label_insert_newlines_at: Option<usize>,
    #[serde(alias = "edge_label_max_length")]
    pub edge_label_max_length: Option<usize>,
    #[serde(alias = "edge_label_insert_newlines_at")]
    pub edge_label_insert_newlines_at: Option<usize>,
    pub enabled: Option<bool>,
    #[serde(alias = "layerConnectionsThreshold")]
    pub layer_connections_threshold: Option<usize>,
}

/// Wire format for deserializing TransformNodeConfig supporting both v1 and v2 schemas.
///
/// This struct accepts multiple input formats for backward compatibility:
/// - `transforms`: Current v2 schema (array of transforms)
/// - `transformType` + `transformConfig`: Legacy v1 schema (single transform)
///
/// See NODE_CONFIG_MIGRATION.md for migration details.
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct TransformNodeConfigWire {
    #[serde(default)]
    transforms: Option<Vec<GraphTransform>>,
    #[serde(default)]
    #[serde(alias = "transformType")]
    transform_type: Option<LegacyTransformType>,
    #[serde(default)]
    #[serde(alias = "transformConfig")]
    transform_config: Option<LegacyTransformConfig>,
}

/// Legacy schema v1 transform types.
///
/// In v1, only one transform type could be specified per node. In v2, multiple
/// transforms can be combined in an array. This enum defines the v1 transform types.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum LegacyTransformType {
    PartitionDepthLimit,
    InvertGraph,
    FilterNodes,
    FilterEdges,
}

/// Legacy schema v1 transform configuration.
///
/// In v1, a single monolithic configuration object held parameters for all transforms.
/// The migration logic extracts relevant parameters based on the transform_type.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct LegacyTransformConfig {
    max_partition_depth: Option<i32>,
    max_partition_width: Option<i32>,
    generate_hierarchy: Option<bool>,
    invert_graph: Option<bool>,
    node_filter: Option<String>,
    edge_filter: Option<String>,
}

impl LegacyTransformConfig {
    /// Migrate legacy v1 configuration to v2 transform array.
    ///
    /// Extracts relevant parameters based on transform_type and converts to
    /// one or more GraphTransform instances. For example, PartitionDepthLimit
    /// might generate both a depth limit transform and a hierarchy transform.
    fn into_graph_transforms(self, transform_type: LegacyTransformType) -> Vec<GraphTransform> {
        let mut transforms = Vec::new();

        match transform_type {
            LegacyTransformType::PartitionDepthLimit => {
                if let Some(depth) = self.max_partition_depth.filter(|d| *d > 0) {
                    transforms.push(GraphTransform {
                        kind: GraphTransformKind::PartitionDepthLimit,
                        params: GraphTransformParams {
                            max_partition_depth: Some(depth),
                            enabled: Some(true),
                            ..Default::default()
                        },
                    });
                }
                if let Some(width) = self.max_partition_width.filter(|w| *w > 0) {
                    transforms.push(GraphTransform {
                        kind: GraphTransformKind::PartitionWidthLimit,
                        params: GraphTransformParams {
                            max_partition_width: Some(width),
                            enabled: Some(true),
                            ..Default::default()
                        },
                    });
                }
                if self.generate_hierarchy.unwrap_or(false) {
                    transforms.push(GraphTransform {
                        kind: GraphTransformKind::GenerateHierarchy,
                        params: GraphTransformParams {
                            enabled: Some(true),
                            ..Default::default()
                        },
                    });
                }
            }
            LegacyTransformType::InvertGraph => {
                if self.invert_graph.unwrap_or(true) {
                    transforms.push(GraphTransform {
                        kind: GraphTransformKind::InvertGraph,
                        params: GraphTransformParams {
                            enabled: Some(true),
                            ..Default::default()
                        },
                    });
                }
            }
            LegacyTransformType::FilterNodes | LegacyTransformType::FilterEdges => {}
        }

        // Legacy pipeline always aggregated edges; preserve unless explicitly disabled later
        transforms.push(GraphTransform {
            kind: GraphTransformKind::AggregateEdges,
            params: GraphTransformParams {
                enabled: Some(true),
                ..Default::default()
            },
        });

        transforms
    }
}

impl TransformNodeConfig {
    pub fn apply_transforms(&self, graph: &mut Graph) -> AnyResult<()> {
        if self.transforms.is_empty() {
            return Err(anyhow!(
                "TransformNode requires at least one transform to execute"
            ));
        }

        for transform in &self.transforms {
            if let Some(annotation) = transform.apply_to(graph)? {
                graph.append_annotation(annotation);
            }
        }

        Ok(())
    }

    pub fn to_graph_config(&self) -> crate::plan::GraphConfig {
        let mut config = crate::plan::GraphConfig::default();

        for transform in &self.transforms {
            if !transform.is_enabled() {
                continue;
            }
            match transform.kind {
                GraphTransformKind::PartitionDepthLimit => {
                    if let Some(depth) = transform.params.max_partition_depth {
                        config.max_partition_depth = depth;
                    }
                }
                GraphTransformKind::PartitionWidthLimit => {
                    if let Some(width) = transform.params.max_partition_width {
                        config.max_partition_width = width;
                    }
                }
                GraphTransformKind::DropUnconnectedNodes => {
                    config.drop_unconnected_nodes = transform.params.enabled.unwrap_or(true);
                }
                GraphTransformKind::NodeLabelMaxLength => {
                    if let Some(length) = transform.params.node_label_max_length {
                        config.node_label_max_length = length;
                    }
                }
                GraphTransformKind::NodeLabelInsertNewlines => {
                    if let Some(wrap) = transform.params.node_label_insert_newlines_at {
                        config.node_label_insert_newlines_at = wrap;
                    }
                }
                GraphTransformKind::EdgeLabelMaxLength => {
                    if let Some(length) = transform.params.edge_label_max_length {
                        config.edge_label_max_length = length;
                    }
                }
                GraphTransformKind::EdgeLabelInsertNewlines => {
                    if let Some(wrap) = transform.params.edge_label_insert_newlines_at {
                        config.edge_label_insert_newlines_at = wrap;
                    }
                }
                GraphTransformKind::InvertGraph => {
                    config.invert_graph = true;
                }
                GraphTransformKind::GenerateHierarchy => {
                    config.generate_hierarchy = true;
                }
                GraphTransformKind::AggregateLayerNodes => {}
                GraphTransformKind::AggregateEdges => {
                    config.aggregate_edges = transform.params.enabled.unwrap_or(true);
                }
            }
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Edge, Layer, Node};
    use std::collections::HashSet;

    fn sample_graph() -> Graph {
        Graph {
            name: "Sample".to_string(),
            nodes: vec![
                Node {
                    id: "A".to_string(),
                    label: "Alpha".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                    dataset: None,
                },
                Node {
                    id: "B".to_string(),
                    label: "Beta".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                    dataset: None,
                },
            ],
            edges: vec![
                Edge {
                    id: "e1".to_string(),
                    source: "A".to_string(),
                    target: "B".to_string(),
                    label: "EdgeLabelLong".to_string(),
                    layer: "layer1".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                },
                Edge {
                    id: "e2".to_string(),
                    source: "A".to_string(),
                    target: "B".to_string(),
                    label: "EdgeLabelLong".to_string(),
                    layer: "layer1".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                },
            ],
            layers: vec![Layer {
                id: "layer1".to_string(),
                label: "Layer 1".to_string(),
                background_color: "FFFFFF".to_string(),
                text_color: "000000".to_string(),
                border_color: "000000".to_string(),
                alias: None,
                dataset: None,
            }],
            annotations: None,
        }
    }

    #[test]
    fn apply_transforms_runs_aggregate_when_present() {
        let mut graph = sample_graph();
        let config = TransformNodeConfig {
            transforms: vec![
                GraphTransform {
                    kind: GraphTransformKind::NodeLabelMaxLength,
                    params: GraphTransformParams {
                        node_label_max_length: Some(3),
                        ..Default::default()
                    },
                },
                GraphTransform {
                    kind: GraphTransformKind::AggregateEdges,
                    params: GraphTransformParams::default(),
                },
            ],
        };

        config
            .apply_transforms(&mut graph)
            .expect("transform should succeed");

        assert_eq!(graph.nodes[0].label, "Alp");
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].weight, 2);
    }

    #[test]
    fn apply_transforms_skips_aggregate_when_not_requested() {
        let mut graph = sample_graph();
        let config = TransformNodeConfig {
            transforms: vec![GraphTransform {
                kind: GraphTransformKind::NodeLabelMaxLength,
                params: GraphTransformParams {
                    node_label_max_length: Some(3),
                    ..Default::default()
                },
            }],
        };

        config
            .apply_transforms(&mut graph)
            .expect("transform should succeed");

        assert_eq!(graph.edges.len(), 2);
    }

    #[test]
    fn apply_transforms_respects_disabled_aggregate() {
        let mut graph = sample_graph();
        let config = TransformNodeConfig {
            transforms: vec![GraphTransform {
                kind: GraphTransformKind::AggregateEdges,
                params: GraphTransformParams {
                    enabled: Some(false),
                    ..Default::default()
                },
            }],
        };

        config
            .apply_transforms(&mut graph)
            .expect("transform should succeed");

        assert_eq!(graph.edges.len(), 2);
    }

    #[test]
    fn apply_transforms_errors_without_required_params() {
        let mut graph = sample_graph();
        let config = TransformNodeConfig {
            transforms: vec![GraphTransform {
                kind: GraphTransformKind::PartitionDepthLimit,
                params: GraphTransformParams::default(),
            }],
        };

        let err = config.apply_transforms(&mut graph).unwrap_err();
        assert!(
            err.to_string().contains("max_partition_depth"),
            "expected missing parameter error, got {err}"
        );
    }

    #[test]
    fn apply_transforms_requires_non_empty_config() {
        let mut graph = sample_graph();
        let config = TransformNodeConfig { transforms: vec![] };
        assert!(config.apply_transforms(&mut graph).is_err());
    }

    #[test]
    fn generate_hierarchy_creates_root_and_edges() {
        let mut graph = Graph {
            name: "HierarchyGraph".to_string(),
            nodes: vec![
                Node {
                    id: "root".to_string(),
                    label: "Root".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: true,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                    dataset: None,
                },
                Node {
                    id: "child".to_string(),
                    label: "Child".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: Some("root".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                },
                Node {
                    id: "leaf".to_string(),
                    label: "Leaf".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: Some("child".to_string()),
                    weight: 1,
                    comment: None,
                    dataset: None,
                },
            ],
            edges: vec![
                Edge {
                    id: "old1".to_string(),
                    source: "root".to_string(),
                    target: "child".to_string(),
                    label: "old".to_string(),
                    layer: "layer1".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                },
                Edge {
                    id: "old2".to_string(),
                    source: "child".to_string(),
                    target: "leaf".to_string(),
                    label: "old".to_string(),
                    layer: "layer1".to_string(),
                    weight: 1,
                    comment: None,
                    dataset: None,
                },
            ],
            layers: vec![Layer::new(
                "layer1", "Layer 1", "ffffff", "000000", "000000",
            )],
            annotations: None,
        };

        let transform = GraphTransform {
            kind: GraphTransformKind::GenerateHierarchy,
            params: GraphTransformParams::default(),
        };

        transform
            .apply_to(&mut graph)
            .expect("hierarchy transform should succeed");

        // Identify new hierarchy node
        let hierarchy_node = graph
            .nodes
            .iter()
            .find(|node| node.id.starts_with("hierarchy"))
            .expect("hierarchy node should exist");
        assert!(hierarchy_node.is_partition);
        assert_eq!(hierarchy_node.belongs_to.as_deref(), Some(""));

        // All other nodes now belong to the hierarchy node
        for node in graph.nodes.iter().filter(|n| n.id != hierarchy_node.id) {
            assert_eq!(
                node.belongs_to.as_deref(),
                Some(hierarchy_node.id.as_str()),
                "node {} should belong to hierarchy node",
                node.id
            );
            assert!(
                !node.is_partition,
                "node {} should no longer be marked as partition",
                node.id
            );
        }

        // Edges represent the former belongs_to relationships
        assert_eq!(graph.edges.len(), 2);
        let edge_pairs: HashSet<(String, String)> = graph
            .edges
            .iter()
            .map(|edge| (edge.source.clone(), edge.target.clone()))
            .collect();
        assert!(edge_pairs.contains(&("root".to_string(), "child".to_string())));
        assert!(edge_pairs.contains(&("child".to_string(), "leaf".to_string())));
    }

    #[test]
    fn transform_params_deserialize_camel_case() {
        let json = r#"{
            "transforms": [
                {
                    "kind": "PartitionDepthLimit",
                    "params": { "maxPartitionDepth": 3 }
                },
                {
                    "kind": "NodeLabelInsertNewlines",
                    "params": { "nodeLabelInsertNewlinesAt": 12 }
                }
            ]
        }"#;

        let config: TransformNodeConfig =
            serde_json::from_str(json).expect("camelCase params should deserialize");

        assert_eq!(
            config.transforms[0].params.max_partition_depth,
            Some(3),
            "maxPartitionDepth should map to max_partition_depth"
        );
        assert_eq!(
            config.transforms[1].params.node_label_insert_newlines_at,
            Some(12),
            "nodeLabelInsertNewlinesAt should map correctly"
        );
    }
}
