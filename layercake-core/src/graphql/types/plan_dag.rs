use anyhow::{anyhow, Context, Result as AnyResult};
use async_graphql::*;
use chrono::{DateTime, Utc};
use sea_orm::{ConnectionTrait, DatabaseConnection, FromQueryResult, Statement, Value};
use serde::{
    de::{self, Deserializer},
    Deserialize, Serialize,
};
use serde_json::json;
use std::collections::HashSet;
use tracing::warn;

use crate::app_context::PlanDagSnapshot;
use crate::database::entities::{data_sources, plan_dag_edges, plan_dag_nodes};
use crate::graph::Graph;
use crate::graphql::types::scalars::JSON;

// Position for ReactFlow nodes
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "PositionInput")]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

// Batch node move input
#[derive(InputObject, Clone, Debug)]
pub struct NodePositionInput {
    pub node_id: String,
    pub position: Position,
    #[graphql(name = "sourcePosition")]
    pub source_position: Option<String>,
    #[graphql(name = "targetPosition")]
    pub target_position: Option<String>,
}

// Node metadata
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "NodeMetadataInput")]
pub struct NodeMetadata {
    pub label: String,
    pub description: Option<String>,
}

// Edge metadata
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "EdgeMetadataInput")]
pub struct EdgeMetadata {
    pub label: Option<String>,
    #[graphql(name = "dataType")]
    pub data_type: DataType,
}

// Plan DAG Node Types (matching frontend enum)
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum PlanDagNodeType {
    #[graphql(name = "DataSourceNode")]
    DataSource,
    #[graphql(name = "GraphNode")]
    Graph,
    #[graphql(name = "TransformNode")]
    Transform,
    #[graphql(name = "FilterNode")]
    Filter,
    #[graphql(name = "MergeNode")]
    Merge,
    #[graphql(name = "CopyNode")]
    Copy,
    #[graphql(name = "OutputNode")]
    Output,
}

// Data type for edges
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum DataType {
    GraphData,
    GraphReference,
}

// Data Source Node Configuration
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "DataSourceNodeConfigInput")]
pub struct DataSourceNodeConfig {
    #[graphql(name = "dataSourceId")]
    pub data_source_id: Option<i32>, // Reference to DataSource entity (new)
    pub display_mode: Option<String>, // 'summary' | 'detailed' | 'preview'

    // Legacy fields (for backward compatibility)
    pub input_type: Option<InputType>,
    pub source: Option<String>,
    pub data_type: Option<InputDataType>,
    // Removed: output_graph_ref - output connections handled by visual edges in DAG
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum InputType {
    CsvNodesFromFile,
    CsvEdgesFromFile,
    CsvLayersFromFile,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum InputDataType {
    Nodes,
    Edges,
    Layers,
}

// Data Source Reference for dropdown selection
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct DataSourceReference {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub data_type: String,
    pub created_at: DateTime<Utc>,
}

impl From<data_sources::Model> for DataSourceReference {
    fn from(model: data_sources::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            data_type: model.data_type,
            created_at: model.created_at,
        }
    }
}

// Graph Node Configuration
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "GraphNodeConfigInput")]
pub struct GraphNodeConfig {
    // Removed: graph_id - graph connections handled by visual edges in DAG
    pub is_reference: bool,
    pub metadata: GraphNodeMetadata,
}

#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "GraphNodeMetadataInput")]
pub struct GraphNodeMetadata {
    pub node_count: Option<i32>,
    pub edge_count: Option<i32>,
    #[graphql(name = "lastModified")]
    pub last_modified: Option<String>,
}

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

// Filter Node Configuration
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize)]
#[graphql(input_name = "FilterNodeConfigInput")]
pub struct FilterNodeConfig {
    pub query: QueryFilterConfig,
}

/// Custom deserializer that handles migration from legacy schema v1 to current schema v2.
///
/// Supports two input formats:
/// 1. Current (v2): `{ query: {...} }` - Query builder configuration
/// 2. Legacy (v1): `{ filters: [{kind: "Query", params: {...}}] }` - Array of filters
///
/// Returns error if no valid query configuration is found in either format.
/// See docs/NODE_CONFIG_MIGRATION.md for detailed migration logic and examples.
impl<'de> Deserialize<'de> for FilterNodeConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct FilterNodeConfigWire {
            query: Option<QueryFilterConfig>,
            filters: Option<Vec<LegacyGraphFilter>>,
        }

        let wire = FilterNodeConfigWire::deserialize(deserializer)?;

        if let Some(query) = wire.query {
            // Current schema v2: query builder
            return Ok(Self {
                query: query.normalized(),
            });
        }

        if let Some(filters) = wire.filters {
            // Legacy schema v1: extract query from filters array
            for filter in filters {
                if filter.is_query() {
                    if let Some(query) = filter.params.and_then(|params| params.query_config) {
                        return Ok(Self {
                            query: query.normalized(),
                        });
                    }
                }
            }
        }

        Err(de::Error::custom(
            "FilterNodeConfig must include a query definition",
        ))
    }
}

pub struct FilterEvaluationContext<'a> {
    pub db: &'a DatabaseConnection,
    pub graph_id: i32,
}

impl FilterNodeConfig {
    pub async fn apply_filters(
        &self,
        graph: &mut Graph,
        context: &FilterEvaluationContext<'_>,
    ) -> AnyResult<()> {
        let normalized = self.query.normalized();
        query_filter_executor::apply_query_filter(graph, context, &normalized).await
    }
}

#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "QueryFilterConfigInput")]
#[serde(rename_all = "camelCase")]
pub struct QueryFilterConfig {
    pub targets: Vec<QueryFilterTarget>,
    pub mode: QueryFilterMode,
    #[graphql(name = "linkPruningMode")]
    pub link_pruning_mode: QueryLinkPruningMode,
    #[graphql(name = "ruleGroup")]
    pub rule_group: JSON,
    #[graphql(name = "fieldMetadataVersion")]
    pub field_metadata_version: String,
    pub notes: Option<String>,
}

impl QueryFilterConfig {
    pub fn normalized(&self) -> Self {
        let mut normalized = self.clone();
        if normalized.targets.is_empty() {
            normalized.targets = vec![QueryFilterTarget::Nodes];
        }
        if normalized.rule_group.is_null() {
            normalized.rule_group = default_rule_group();
        }
        if normalized.field_metadata_version.trim().is_empty() {
            normalized.field_metadata_version = "v1".to_string();
        }
        normalized
    }
}

fn default_rule_group() -> JSON {
    json!({
        "combinator": "and",
        "rules": []
    })
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QueryFilterTarget {
    #[graphql(name = "nodes")]
    Nodes,
    #[graphql(name = "edges")]
    Edges,
    #[graphql(name = "layers")]
    Layers,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QueryFilterMode {
    #[graphql(name = "include")]
    Include,
    #[graphql(name = "exclude")]
    Exclude,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QueryLinkPruningMode {
    #[graphql(name = "autoDropDanglingEdges")]
    AutoDropDanglingEdges,
    #[graphql(name = "retainEdges")]
    RetainEdges,
    #[graphql(name = "dropOrphanNodes")]
    DropOrphanNodes,
}

/// Legacy schema v1 filter format.
///
/// In v1, filters were specified as an array with a `kind` discriminator.
/// The "Query" kind contained the query builder configuration. This struct
/// is used during migration to extract query configurations from legacy plans.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyGraphFilter {
    kind: Option<String>,
    params: Option<LegacyGraphFilterParams>,
}

impl LegacyGraphFilter {
    /// Check if this legacy filter is a "Query" filter.
    ///
    /// Matches both "query" and "querytext" (case-insensitive) for
    /// backward compatibility with different v1 variants.
    fn is_query(&self) -> bool {
        self.kind
            .as_deref()
            .map(|k| k.eq_ignore_ascii_case("query") || k.eq_ignore_ascii_case("querytext"))
            .unwrap_or(false)
    }
}

/// Parameters for legacy schema v1 filters.
///
/// Contains the embedded queryConfig that needs to be extracted during migration.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyGraphFilterParams {
    #[serde(rename = "queryConfig")]
    query_config: Option<QueryFilterConfig>,
}

mod query_filter_executor {
    use super::*;
    use serde::Deserialize;

    pub async fn apply_query_filter(
        graph: &mut Graph,
        context: &FilterEvaluationContext<'_>,
        config: &QueryFilterConfig,
    ) -> AnyResult<()> {
        let rule_group: QueryRuleGroup = serde_json::from_value(config.rule_group.clone())
            .context("Invalid query builder configuration")?;

        let targets: Vec<QueryFilterTarget> = if config.targets.is_empty() {
            vec![QueryFilterTarget::Nodes]
        } else {
            config.targets.clone()
        };

        let mut applied = false;
        for target in targets {
            if let Some(fragment) = build_fragment_for_target(target, &rule_group) {
                let matches = execute_sql_for_target(context, target, fragment).await?;
                apply_matches_to_graph(graph, target, &matches, config.mode);
                applied = true;
            }
        }

        if !applied {
            warn!("Query filter had no applicable rules; skipping execution");
            return Ok(());
        }

        apply_link_pruning(graph, config.link_pruning_mode);

        Ok(())
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct QueryRuleGroup {
        combinator: QueryRuleCombinator,
        #[serde(default)]
        not: bool,
        #[serde(default)]
        rules: Vec<QueryRule>,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    enum QueryRuleCombinator {
        And,
        Or,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(untagged)]
    enum QueryRule {
        Group(Box<QueryRuleGroup>),
        Leaf(QueryRuleLeaf),
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct QueryRuleLeaf {
        field: String,
        operator: String,
        #[serde(default)]
        value: serde_json::Value,
    }

    #[derive(Debug, Clone)]
    struct SqlFragment {
        sql: String,
        params: Vec<Value>,
    }

    #[derive(Debug, Clone, Copy)]
    enum ColumnValueType {
        Text,
        Number,
        Boolean,
    }

    #[derive(Debug, Clone)]
    enum ColumnSelector {
        Column {
            sql: &'static str,
            value_type: ColumnValueType,
        },
        Json {
            column: &'static str,
            path: String,
        },
    }

    impl ColumnSelector {
        fn expression(&self) -> String {
            match self {
                ColumnSelector::Column { sql, .. } => sql.to_string(),
                ColumnSelector::Json { column, path } => {
                    format!("json_extract({column}, '$.{path}')")
                }
            }
        }

        fn value_type(&self) -> ColumnValueType {
            match self {
                ColumnSelector::Column { value_type, .. } => *value_type,
                ColumnSelector::Json { .. } => ColumnValueType::Text,
            }
        }
    }

    async fn execute_sql_for_target(
        context: &FilterEvaluationContext<'_>,
        target: QueryFilterTarget,
        fragment: SqlFragment,
    ) -> AnyResult<HashSet<String>> {
        #[derive(FromQueryResult)]
        struct EntityIdRow {
            entity_id: String,
        }

        let (table, id_column) = match target {
            QueryFilterTarget::Nodes => ("graph_nodes", "graph_nodes.id"),
            QueryFilterTarget::Edges => ("graph_edges", "graph_edges.id"),
            QueryFilterTarget::Layers => ("graph_layers", "graph_layers.layer_id"),
        };

        let sql = format!(
            "SELECT {id_column} AS entity_id FROM {table} WHERE graph_id = ? AND {condition}",
            id_column = id_column,
            table = table,
            condition = fragment.sql,
        );

        let mut params = vec![Value::from(context.graph_id)];
        params.extend(fragment.params);

        let stmt = Statement::from_sql_and_values(context.db.get_database_backend(), sql, params);

        let rows = EntityIdRow::find_by_statement(stmt).all(context.db).await?;
        Ok(rows.into_iter().map(|row| row.entity_id).collect())
    }

    fn build_fragment_for_target(
        target: QueryFilterTarget,
        group: &QueryRuleGroup,
    ) -> Option<SqlFragment> {
        build_fragment_for_group(group, target)
    }

    fn build_fragment_for_group(
        group: &QueryRuleGroup,
        target: QueryFilterTarget,
    ) -> Option<SqlFragment> {
        let mut fragments = Vec::new();
        for rule in &group.rules {
            match rule {
                QueryRule::Group(sub) => {
                    if let Some(fragment) = build_fragment_for_group(sub, target) {
                        fragments.push(fragment);
                    }
                }
                QueryRule::Leaf(leaf) => {
                    if let Some(fragment) = build_fragment_for_rule(leaf, target) {
                        fragments.push(fragment);
                    }
                }
            }
        }

        if fragments.is_empty() {
            return None;
        }

        let joiner = match group.combinator {
            QueryRuleCombinator::And => " AND ",
            QueryRuleCombinator::Or => " OR ",
        };

        let mut sql = String::new();
        let mut params = Vec::new();
        for (idx, fragment) in fragments.into_iter().enumerate() {
            if idx > 0 {
                sql.push_str(joiner);
            }
            sql.push_str(&fragment.sql);
            params.extend(fragment.params);
        }

        let mut wrapped = format!("({sql})");
        if group.not {
            wrapped = format!("NOT {wrapped}");
        }

        Some(SqlFragment {
            sql: wrapped,
            params,
        })
    }

    fn build_fragment_for_rule(
        rule: &QueryRuleLeaf,
        target: QueryFilterTarget,
    ) -> Option<SqlFragment> {
        let prefix = match target {
            QueryFilterTarget::Nodes => "node.",
            QueryFilterTarget::Edges => "edge.",
            QueryFilterTarget::Layers => "layer.",
        };

        if !rule.field.starts_with(prefix) {
            return None;
        }

        let selector = map_field(target, &rule.field[prefix.len()..])?;
        build_operator_fragment(&selector, &rule.operator, &rule.value)
    }

    fn map_field(target: QueryFilterTarget, field: &str) -> Option<ColumnSelector> {
        match target {
            QueryFilterTarget::Nodes => match field {
                "id" => Some(ColumnSelector::Column {
                    sql: "graph_nodes.id",
                    value_type: ColumnValueType::Text,
                }),
                "label" => Some(ColumnSelector::Column {
                    sql: "graph_nodes.label",
                    value_type: ColumnValueType::Text,
                }),
                "layer" => Some(ColumnSelector::Column {
                    sql: "graph_nodes.layer",
                    value_type: ColumnValueType::Text,
                }),
                "weight" => Some(ColumnSelector::Column {
                    sql: "graph_nodes.weight",
                    value_type: ColumnValueType::Number,
                }),
                "belongs_to" => Some(ColumnSelector::Column {
                    sql: "graph_nodes.belongs_to",
                    value_type: ColumnValueType::Text,
                }),
                "is_partition" => Some(ColumnSelector::Column {
                    sql: "graph_nodes.is_partition",
                    value_type: ColumnValueType::Boolean,
                }),
                "datasource_id" => Some(ColumnSelector::Column {
                    sql: "graph_nodes.data_source_id",
                    value_type: ColumnValueType::Number,
                }),
                "comment" => Some(ColumnSelector::Column {
                    sql: "graph_nodes.comment",
                    value_type: ColumnValueType::Text,
                }),
                _ if field.starts_with("attrs.") => {
                    sanitize_json_path(field.trim_start_matches("attrs.")).map(|path| {
                        ColumnSelector::Json {
                            column: "graph_nodes.attrs",
                            path,
                        }
                    })
                }
                _ => None,
            },
            QueryFilterTarget::Edges => match field {
                "id" => Some(ColumnSelector::Column {
                    sql: "graph_edges.id",
                    value_type: ColumnValueType::Text,
                }),
                "label" => Some(ColumnSelector::Column {
                    sql: "graph_edges.label",
                    value_type: ColumnValueType::Text,
                }),
                "source" => Some(ColumnSelector::Column {
                    sql: "graph_edges.source",
                    value_type: ColumnValueType::Text,
                }),
                "target" => Some(ColumnSelector::Column {
                    sql: "graph_edges.target",
                    value_type: ColumnValueType::Text,
                }),
                "layer" => Some(ColumnSelector::Column {
                    sql: "graph_edges.layer",
                    value_type: ColumnValueType::Text,
                }),
                "weight" => Some(ColumnSelector::Column {
                    sql: "graph_edges.weight",
                    value_type: ColumnValueType::Number,
                }),
                "datasource_id" => Some(ColumnSelector::Column {
                    sql: "graph_edges.data_source_id",
                    value_type: ColumnValueType::Number,
                }),
                _ if field.starts_with("attrs.") => {
                    sanitize_json_path(field.trim_start_matches("attrs.")).map(|path| {
                        ColumnSelector::Json {
                            column: "graph_edges.attrs",
                            path,
                        }
                    })
                }
                _ => None,
            },
            QueryFilterTarget::Layers => match field {
                "layer_id" => Some(ColumnSelector::Column {
                    sql: "graph_layers.layer_id",
                    value_type: ColumnValueType::Text,
                }),
                "name" => Some(ColumnSelector::Column {
                    sql: "graph_layers.name",
                    value_type: ColumnValueType::Text,
                }),
                "background_color" => Some(ColumnSelector::Column {
                    sql: "graph_layers.background_color",
                    value_type: ColumnValueType::Text,
                }),
                "text_color" => Some(ColumnSelector::Column {
                    sql: "graph_layers.text_color",
                    value_type: ColumnValueType::Text,
                }),
                "border_color" => Some(ColumnSelector::Column {
                    sql: "graph_layers.border_color",
                    value_type: ColumnValueType::Text,
                }),
                "datasource_id" => Some(ColumnSelector::Column {
                    sql: "graph_layers.data_source_id",
                    value_type: ColumnValueType::Number,
                }),
                _ if field.starts_with("properties.") => {
                    sanitize_json_path(field.trim_start_matches("properties.")).map(|path| {
                        ColumnSelector::Json {
                            column: "graph_layers.properties",
                            path,
                        }
                    })
                }
                _ => None,
            },
        }
    }

    fn sanitize_json_path(raw: &str) -> Option<String> {
        let segments: Vec<String> = raw
            .split('.')
            .map(|segment| {
                segment
                    .chars()
                    .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect::<String>()
            })
            .filter(|segment| !segment.is_empty())
            .collect();

        if segments.is_empty() {
            None
        } else {
            Some(segments.join("."))
        }
    }

    fn build_operator_fragment(
        selector: &ColumnSelector,
        operator: &str,
        raw_value: &serde_json::Value,
    ) -> Option<SqlFragment> {
        let expr = selector.expression();
        let column_type = selector.value_type();
        let op = operator.to_lowercase();

        match op.as_str() {
            "=" | "==" => parse_scalar_value(raw_value, column_type).map(|value| SqlFragment {
                sql: format!("({expr} = ?)", expr = expr),
                params: vec![value],
            }),
            "!=" | "<>" => parse_scalar_value(raw_value, column_type).map(|value| SqlFragment {
                sql: format!("({expr} != ?)", expr = expr),
                params: vec![value],
            }),
            "<" | "<=" | ">" | ">=" => {
                if !matches!(column_type, ColumnValueType::Number) {
                    warn!(
                        "Operator '{}' is only supported for numeric fields (field: {})",
                        operator, expr
                    );
                    return None;
                }
                let value = parse_scalar_value(raw_value, ColumnValueType::Number)?;
                let sql = format!("({expr} {op} ?)", expr = expr, op = op);
                Some(SqlFragment {
                    sql,
                    params: vec![value],
                })
            }
            "between" => {
                if !matches!(column_type, ColumnValueType::Number) {
                    warn!(
                        "Operator '{}' is only supported for numeric fields (field: {})",
                        operator, expr
                    );
                    return None;
                }
                let values = parse_value_list(raw_value, ColumnValueType::Number);
                if let Some(mut values) = values {
                    if values.len() == 2 {
                        let right = values.pop().unwrap();
                        let left = values.pop().unwrap();
                        Some(SqlFragment {
                            sql: format!("({expr} BETWEEN ? AND ?)", expr = expr),
                            params: vec![left, right],
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "in" => {
                let values = parse_value_list(raw_value, column_type)?;
                if values.is_empty() {
                    return None;
                }
                let placeholders = vec!["?"; values.len()].join(", ");
                let mut params = Vec::new();
                params.extend(values);
                Some(SqlFragment {
                    sql: format!("({expr} IN ({placeholders}))", expr = expr),
                    params,
                })
            }
            "contains" | "beginswith" | "endswith" => {
                if !matches!(column_type, ColumnValueType::Text) {
                    warn!(
                        "Operator '{}' is only supported for text fields (field: {})",
                        operator, expr
                    );
                    return None;
                }
                let mut value = match raw_value {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                if value.is_empty() {
                    return None;
                }
                value = match op.as_str() {
                    "contains" => format!("%{value}%"),
                    "beginswith" => format!("{value}%"),
                    "endswith" => format!("%{value}"),
                    _ => value,
                };
                Some(SqlFragment {
                    sql: format!("({expr} LIKE ?)", expr = expr),
                    params: vec![Value::from(value)],
                })
            }
            _ => {
                warn!("Unsupported query operator: {}", operator);
                None
            }
        }
    }

    fn parse_scalar_value(
        value: &serde_json::Value,
        column_type: ColumnValueType,
    ) -> Option<Value> {
        match column_type {
            ColumnValueType::Text => match value {
                serde_json::Value::String(s) => Some(Value::from(s.clone())),
                serde_json::Value::Number(n) => Some(Value::from(n.to_string())),
                serde_json::Value::Bool(b) => Some(Value::from(b.to_string())),
                _ => None,
            },
            ColumnValueType::Number => {
                if let Some(num) = value.as_f64() {
                    Some(Value::from(num))
                } else if let Some(s) = value.as_str() {
                    s.parse::<f64>().ok().map(Value::from)
                } else {
                    None
                }
            }
            ColumnValueType::Boolean => {
                if let Some(b) = value.as_bool() {
                    Some(Value::Bool(Some(b)))
                } else if let Some(s) = value.as_str() {
                    match s.to_lowercase().as_str() {
                        "true" | "1" | "yes" => Some(Value::Bool(Some(true))),
                        "false" | "0" | "no" => Some(Value::Bool(Some(false))),
                        _ => None,
                    }
                } else {
                    None
                }
            }
        }
    }

    fn parse_value_list(
        value: &serde_json::Value,
        column_type: ColumnValueType,
    ) -> Option<Vec<Value>> {
        match value {
            serde_json::Value::Array(items) => {
                let mut result = Vec::new();
                for item in items {
                    let Some(val) = parse_scalar_value(item, column_type) else {
                        return None;
                    };
                    result.push(val);
                }
                Some(result)
            }
            serde_json::Value::String(s) => {
                let parts: Vec<serde_json::Value> = s
                    .split(',')
                    .map(|segment| serde_json::Value::String(segment.trim().to_string()))
                    .collect();
                parse_value_list(&serde_json::Value::Array(parts), column_type)
            }
            _ => None,
        }
    }

    fn apply_matches_to_graph(
        graph: &mut Graph,
        target: QueryFilterTarget,
        matches: &HashSet<String>,
        mode: QueryFilterMode,
    ) {
        match target {
            QueryFilterTarget::Nodes => match mode {
                QueryFilterMode::Include => {
                    graph.nodes.retain(|node| matches.contains(&node.id));
                }
                QueryFilterMode::Exclude => {
                    graph.nodes.retain(|node| !matches.contains(&node.id));
                }
            },
            QueryFilterTarget::Edges => match mode {
                QueryFilterMode::Include => {
                    graph.edges.retain(|edge| matches.contains(&edge.id));
                }
                QueryFilterMode::Exclude => {
                    graph.edges.retain(|edge| !matches.contains(&edge.id));
                }
            },
            QueryFilterTarget::Layers => match mode {
                QueryFilterMode::Include => {
                    graph.layers.retain(|layer| matches.contains(&layer.id));
                }
                QueryFilterMode::Exclude => {
                    graph.layers.retain(|layer| !matches.contains(&layer.id));
                }
            },
        }
    }

    fn apply_link_pruning(graph: &mut Graph, mode: QueryLinkPruningMode) {
        match mode {
            QueryLinkPruningMode::AutoDropDanglingEdges => {
                graph.remove_dangling_edges();
            }
            QueryLinkPruningMode::RetainEdges => {}
            QueryLinkPruningMode::DropOrphanNodes => {
                graph.remove_dangling_edges();
                graph.remove_unconnected_nodes();
            }
        }
    }
}

// Merge Node Configuration
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "MergeNodeConfigInput")]
pub struct MergeNodeConfig {
    // Removed: input_refs - inputs come from incoming edges
    // Removed: output_graph_ref - output goes to outgoing edge
    pub merge_strategy: MergeStrategy,
    pub conflict_resolution: ConflictResolution,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum MergeStrategy {
    Union,
    Intersection,
    Difference,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum ConflictResolution {
    PreferFirst,
    PreferLast,
    Manual,
}

// Copy Node Configuration
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "CopyNodeConfigInput")]
pub struct CopyNodeConfig {
    // Removed: source_graph_ref - source comes from incoming edge
    // Removed: output_graph_ref - output goes to outgoing edge
    pub copy_type: CopyType,
    pub preserve_metadata: bool,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum CopyType {
    DeepCopy,
    ShallowCopy,
    Reference,
}

// Output Node Configuration
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "OutputNodeConfigInput")]
pub struct OutputNodeConfig {
    // Removed: source_graph_ref - source comes from incoming edge connection
    pub render_target: RenderTarget,
    pub output_path: String,
    pub render_config: Option<RenderConfig>,
    pub graph_config: Option<GraphConfig>,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum RenderTarget {
    Dot,
    Gml,
    Json,
    PlantUml,
    CsvNodes,
    CsvEdges,
    Mermaid,
    Custom,
}

#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "RenderConfigInput")]
pub struct RenderConfig {
    pub contain_nodes: Option<bool>,
    pub orientation: Option<Orientation>,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Orientation {
    Lr,
    Tb,
}

#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "GraphConfigInput")]
pub struct GraphConfig {
    pub generate_hierarchy: Option<bool>,
    pub max_partition_depth: Option<i32>,
    pub max_partition_width: Option<i32>,
    pub invert_graph: Option<bool>,
    pub aggregate_edges: Option<bool>,
    pub node_label_max_length: Option<i32>,
    pub node_label_insert_newlines_at: Option<i32>,
    pub edge_label_max_length: Option<i32>,
    pub edge_label_insert_newlines_at: Option<i32>,
}

// Union type for all node configurations
#[derive(Union, Clone, Debug, Serialize, Deserialize)]
pub enum NodeConfig {
    DataSource(DataSourceNodeConfig),
    Graph(GraphNodeConfig),
    Transform(TransformNodeConfig),
    Filter(FilterNodeConfig),
    Merge(MergeNodeConfig),
    Copy(CopyNodeConfig),
    Output(OutputNodeConfig),
}

// Execution metadata for DataSource nodes
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct DataSourceExecutionMetadata {
    #[graphql(name = "dataSourceId")]
    pub data_source_id: i32,
    pub filename: String,
    pub status: String,
    #[graphql(name = "processedAt")]
    pub processed_at: Option<String>,
    #[graphql(name = "executionState")]
    pub execution_state: String,
    #[graphql(name = "errorMessage")]
    pub error_message: Option<String>,
}

// Execution metadata for Graph nodes
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct GraphExecutionMetadata {
    #[graphql(name = "graphId")]
    pub graph_id: i32,
    #[graphql(name = "nodeCount")]
    pub node_count: i32,
    #[graphql(name = "edgeCount")]
    pub edge_count: i32,
    #[graphql(name = "executionState")]
    pub execution_state: String,
    #[graphql(name = "computedDate")]
    pub computed_date: Option<String>,
    #[graphql(name = "errorMessage")]
    pub error_message: Option<String>,
}

// Node execution status change event for subscriptions
#[derive(Clone, Debug, SimpleObject)]
pub struct NodeExecutionStatusEvent {
    #[graphql(name = "projectId")]
    pub project_id: i32,
    #[graphql(name = "nodeId")]
    pub node_id: String,
    #[graphql(name = "nodeType")]
    pub node_type: PlanDagNodeType,
    #[graphql(name = "datasourceExecution")]
    pub datasource_execution: Option<DataSourceExecutionMetadata>,
    #[graphql(name = "graphExecution")]
    pub graph_execution: Option<GraphExecutionMetadata>,
    pub timestamp: String,
}

// Plan DAG Node Structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanDagNode {
    pub id: String,
    pub node_type: PlanDagNodeType,
    pub position: Position,
    pub source_position: Option<String>,
    pub target_position: Option<String>,
    pub metadata: NodeMetadata,
    pub config: String, // JSON string for now, will be parsed as NodeConfig
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Optional execution metadata populated by query resolver
    pub datasource_execution: Option<DataSourceExecutionMetadata>,
    pub graph_execution: Option<GraphExecutionMetadata>,
}

#[Object]
impl PlanDagNode {
    async fn id(&self) -> &str {
        &self.id
    }

    #[graphql(name = "nodeType")]
    async fn node_type(&self) -> PlanDagNodeType {
        self.node_type
    }

    async fn position(&self) -> &Position {
        &self.position
    }

    #[graphql(name = "sourcePosition")]
    async fn source_position(&self) -> Option<&String> {
        self.source_position.as_ref()
    }

    #[graphql(name = "targetPosition")]
    async fn target_position(&self) -> Option<&String> {
        self.target_position.as_ref()
    }

    async fn metadata(&self) -> &NodeMetadata {
        &self.metadata
    }

    async fn config(&self) -> &str {
        &self.config
    }

    #[graphql(name = "createdAt")]
    async fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[graphql(name = "updatedAt")]
    async fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    #[graphql(name = "datasourceExecution")]
    async fn datasource_execution(&self) -> Option<&DataSourceExecutionMetadata> {
        self.datasource_execution.as_ref()
    }

    #[graphql(name = "graphExecution")]
    async fn graph_execution(&self) -> Option<&GraphExecutionMetadata> {
        self.graph_execution.as_ref()
    }

    /// Parse the config JSON into a specific node configuration type
    async fn parsed_config(&self) -> Result<String> {
        // For now, return the raw JSON. In the future, this could parse to specific types
        Ok(self.config.clone())
    }
}

// Plan DAG Edge Structure
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct PlanDagEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    // Removed source_handle and target_handle for floating edges
    pub metadata: EdgeMetadata,
    #[graphql(name = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[graphql(name = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

// Plan DAG Metadata
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "PlanDagMetadataInput")]
pub struct PlanDagMetadata {
    pub version: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub created: Option<String>,
    #[graphql(name = "lastModified")]
    pub last_modified: Option<String>,
    pub author: Option<String>,
}

// Complete Plan DAG Structure
#[derive(SimpleObject, Clone, Debug)]
pub struct PlanDag {
    pub version: String,
    pub nodes: Vec<PlanDagNode>,
    pub edges: Vec<PlanDagEdge>,
    pub metadata: PlanDagMetadata,
}

// Input types for mutations
#[derive(InputObject, Clone, Debug)]
pub struct PlanDagInput {
    pub version: String,
    pub nodes: Vec<PlanDagNodeInput>,
    pub edges: Vec<PlanDagEdgeInput>,
    pub metadata: PlanDagMetadata,
}

#[derive(InputObject, Clone, Debug)]
pub struct PlanDagNodeInput {
    /// Optional ID - if not provided, backend will generate one
    pub id: Option<String>,
    #[graphql(name = "nodeType")]
    pub node_type: PlanDagNodeType,
    pub position: Position,
    pub metadata: NodeMetadata,
    pub config: String, // JSON string
}

#[derive(InputObject, Clone, Debug)]
pub struct PlanDagEdgeInput {
    /// Optional ID - if not provided, backend will generate one
    pub id: Option<String>,
    pub source: String,
    pub target: String,
    // Removed source_handle and target_handle for floating edges
    pub metadata: EdgeMetadata,
}

#[derive(InputObject, Clone, Debug)]
pub struct PlanDagNodeUpdateInput {
    pub position: Option<Position>,
    pub metadata: Option<NodeMetadata>,
    pub config: Option<String>,
}

#[derive(InputObject, Clone, Debug)]
pub struct PlanDagEdgeUpdateInput {
    // Removed source_handle and target_handle for floating edges
    pub metadata: Option<EdgeMetadata>,
}

// Validation types
#[derive(SimpleObject, Clone, Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct ValidationError {
    #[graphql(name = "nodeId")]
    pub node_id: Option<String>,
    #[graphql(name = "edgeId")]
    pub edge_id: Option<String>,
    pub error_type: ValidationErrorType,
    pub message: String,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct ValidationWarning {
    #[graphql(name = "nodeId")]
    pub node_id: Option<String>,
    #[graphql(name = "edgeId")]
    pub edge_id: Option<String>,
    pub warning_type: ValidationWarningType,
    pub message: String,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum ValidationErrorType {
    MissingInput,
    InvalidConnection,
    CyclicDependency,
    InvalidConfig,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum ValidationWarningType {
    UnusedOutput,
    PerformanceImpact,
    ConfigurationSuggestion,
}

// Conversions from database entities
impl From<plan_dag_nodes::Model> for PlanDagNode {
    fn from(model: plan_dag_nodes::Model) -> Self {
        let node_type = match model.node_type.as_str() {
            "DataSourceNode" => PlanDagNodeType::DataSource,
            "GraphNode" => PlanDagNodeType::Graph,
            "TransformNode" => PlanDagNodeType::Transform,
            "FilterNode" => PlanDagNodeType::Filter,
            "MergeNode" => PlanDagNodeType::Merge,
            "CopyNode" => PlanDagNodeType::Copy,
            "OutputNode" => PlanDagNodeType::Output,
            _ => PlanDagNodeType::DataSource, // Default fallback
        };

        let metadata: NodeMetadata =
            serde_json::from_str(&model.metadata_json).unwrap_or_else(|_| NodeMetadata {
                label: "Unnamed Node".to_string(),
                description: None,
            });

        Self {
            id: model.id,
            node_type,
            position: Position {
                x: model.position_x,
                y: model.position_y,
            },
            source_position: model.source_position,
            target_position: model.target_position,
            metadata,
            config: model.config_json,
            created_at: model.created_at,
            updated_at: model.updated_at,
            datasource_execution: None,
            graph_execution: None,
        }
    }
}

impl From<plan_dag_edges::Model> for PlanDagEdge {
    fn from(model: plan_dag_edges::Model) -> Self {
        let metadata: EdgeMetadata =
            serde_json::from_str(&model.metadata_json).unwrap_or(EdgeMetadata {
                label: None,
                data_type: DataType::GraphData,
            });

        Self {
            id: model.id,
            source: model.source_node_id,
            target: model.target_node_id,
            // Removed source_handle and target_handle for floating edges
            metadata,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl From<PlanDagSnapshot> for PlanDag {
    fn from(snapshot: PlanDagSnapshot) -> Self {
        Self {
            version: snapshot.version,
            nodes: snapshot.nodes,
            edges: snapshot.edges,
            metadata: snapshot.metadata,
        }
    }
}
