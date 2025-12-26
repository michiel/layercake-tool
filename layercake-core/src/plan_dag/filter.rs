use anyhow::{Context, Result as AnyResult};
use sea_orm::{ConnectionTrait, DatabaseConnection, FromQueryResult, Statement};
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

        if targets.contains(&QueryFilterTarget::Nodes) {
            let nodes = query_nodes(context.db, context.graph_id, &rule_group).await?;
            selected_nodes.extend(nodes);
        }

        if targets.contains(&QueryFilterTarget::Edges) {
            let edges = query_edges(context.db, context.graph_id, &rule_group).await?;
            selected_edges.extend(edges);
        }

        if targets.contains(&QueryFilterTarget::Layers) {
            let layers = query_layers(context.db, context.graph_id, &rule_group).await?;
            selected_layers.extend(layers);
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
                        graph
                            .edges
                            .iter()
                            .any(|edge| &edge.id == edge_id && node_ids.contains(&edge.source)
                                && node_ids.contains(&edge.target))
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
                graph
                    .edges
                    .retain(|edge| !pruned_edges.contains(&edge.id));
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

    async fn query_nodes(
        db: &DatabaseConnection,
        graph_id: i32,
        rule_group: &QueryRuleGroup,
    ) -> AnyResult<HashSet<String>> {
        let sql = build_query_sql("graph_nodes", graph_id, rule_group)?;
        execute_query(db, &sql)
            .await
            .context("Failed to query nodes")
    }

    async fn query_edges(
        db: &DatabaseConnection,
        graph_id: i32,
        rule_group: &QueryRuleGroup,
    ) -> AnyResult<HashSet<String>> {
        let sql = build_query_sql("graph_edges", graph_id, rule_group)?;
        execute_query(db, &sql)
            .await
            .context("Failed to query edges")
    }

    async fn query_layers(
        db: &DatabaseConnection,
        graph_id: i32,
        rule_group: &QueryRuleGroup,
    ) -> AnyResult<HashSet<String>> {
        let sql = build_query_sql("graph_layers", graph_id, rule_group)?;
        execute_query(db, &sql)
            .await
            .context("Failed to query layers")
    }

    async fn execute_query(db: &DatabaseConnection, sql: &str) -> AnyResult<HashSet<String>> {
        #[derive(FromQueryResult)]
        struct ResultRow {
            id: String,
        }

        let statement = Statement::from_string(db.get_database_backend(), sql.to_string());
        let rows: Vec<ResultRow> = ResultRow::find_by_statement(statement)
            .all(db)
            .await
            .context("Failed to execute query")?;

        Ok(rows.into_iter().map(|row| row.id).collect())
    }

    fn build_query_sql(table: &str, graph_id: i32, rule_group: &QueryRuleGroup) -> AnyResult<String> {
        let rule_sql = build_rule_group_sql(rule_group)?;
        Ok(format!(
            "SELECT id FROM {} WHERE graph_id = {} AND {}",
            table, graph_id, rule_sql
        ))
    }

    fn build_rule_group_sql(rule_group: &QueryRuleGroup) -> AnyResult<String> {
        let mut clauses = Vec::new();
        for rule in &rule_group.rules {
            match rule {
                QueryRule::Group(group) => {
                    let group_sql = build_rule_group_sql(group)?;
                    clauses.push(format!("({})", group_sql));
                }
                QueryRule::Rule(rule) => {
                    let clause = build_rule_clause(rule)?;
                    clauses.push(clause);
                }
            }
        }

        let joiner = match rule_group.combinator.as_str() {
            "and" => " AND ",
            "or" => " OR ",
            _ => " AND ",
        };

        if clauses.is_empty() {
            Ok("1=1".to_string())
        } else {
            Ok(clauses.join(joiner))
        }
    }

    fn build_rule_clause(rule: &QueryRuleConfig) -> AnyResult<String> {
        let field = &rule.field;
        let operator = rule.operator.as_str();
        let value = rule.value.as_deref().unwrap_or("");
        let comparator = rule.comparator.as_deref().unwrap_or("=");

        let clause = match operator {
            "isEmpty" => format!("({} IS NULL OR {} = '')", field, field),
            "isNotEmpty" => format!("({} IS NOT NULL AND {} <> '')", field, field),
            "contains" => format!("{} LIKE '%{}%'", field, value.replace('\'', "''")),
            "notContains" => format!("{} NOT LIKE '%{}%'", field, value.replace('\'', "''")),
            "equals" => format!("{} {} '{}'", field, comparator, value.replace('\'', "''")),
            "notEquals" => format!("{} {} '{}'", field, comparator, value.replace('\'', "''")),
            "startsWith" => format!("{} LIKE '{}%'", field, value.replace('\'', "''")),
            "endsWith" => format!("{} LIKE '%{}'", field, value.replace('\'', "''")),
            "in" => {
                let items: Vec<String> = value
                    .split(',')
                    .map(|item| format!("'{}'", item.trim().replace('\'', "''")))
                    .collect();
                format!("{} IN ({})", field, items.join(", "))
            }
            "notIn" => {
                let items: Vec<String> = value
                    .split(',')
                    .map(|item| format!("'{}'", item.trim().replace('\'', "''")))
                    .collect();
                format!("{} NOT IN ({})", field, items.join(", "))
            }
            "isNull" => format!("{} IS NULL", field),
            "isNotNull" => format!("{} IS NOT NULL", field),
            "greaterThan" => format!("{} > {}", field, value),
            "lessThan" => format!("{} < {}", field, value),
            "greaterThanOrEqual" => format!("{} >= {}", field, value),
            "lessThanOrEqual" => format!("{} <= {}", field, value),
            "between" => {
                let parts: Vec<&str> = value.split(',').collect();
                if parts.len() != 2 {
                    return Err(anyhow::anyhow!(
                        "Between operator requires two values separated by comma"
                    ));
                }
                format!("{} BETWEEN {} AND {}", field, parts[0], parts[1])
            }
            _ => {
                warn!("Unsupported operator: {}", operator);
                "1=1".to_string()
            }
        };

        Ok(clause)
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
