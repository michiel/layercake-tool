use anyhow::{Context, Result as AnyResult};
use async_graphql::*;
use sea_orm::{ConnectionTrait, DatabaseConnection, FromQueryResult, Statement, Value};
use serde::{de, Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use tracing::warn;

use crate::graph::Graph;
use crate::graphql::types::scalars::JSON;

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
        D: serde::Deserializer<'de>,
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
                        // Safe to unwrap: we just checked that values.len() == 2
                        let right = values.pop().expect("Expected right value in BETWEEN clause");
                        let left = values.pop().expect("Expected left value in BETWEEN clause");
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
