use crate::database::entities::{
    graph_data, graph_data::GraphDataStatus, graph_data_edges, graph_data_nodes,
};
use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait,
};
use serde_json::Value;

pub struct GraphDataService {
    db: DatabaseConnection,
}

impl GraphDataService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(&self, input: GraphDataCreate) -> Result<graph_data::Model, sea_orm::DbErr> {
        let now = Utc::now();
        let active = graph_data::ActiveModel {
            project_id: Set(input.project_id),
            name: Set(input.name),
            source_type: Set(input.source_type),
            dag_node_id: Set(input.dag_node_id),
            file_format: Set(input.file_format),
            origin: Set(input.origin),
            filename: Set(input.filename),
            blob: Set(input.blob),
            file_size: Set(input.file_size),
            processed_at: Set(input.processed_at),
            source_hash: Set(input.source_hash),
            computed_date: Set(input.computed_date),
            last_edit_sequence: Set(input.last_edit_sequence.unwrap_or(0)),
            has_pending_edits: Set(input.has_pending_edits.unwrap_or(false)),
            last_replay_at: Set(input.last_replay_at),
            node_count: Set(0),
            edge_count: Set(0),
            error_message: Set(None),
            metadata: Set(input.metadata),
            annotations: Set(input.annotations),
            status: Set(input.status.unwrap_or(GraphDataStatus::Processing).into()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        graph_data::Entity::insert(active).exec_with_returning(&self.db).await
    }

    pub async fn get_by_id(&self, id: i32) -> Result<Option<graph_data::Model>, sea_orm::DbErr> {
        graph_data::Entity::find_by_id(id).one(&self.db).await
    }

    pub async fn list_by_project_and_source(
        &self,
        project_id: i32,
        source_type: &str,
    ) -> Result<Vec<graph_data::Model>, sea_orm::DbErr> {
        graph_data::Entity::find()
            .filter(graph_data::Column::ProjectId.eq(project_id))
            .filter(graph_data::Column::SourceType.eq(source_type))
            .all(&self.db)
            .await
    }

    pub async fn replace_nodes(
        &self,
        graph_data_id: i32,
        nodes: Vec<GraphDataNodeInput>,
    ) -> Result<(), sea_orm::DbErr> {
        let txn = self.db.begin().await?;

        graph_data_nodes::Entity::delete_many()
            .filter(graph_data_nodes::Column::GraphDataId.eq(graph_data_id))
            .exec(&txn)
            .await?;

        let now = Utc::now();
        for node in nodes.iter() {
            let active = graph_data_nodes::ActiveModel {
                graph_data_id: Set(graph_data_id),
                external_id: Set(node.external_id.clone()),
                label: Set(node.label.clone()),
                layer: Set(node.layer.clone()),
                weight: Set(node.weight),
                is_partition: Set(node.is_partition.unwrap_or(false)),
                belongs_to: Set(node.belongs_to.clone()),
                comment: Set(node.comment.clone()),
                source_dataset_id: Set(node.source_dataset_id),
                attributes: Set(node.attributes.clone()),
                created_at: Set(node.created_at.unwrap_or(now)),
                ..Default::default()
            };
            graph_data_nodes::Entity::insert(active).exec(&txn).await?;
        }

        graph_data::ActiveModel {
            id: Set(graph_data_id),
            node_count: Set(nodes.len() as i32),
            updated_at: Set(now),
            ..Default::default()
        }
        .update(&txn)
        .await?;

        txn.commit().await
    }

    pub async fn replace_edges(
        &self,
        graph_data_id: i32,
        edges: Vec<GraphDataEdgeInput>,
    ) -> Result<(), sea_orm::DbErr> {
        let txn = self.db.begin().await?;

        graph_data_edges::Entity::delete_many()
            .filter(graph_data_edges::Column::GraphDataId.eq(graph_data_id))
            .exec(&txn)
            .await?;

        let now = Utc::now();
        for edge in edges.iter() {
            let active = graph_data_edges::ActiveModel {
                graph_data_id: Set(graph_data_id),
                external_id: Set(edge.external_id.clone()),
                source: Set(edge.source.clone()),
                target: Set(edge.target.clone()),
                label: Set(edge.label.clone()),
                layer: Set(edge.layer.clone()),
                weight: Set(edge.weight),
                comment: Set(edge.comment.clone()),
                source_dataset_id: Set(edge.source_dataset_id),
                attributes: Set(edge.attributes.clone()),
                created_at: Set(edge.created_at.unwrap_or(now)),
                ..Default::default()
            };
            graph_data_edges::Entity::insert(active).exec(&txn).await?;
        }

        graph_data::ActiveModel {
            id: Set(graph_data_id),
            edge_count: Set(edges.len() as i32),
            updated_at: Set(now),
            ..Default::default()
        }
        .update(&txn)
        .await?;

        txn.commit().await
    }

    pub async fn load_nodes(
        &self,
        graph_data_id: i32,
    ) -> Result<Vec<graph_data_nodes::Model>, sea_orm::DbErr> {
        graph_data_nodes::Entity::find()
            .filter(graph_data_nodes::Column::GraphDataId.eq(graph_data_id))
            .all(&self.db)
            .await
    }

    pub async fn load_edges(
        &self,
        graph_data_id: i32,
    ) -> Result<Vec<graph_data_edges::Model>, sea_orm::DbErr> {
        graph_data_edges::Entity::find()
            .filter(graph_data_edges::Column::GraphDataId.eq(graph_data_id))
            .all(&self.db)
            .await
    }

    pub async fn mark_status(
        &self,
        graph_data_id: i32,
        status: GraphDataStatus,
        source_hash: Option<String>,
    ) -> Result<(), sea_orm::DbErr> {
        let mut model: graph_data::ActiveModel = graph_data::Entity::find_by_id(graph_data_id)
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound(format!(
                "graph_data {}",
                graph_data_id
            )))?
            .into();

        model.status = Set(status.into());
        if let Some(hash) = source_hash {
            model.source_hash = Set(Some(hash));
        }
        model.updated_at = Set(Utc::now());
        model.update(&self.db).await.map(|_| ())
    }

    pub async fn load_full(
        &self,
        graph_data_id: i32,
    ) -> Result<(graph_data::Model, Vec<graph_data_nodes::Model>, Vec<graph_data_edges::Model>), sea_orm::DbErr>
    {
        let graph = graph_data::Entity::find_by_id(graph_data_id)
            .one(&self.db)
            .await?;

        if let Some(graph) = graph {
            let nodes = self.load_nodes(graph.id).await?;
            let edges = self.load_edges(graph.id).await?;
            Ok((graph, nodes, edges))
        } else {
            Err(sea_orm::DbErr::RecordNotFound(format!(
                "graph_data {}",
                graph_data_id
            )))
        }
    }
}

pub struct GraphDataCreate {
    pub project_id: i32,
    pub name: String,
    pub source_type: String,
    pub dag_node_id: Option<String>,
    pub file_format: Option<String>,
    pub origin: Option<String>,
    pub filename: Option<String>,
    pub blob: Option<Vec<u8>>,
    pub file_size: Option<i64>,
    pub processed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub source_hash: Option<String>,
    pub computed_date: Option<chrono::DateTime<chrono::Utc>>,
    pub last_edit_sequence: Option<i32>,
    pub has_pending_edits: Option<bool>,
    pub last_replay_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: Option<Value>,
    pub annotations: Option<Value>,
    pub status: Option<GraphDataStatus>,
}

pub struct GraphDataNodeInput {
    pub external_id: String,
    pub label: Option<String>,
    pub layer: Option<String>,
    pub weight: Option<f64>,
    pub is_partition: Option<bool>,
    pub belongs_to: Option<String>,
    pub comment: Option<String>,
    pub source_dataset_id: Option<i32>,
    pub attributes: Option<Value>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct GraphDataEdgeInput {
    pub external_id: String,
    pub source: String,
    pub target: String,
    pub label: Option<String>,
    pub layer: Option<String>,
    pub weight: Option<f64>,
    pub comment: Option<String>,
    pub source_dataset_id: Option<i32>,
    pub attributes: Option<Value>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
