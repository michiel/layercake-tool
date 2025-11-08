use anyhow::{anyhow, Result as AnyResult};
use async_graphql::*;
use serde::{Deserialize, Serialize};

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

    pub fn apply_to(&self, graph: &mut Graph) -> AnyResult<bool> {
        if matches!(self.kind, GraphTransformKind::AggregateEdges) {
            if self.is_enabled() {
                graph.aggregate_edges();
            }
            return Ok(true);
        }

        if !self.is_enabled() {
            return Ok(false);
        }

        match self.kind {
            GraphTransformKind::PartitionDepthLimit => {
                let depth = self.params.max_partition_depth.ok_or_else(|| {
                    anyhow!("PartitionDepthLimit transform requires max_partition_depth")
                })?;
                if depth <= 0 {
                    return Err(anyhow!("max_partition_depth must be greater than zero"));
                }
                graph
                    .modify_graph_limit_partition_depth(depth)
                    .map_err(|e| anyhow!(e))?;
            }
            GraphTransformKind::PartitionWidthLimit => {
                let width = self.params.max_partition_width.ok_or_else(|| {
                    anyhow!("PartitionWidthLimit transform requires max_partition_width")
                })?;
                if width <= 0 {
                    return Err(anyhow!("max_partition_width must be greater than zero"));
                }
                graph
                    .modify_graph_limit_partition_width(width)
                    .map_err(|e| anyhow!(e))?;
            }
            GraphTransformKind::NodeLabelMaxLength => {
                let length = self.params.node_label_max_length.ok_or_else(|| {
                    anyhow!("NodeLabelMaxLength transform requires node_label_max_length")
                })?;
                if length == 0 {
                    return Err(anyhow!("node_label_max_length must be greater than zero"));
                }
                graph.truncate_node_labels(length);
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
            }
            GraphTransformKind::EdgeLabelMaxLength => {
                let length = self.params.edge_label_max_length.ok_or_else(|| {
                    anyhow!("EdgeLabelMaxLength transform requires edge_label_max_length")
                })?;
                if length == 0 {
                    return Err(anyhow!("edge_label_max_length must be greater than zero"));
                }
                graph.truncate_edge_labels(length);
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
            }
            GraphTransformKind::InvertGraph => {
                *graph = graph.invert_graph().map_err(|e| anyhow!(e))?;
            }
            GraphTransformKind::GenerateHierarchy => {
                // Placeholder – hierarchy generation handled during export using GraphConfig
            }
            GraphTransformKind::AggregateEdges => {
                // Handled above
                unreachable!("AggregateEdges should have been handled earlier");
            }
        }

        Ok(false)
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum GraphTransformKind {
    PartitionDepthLimit,
    PartitionWidthLimit,
    NodeLabelMaxLength,
    NodeLabelInsertNewlines,
    EdgeLabelMaxLength,
    EdgeLabelInsertNewlines,
    InvertGraph,
    GenerateHierarchy,
    AggregateEdges,
}

#[derive(SimpleObject, InputObject, Clone, Debug, Default, Serialize, Deserialize)]
#[graphql(input_name = "GraphTransformParamsInput")]
pub struct GraphTransformParams {
    pub max_partition_depth: Option<i32>,
    pub max_partition_width: Option<i32>,
    pub node_label_max_length: Option<usize>,
    pub node_label_insert_newlines_at: Option<usize>,
    pub edge_label_max_length: Option<usize>,
    pub edge_label_insert_newlines_at: Option<usize>,
    pub enabled: Option<bool>,
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

        let mut aggregate_handled = false;
        for transform in &self.transforms {
            let handled = transform.apply_to(graph)?;
            if handled {
                aggregate_handled = true;
            }
        }

        if !aggregate_handled {
            graph.aggregate_edges();
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
                    datasource: None,
                },
                Node {
                    id: "B".to_string(),
                    label: "Beta".to_string(),
                    layer: "layer1".to_string(),
                    is_partition: false,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                    datasource: None,
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
                    datasource: None,
                },
                Edge {
                    id: "e2".to_string(),
                    source: "A".to_string(),
                    target: "B".to_string(),
                    label: "EdgeLabelLong".to_string(),
                    layer: "layer1".to_string(),
                    weight: 1,
                    comment: None,
                    datasource: None,
                },
            ],
            layers: vec![Layer {
                id: "layer1".to_string(),
                label: "Layer 1".to_string(),
                background_color: "FFFFFF".to_string(),
                text_color: "000000".to_string(),
                border_color: "000000".to_string(),
                datasource: None,
            }],
        }
    }

    #[test]
    fn apply_transforms_runs_default_aggregate() {
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

        assert_eq!(graph.nodes[0].label, "Alp");
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].weight, 2);
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
}
