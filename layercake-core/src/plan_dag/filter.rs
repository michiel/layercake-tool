use anyhow::{Context, Result as AnyResult};
use sea_orm::DatabaseConnection;
use serde::{de, Deserialize, Serialize};
use serde_json::json;
use serde_json::Value as JsonValue;
use std::collections::HashSet;
use tracing::warn;

use crate::graph::Graph;

// Filter Node Configuration
#[derive(Clone, Debug, Serialize)]
pub struct FilterNodeConfig {
    pub query: QueryFilterConfig,
}

/// Custom deserializer that handles migration from legacy schema v1 to current schema v2.
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
            return Ok(Self {
                query: query.normalized(),
            });
        }

        if let Some(filters) = wire.filters {
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryFilterConfig {
    pub targets: Vec<QueryFilterTarget>,
    pub mode: QueryFilterMode,
    pub link_pruning_mode: QueryLinkPruningMode,
    pub rule_group: JsonValue,
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

fn default_rule_group() -> JsonValue {
    json!({
        "combinator": "and",
        "rules": []
    })
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QueryFilterTarget {
    Nodes,
    Edges,
    Layers,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QueryFilterMode {
    Include,
    Exclude,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QueryLinkPruningMode {
    AutoDropDanglingEdges,
    RetainEdges,
    DropOrphanNodes,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyGraphFilter {
    kind: Option<String>,
    params: Option<LegacyGraphFilterParams>,
}

impl LegacyGraphFilter {
    fn is_query(&self) -> bool {
        self.kind
            .as_deref()
            .map(|k| k.eq_ignore_ascii_case("query") || k.eq_ignore_ascii_case("querytext"))
            .unwrap_or(false)
    }
}

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

        let mut selected_nodes = HashSet::new();
        let mut selected_edges = HashSet::new();
        let mut selected_layers = HashSet::new();

        // Evaluate the rule group against the in-memory graph. The graph is
        // already fully loaded here, so there is no need (and, since the legacy
        // graph_nodes/graph_edges/graph_layers tables were dropped, no ability)
        // to re-query the database. `_context` is retained for API stability.
        let _ = context;

        if targets.contains(&QueryFilterTarget::Nodes) {
            for node in &graph.nodes {
                if eval_rule_group(&rule_group, &node_field_value(node))? {
                    selected_nodes.insert(node.id.clone());
                }
            }
        }

        if targets.contains(&QueryFilterTarget::Edges) {
            for edge in &graph.edges {
                if eval_rule_group(&rule_group, &edge_field_value(edge))? {
                    selected_edges.insert(edge.id.clone());
                }
            }
        }

        if targets.contains(&QueryFilterTarget::Layers) {
            for layer in &graph.layers {
                if eval_rule_group(&rule_group, &layer_field_value(layer))? {
                    selected_layers.insert(layer.id.clone());
                }
            }
        }

        let should_include = matches!(config.mode, QueryFilterMode::Include);
        let mut node_ids: HashSet<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
        let mut edge_ids: HashSet<String> = graph.edges.iter().map(|e| e.id.clone()).collect();
        let mut layer_ids: HashSet<String> = graph.layers.iter().map(|l| l.id.clone()).collect();

        if should_include {
            if targets.contains(&QueryFilterTarget::Nodes) {
                node_ids = node_ids.intersection(&selected_nodes).cloned().collect();
            }
            if targets.contains(&QueryFilterTarget::Edges) {
                edge_ids = edge_ids.intersection(&selected_edges).cloned().collect();
            }
            if targets.contains(&QueryFilterTarget::Layers) {
                layer_ids = layer_ids.intersection(&selected_layers).cloned().collect();
            }
        } else {
            if targets.contains(&QueryFilterTarget::Nodes) {
                node_ids = node_ids.difference(&selected_nodes).cloned().collect();
            }
            if targets.contains(&QueryFilterTarget::Edges) {
                edge_ids = edge_ids.difference(&selected_edges).cloned().collect();
            }
            if targets.contains(&QueryFilterTarget::Layers) {
                layer_ids = layer_ids.difference(&selected_layers).cloned().collect();
            }
        }

        if targets.contains(&QueryFilterTarget::Nodes) {
            let remove_edges_for_missing_nodes = !edge_ids.is_empty();
            graph.nodes.retain(|node| node_ids.contains(&node.id));
            if remove_edges_for_missing_nodes {
                edge_ids = edge_ids
                    .into_iter()
                    .filter(|edge_id| {
                        graph.edges.iter().any(|edge| {
                            &edge.id == edge_id
                                && node_ids.contains(&edge.source)
                                && node_ids.contains(&edge.target)
                        })
                    })
                    .collect();
            }
        }

        if targets.contains(&QueryFilterTarget::Edges) {
            graph.edges.retain(|edge| edge_ids.contains(&edge.id));
        }

        if targets.contains(&QueryFilterTarget::Layers) {
            graph.layers.retain(|layer| layer_ids.contains(&layer.id));
        }

        let mut pruned_edges = HashSet::new();
        if !targets.contains(&QueryFilterTarget::Edges) {
            for edge in &graph.edges {
                if !node_ids.contains(&edge.source) || !node_ids.contains(&edge.target) {
                    pruned_edges.insert(edge.id.clone());
                }
            }
        }

        match config.link_pruning_mode {
            QueryLinkPruningMode::AutoDropDanglingEdges => {
                graph.edges.retain(|edge| !pruned_edges.contains(&edge.id));
            }
            QueryLinkPruningMode::RetainEdges => {}
            QueryLinkPruningMode::DropOrphanNodes => {
                let mut referenced_nodes: HashSet<String> = HashSet::new();
                for edge in &graph.edges {
                    referenced_nodes.insert(edge.source.clone());
                    referenced_nodes.insert(edge.target.clone());
                }

                graph
                    .nodes
                    .retain(|node| referenced_nodes.contains(&node.id));
            }
        }

        Ok(())
    }

    /// Resolve a filter `field` to a string value for a node. Known columns map
    /// to struct fields; anything else is looked up in the `attributes` JSON.
    fn node_field_value(node: &crate::graph::Node) -> impl Fn(&str) -> Option<String> + '_ {
        move |field: &str| match field {
            "id" | "external_id" => Some(node.id.clone()),
            "label" => Some(node.label.clone()),
            "layer" => Some(node.layer.clone()),
            "weight" => Some(node.weight.to_string()),
            "comment" => node.comment.clone(),
            "belongs_to" | "belongsTo" => node.belongs_to.clone(),
            "is_partition" | "isPartition" => Some(node.is_partition.to_string()),
            "dataset" => node.dataset.map(|d| d.to_string()),
            other => attribute_value(node.attributes.as_ref(), other),
        }
    }

    fn edge_field_value(edge: &crate::graph::Edge) -> impl Fn(&str) -> Option<String> + '_ {
        move |field: &str| match field {
            "id" | "external_id" => Some(edge.id.clone()),
            "source" => Some(edge.source.clone()),
            "target" => Some(edge.target.clone()),
            "label" => Some(edge.label.clone()),
            "layer" => Some(edge.layer.clone()),
            "weight" => Some(edge.weight.to_string()),
            "comment" => edge.comment.clone(),
            "dataset" => edge.dataset.map(|d| d.to_string()),
            other => attribute_value(edge.attributes.as_ref(), other),
        }
    }

    fn layer_field_value(layer: &crate::graph::Layer) -> impl Fn(&str) -> Option<String> + '_ {
        move |field: &str| match field {
            "id" => Some(layer.id.clone()),
            "label" => Some(layer.label.clone()),
            "alias" => layer.alias.clone(),
            "dataset" => layer.dataset.map(|d| d.to_string()),
            other => attribute_value(layer.attributes.as_ref(), other),
        }
    }

    /// Look up a field in an optional `attributes` JSON object, returning its
    /// value as a string (strings unquoted, numbers/bools stringified).
    fn attribute_value(attributes: Option<&JsonValue>, field: &str) -> Option<String> {
        let value = attributes?.get(field)?;
        match value {
            JsonValue::String(s) => Some(s.clone()),
            JsonValue::Null => None,
            other => Some(other.to_string()),
        }
    }

    /// Evaluate a rule group against a single record's field resolver.
    fn eval_rule_group(
        rule_group: &QueryRuleGroup,
        get_field: &impl Fn(&str) -> Option<String>,
    ) -> AnyResult<bool> {
        let is_and = !rule_group.combinator.eq_ignore_ascii_case("or");

        // An empty rule group matches everything (parity with the previous "1=1").
        if rule_group.rules.is_empty() {
            return Ok(true);
        }

        let mut result = is_and;
        for rule in &rule_group.rules {
            let matched = match rule {
                QueryRule::Group(group) => eval_rule_group(group, get_field)?,
                QueryRule::Rule(rule) => eval_rule(rule, get_field)?,
            };
            if is_and {
                result = result && matched;
                if !result {
                    break;
                }
            } else {
                result = result || matched;
                if result {
                    break;
                }
            }
        }
        Ok(result)
    }

    fn eval_rule(
        rule: &QueryRuleConfig,
        get_field: &impl Fn(&str) -> Option<String>,
    ) -> AnyResult<bool> {
        let field_value = get_field(&rule.field);
        let value = rule.value.as_deref().unwrap_or("");
        let comparator = rule.comparator.as_deref().unwrap_or("=");

        let as_str = field_value.as_deref();
        let matched = match rule.operator.as_str() {
            "isEmpty" => as_str.map(|s| s.is_empty()).unwrap_or(true),
            "isNotEmpty" => as_str.map(|s| !s.is_empty()).unwrap_or(false),
            "isNull" => field_value.is_none(),
            "isNotNull" => field_value.is_some(),
            "contains" => as_str.map(|s| s.contains(value)).unwrap_or(false),
            "notContains" => as_str.map(|s| !s.contains(value)).unwrap_or(true),
            "startsWith" => as_str.map(|s| s.starts_with(value)).unwrap_or(false),
            "endsWith" => as_str.map(|s| s.ends_with(value)).unwrap_or(false),
            "equals" => match comparator {
                "<>" | "!=" => as_str != Some(value),
                _ => as_str == Some(value),
            },
            "notEquals" => as_str != Some(value),
            "in" => {
                let set: HashSet<&str> = value.split(',').map(|s| s.trim()).collect();
                as_str.map(|s| set.contains(s)).unwrap_or(false)
            }
            "notIn" => {
                let set: HashSet<&str> = value.split(',').map(|s| s.trim()).collect();
                as_str.map(|s| !set.contains(s)).unwrap_or(true)
            }
            "greaterThan" => numeric_cmp(as_str, value, |a, b| a > b),
            "lessThan" => numeric_cmp(as_str, value, |a, b| a < b),
            "greaterThanOrEqual" => numeric_cmp(as_str, value, |a, b| a >= b),
            "lessThanOrEqual" => numeric_cmp(as_str, value, |a, b| a <= b),
            "between" => {
                let parts: Vec<&str> = value.split(',').collect();
                if parts.len() != 2 {
                    return Err(anyhow::anyhow!(
                        "Between operator requires two values separated by comma"
                    ));
                }
                let lo = parts[0].trim().parse::<f64>().ok();
                let hi = parts[1].trim().parse::<f64>().ok();
                match (as_str.and_then(|s| s.trim().parse::<f64>().ok()), lo, hi) {
                    (Some(v), Some(lo), Some(hi)) => v >= lo && v <= hi,
                    _ => false,
                }
            }
            other => {
                warn!("Unsupported filter operator: {}", other);
                true
            }
        };
        Ok(matched)
    }

    /// Compare a field value and a target value numerically; if either is not a
    /// number, no match (mirrors SQL comparison against non-numeric text).
    fn numeric_cmp(field: Option<&str>, value: &str, op: impl Fn(f64, f64) -> bool) -> bool {
        match (
            field.and_then(|s| s.trim().parse::<f64>().ok()),
            value.trim().parse::<f64>().ok(),
        ) {
            (Some(a), Some(b)) => op(a, b),
            _ => false,
        }
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct QueryRuleGroup {
        pub combinator: String,
        pub rules: Vec<QueryRule>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(tag = "type", rename_all = "camelCase")]
    enum QueryRule {
        Group(QueryRuleGroup),
        Rule(QueryRuleConfig),
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct QueryRuleConfig {
        pub field: String,
        pub operator: String,
        pub value: Option<String>,
        pub comparator: Option<String>,
    }
}
