use anyhow::Result;
use chrono::Utc;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};

use crate::database::entities::{graph_edges, graph_nodes};

const GRAPH_BATCH_SIZE: usize = 500;

/// Delete all node/edge/layer rows for a graph using the provided connection/transaction.
pub async fn clear_graph_storage<C>(conn: &C, graph_id: i32) -> Result<()>
where
    C: ConnectionTrait,
{
    use crate::database::entities::graph_edges::{Column as EdgeColumn, Entity as EdgeEntity};
    use crate::database::entities::graph_layers::{Column as LayerColumn, Entity as LayerEntity};
    use crate::database::entities::graph_nodes::{Column as NodeColumn, Entity as NodeEntity};

    EdgeEntity::delete_many()
        .filter(EdgeColumn::GraphId.eq(graph_id))
        .exec(conn)
        .await?;

    NodeEntity::delete_many()
        .filter(NodeColumn::GraphId.eq(graph_id))
        .exec(conn)
        .await?;

    LayerEntity::delete_many()
        .filter(LayerColumn::GraphId.eq(graph_id))
        .exec(conn)
        .await?;

    Ok(())
}

/// Insert graph node ActiveModels in manageable batches.
pub async fn insert_node_batches<C>(
    conn: &C,
    mut models: Vec<graph_nodes::ActiveModel>,
) -> Result<()>
where
    C: ConnectionTrait,
{
    while !models.is_empty() {
        let batch_size = GRAPH_BATCH_SIZE.min(models.len());
        let batch: Vec<_> = models.drain(..batch_size).collect();
        graph_nodes::Entity::insert_many(batch).exec(conn).await?;
    }
    Ok(())
}

/// Insert graph edge ActiveModels in manageable batches.
pub async fn insert_edge_batches<C>(
    conn: &C,
    mut models: Vec<graph_edges::ActiveModel>,
) -> Result<()>
where
    C: ConnectionTrait,
{
    while !models.is_empty() {
        let batch_size = GRAPH_BATCH_SIZE.min(models.len());
        let batch: Vec<_> = models.drain(..batch_size).collect();
        graph_edges::Entity::insert_many(batch).exec(conn).await?;
    }
    Ok(())
}

/// Helper to build a graph_nodes::ActiveModel from a Graph node.
pub fn node_to_active_model(graph_id: i32, node: &crate::graph::Node) -> graph_nodes::ActiveModel {
    let attrs = node
        .comment
        .as_ref()
        .map(|comment| serde_json::json!({ "comment": comment }));

    graph_nodes::ActiveModel {
        id: Set(node.id.clone()),
        graph_id: Set(graph_id),
        label: Set(Some(node.label.clone())),
        layer: Set(Some(node.layer.clone())),
        weight: Set(Some(node.weight as f64)),
        is_partition: Set(node.is_partition),
        belongs_to: Set(node.belongs_to.clone()),
        dataset_id: Set(node.dataset),
        attrs: Set(attrs),
        comment: Set(node.comment.clone()),
        created_at: Set(Utc::now()),
    }
}

/// Helper to build a graph_edges::ActiveModel from a Graph edge.
pub fn edge_to_active_model(graph_id: i32, edge: &crate::graph::Edge) -> graph_edges::ActiveModel {
    let attrs = edge
        .comment
        .as_ref()
        .map(|comment| serde_json::json!({ "comment": comment }));

    graph_edges::ActiveModel {
        id: Set(edge.id.clone()),
        graph_id: Set(graph_id),
        source: Set(edge.source.clone()),
        target: Set(edge.target.clone()),
        label: Set(Some(edge.label.clone())),
        layer: Set(Some(edge.layer.clone())),
        weight: Set(Some(edge.weight as f64)),
        dataset_id: Set(edge.dataset),
        attrs: Set(attrs),
        comment: Set(edge.comment.clone()),
        created_at: Set(Utc::now()),
    }
}
