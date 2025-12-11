use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use tokio::sync::{broadcast, RwLock};

use crate::database::entities::{
    graph_data, graph_data_edges, graph_data_nodes,
    projections::{self, Entity as Projections},
};

#[derive(Clone)]
pub struct ProjectionService {
    db: DatabaseConnection,
    state_store: Arc<RwLock<HashMap<i32, serde_json::Value>>>,
    state_tx: broadcast::Sender<ProjectionStateEvent>,
    graph_tx: broadcast::Sender<ProjectionGraphEvent>,
}

#[derive(Clone, Debug)]
pub struct ProjectionStateEvent {
    pub projection_id: i32,
    pub projection_type: String,
    pub state: serde_json::Value,
}

#[derive(Clone, Debug)]
pub struct ProjectionGraphEvent {
    pub projection_id: i32,
    pub graph: ProjectionGraphView,
}

#[derive(Clone, Debug)]
pub struct ProjectionGraphView {
    pub nodes: Vec<ProjectionGraphNode>,
    pub edges: Vec<ProjectionGraphEdge>,
}

#[derive(Clone, Debug)]
pub struct ProjectionGraphNode {
    pub id: String,
    pub label: Option<String>,
    pub layer: Option<String>,
    pub weight: Option<f64>,
    pub attributes: Option<serde_json::Value>,
}

#[derive(Clone, Debug)]
pub struct ProjectionGraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: Option<String>,
    pub layer: Option<String>,
    pub weight: Option<f64>,
    pub attributes: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct ProjectionCreateInput {
    pub project_id: i32,
    pub graph_id: i32,
    pub name: String,
    pub projection_type: String,
    pub settings_json: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct ProjectionUpdateInput {
    pub name: Option<String>,
    pub projection_type: Option<String>,
    pub settings_json: Option<Option<serde_json::Value>>,
}

impl ProjectionService {
    pub fn new(db: DatabaseConnection) -> Self {
        let (state_tx, _) = broadcast::channel(64);
        let (graph_tx, _) = broadcast::channel(64);

        Self {
            db,
            state_store: Arc::new(RwLock::new(HashMap::new())),
            state_tx,
            graph_tx,
        }
    }

    pub async fn list_by_project(
        &self,
        project_id: i32,
    ) -> Result<Vec<projections::Model>, sea_orm::DbErr> {
        Projections::find()
            .filter(projections::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
    }

    pub async fn get(&self, id: i32) -> Result<Option<projections::Model>, sea_orm::DbErr> {
        Projections::find_by_id(id).one(&self.db).await
    }

    pub async fn create(
        &self,
        input: ProjectionCreateInput,
    ) -> Result<projections::Model, sea_orm::DbErr> {
        self.ensure_graph_in_project(input.project_id, input.graph_id)
            .await?;

        let now = Utc::now();
        let model = projections::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            project_id: Set(input.project_id),
            graph_id: Set(input.graph_id),
            name: Set(input.name),
            projection_type: Set(input.projection_type),
            settings_json: Set(input.settings_json),
            created_at: Set(now),
            updated_at: Set(now),
        };

        model.insert(&self.db).await
    }

    pub async fn update_settings(
        &self,
        projection_id: i32,
        settings_json: Option<serde_json::Value>,
    ) -> Result<projections::Model, sea_orm::DbErr> {
        let Some(existing) = self.get(projection_id).await? else {
            return Err(sea_orm::DbErr::RecordNotFound(format!(
                "projection {}",
                projection_id
            )));
        };

        let mut active: projections::ActiveModel = existing.clone().into();
        active.settings_json = Set(settings_json.clone());
        active.updated_at = Set(Utc::now());

        let updated = active.update(&self.db).await?;

        if let Some(settings) = settings_json {
            let _ = self.state_tx.send(ProjectionStateEvent {
                projection_id,
                projection_type: existing.projection_type,
                state: settings,
            });
        }

        Ok(updated)
    }

    pub async fn update(
        &self,
        projection_id: i32,
        input: ProjectionUpdateInput,
    ) -> Result<projections::Model, sea_orm::DbErr> {
        let Some(existing) = self.get(projection_id).await? else {
            return Err(sea_orm::DbErr::RecordNotFound(format!(
                "projection {}",
                projection_id
            )));
        };

        let mut active: projections::ActiveModel = existing.clone().into();

        if let Some(name) = input.name {
            active.name = Set(name);
        }

        if let Some(pt) = input.projection_type {
            active.projection_type = Set(pt.clone());
        }

        if let Some(settings) = input.settings_json.clone() {
            active.settings_json = Set(settings.clone());
        }

        active.updated_at = Set(Utc::now());

        let updated = active.update(&self.db).await?;

        if let Some(Some(settings)) = input.settings_json {
            let _ = self.state_tx.send(ProjectionStateEvent {
                projection_id,
                projection_type: updated.projection_type.clone(),
                state: settings,
            });
        }

        Ok(updated)
    }

    pub async fn delete(&self, projection_id: i32) -> Result<u64, sea_orm::DbErr> {
        use sea_orm::ActiveValue::Set as AVSet;
        let result = projections::ActiveModel {
            id: AVSet(projection_id),
            ..Default::default()
        }
        .delete(&self.db)
        .await?;

        Ok(result.rows_affected)
    }

    pub async fn load_graph(
        &self,
        projection_id: i32,
    ) -> Result<ProjectionGraphView, sea_orm::DbErr> {
        let projection = self.get(projection_id).await?.ok_or_else(|| {
            sea_orm::DbErr::RecordNotFound(format!("projection {}", projection_id))
        })?;

        self.ensure_graph_in_project(projection.project_id, projection.graph_id)
            .await?;

        let nodes = graph_data_nodes::Entity::find()
            .filter(graph_data_nodes::Column::GraphDataId.eq(projection.graph_id))
            .all(&self.db)
            .await?;

        let edges = graph_data_edges::Entity::find()
            .filter(graph_data_edges::Column::GraphDataId.eq(projection.graph_id))
            .all(&self.db)
            .await?;

        let view = ProjectionGraphView {
            nodes: nodes
                .into_iter()
                .map(|n| ProjectionGraphNode {
                    id: n.external_id,
                    label: n.label,
                    layer: n.layer,
                    weight: n.weight,
                    attributes: n.attributes,
                })
                .collect(),
            edges: edges
                .into_iter()
                .map(|e| ProjectionGraphEdge {
                    id: e.external_id,
                    source: e.source,
                    target: e.target,
                    label: e.label,
                    layer: e.layer,
                    weight: e.weight,
                    attributes: e.attributes,
                })
                .collect(),
        };

        let _ = self.graph_tx.send(ProjectionGraphEvent {
            projection_id,
            graph: view.clone(),
        });

        Ok(view)
    }

    pub async fn save_state(
        &self,
        projection_id: i32,
        state: serde_json::Value,
    ) -> Result<(), sea_orm::DbErr> {
        let Some(existing) = self.get(projection_id).await? else {
            return Err(sea_orm::DbErr::RecordNotFound(format!(
                "projection {}",
                projection_id
            )));
        };

        let mut active: projections::ActiveModel = existing.clone().into();
        active.settings_json = Set(Some(state.clone()));
        active.updated_at = Set(Utc::now());
        active.update(&self.db).await?;

        {
            let mut store = self.state_store.write().await;
            store.insert(projection_id, state.clone());
        }

        let _ = self.state_tx.send(ProjectionStateEvent {
            projection_id,
            projection_type: existing.projection_type,
            state,
        });

        Ok(())
    }

    pub async fn get_state(&self, projection_id: i32) -> Option<serde_json::Value> {
        let store = self.state_store.read().await;
        store.get(&projection_id).cloned()
    }

    pub fn subscribe_state(&self) -> broadcast::Receiver<ProjectionStateEvent> {
        self.state_tx.subscribe()
    }

    pub fn subscribe_graph(&self) -> broadcast::Receiver<ProjectionGraphEvent> {
        self.graph_tx.subscribe()
    }

    pub async fn export_payload(
        &self,
        projection_id: i32,
    ) -> Result<serde_json::Value, sea_orm::DbErr> {
        let projection = self
            .get(projection_id)
            .await?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound(format!("projection {}", projection_id)))?;

        let graph = self.load_graph(projection_id).await?;
        let state = self
            .get_state(projection_id)
            .await
            .or(projection.settings_json.clone());

        let payload = serde_json::json!({
            "projection": {
                "id": projection.id,
                "projectId": projection.project_id,
                "graphId": projection.graph_id,
                "name": projection.name,
                "projectionType": projection.projection_type,
                "updatedAt": projection.updated_at,
            },
            "state": state,
            "graph": {
                "nodes": graph.nodes
                    .into_iter()
                    .map(|n| serde_json::json!({
                        "id": n.id,
                        "label": n.label,
                        "layer": n.layer,
                        "weight": n.weight,
                        "attributes": n.attributes,
                    }))
                    .collect::<Vec<_>>(),
                "edges": graph.edges
                    .into_iter()
                    .map(|e| serde_json::json!({
                        "id": e.id,
                        "source": e.source,
                        "target": e.target,
                        "label": e.label,
                        "layer": e.layer,
                        "weight": e.weight,
                        "attributes": e.attributes,
                    }))
                    .collect::<Vec<_>>(),
            }
        });

        Ok(payload)
    }

    async fn ensure_graph_in_project(
        &self,
        project_id: i32,
        graph_id: i32,
    ) -> Result<(), sea_orm::DbErr> {
        let graph = graph_data::Entity::find_by_id(graph_id)
            .one(&self.db)
            .await?;

        let Some(graph) = graph else {
            return Err(sea_orm::DbErr::RecordNotFound(format!(
                "graph_data {}",
                graph_id
            )));
        };

        if graph.project_id != project_id {
            return Err(sea_orm::DbErr::Custom(
                "graph does not belong to project".to_string(),
            ));
        }

        Ok(())
    }
}
