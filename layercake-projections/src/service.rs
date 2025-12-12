use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use std::{fs, path::Path};

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde_json;
use tokio::sync::{broadcast, RwLock};
use zip::write::FileOptions;

use crate::entities::{
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
pub struct ProjectionExportBundle {
    pub filename: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
struct ProjectionBuildAssets {
    index_html: String,
    assets: Vec<(String, Vec<u8>)>,
}

#[derive(Clone, Debug)]
pub struct ProjectionGraphView {
    pub nodes: Vec<ProjectionGraphNode>,
    pub edges: Vec<ProjectionGraphEdge>,
    pub layers: Vec<ProjectionLayer>,
}

#[derive(Clone, Debug)]
pub struct ProjectionGraphNode {
    pub id: String,
    pub label: Option<String>,
    pub layer: Option<String>,
    pub weight: Option<f64>,
    pub attributes: Option<serde_json::Value>,
    pub color: Option<String>,
    pub label_color: Option<String>,
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

#[derive(Clone, Debug)]
pub struct ProjectionLayer {
    pub layer_id: String,
    pub name: String,
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub border_color: Option<String>,
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
        let resolved_graph_id = self
            .resolve_graph_id(input.project_id, input.graph_id)
            .await?;
        self.ensure_graph_in_project(input.project_id, resolved_graph_id)
            .await?;

        let now = Utc::now();
        let model = projections::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            project_id: Set(input.project_id),
            graph_id: Set(resolved_graph_id),
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

    async fn load_layer_palette(
        &self,
        project_id: i32,
    ) -> Result<Vec<ProjectionLayer>, sea_orm::DbErr> {
        use crate::entities::project_layers;

        let rows = project_layers::Entity::find()
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .order_by_asc(project_layers::Column::LayerId)
            .all(&self.db)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| ProjectionLayer {
                layer_id: row.layer_id,
                name: row.name,
                background_color: Some(row.background_color),
                text_color: Some(row.text_color),
                border_color: Some(row.border_color),
            })
            .collect())
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
            .order_by_asc(graph_data_nodes::Column::Id)
            .all(&self.db)
            .await?;

        let edges = graph_data_edges::Entity::find()
            .filter(graph_data_edges::Column::GraphDataId.eq(projection.graph_id))
            .order_by_asc(graph_data_edges::Column::Id)
            .all(&self.db)
            .await?;

        let layers = self
            .load_layer_palette(projection.project_id)
            .await
            .unwrap_or_default();
        let layer_lookup: HashMap<String, ProjectionLayer> = layers
            .iter()
            .map(|l| (l.layer_id.clone(), l.clone()))
            .collect();

        let view = ProjectionGraphView {
            nodes: nodes
                .into_iter()
                .map(|n| {
                    let layer_key = n.layer.clone();
                    let layer_colors = layer_key
                        .as_ref()
                        .and_then(|l| layer_lookup.get(l))
                        .cloned();
                    ProjectionGraphNode {
                        id: n.external_id,
                        label: n.label,
                        layer: layer_key,
                        weight: n.weight,
                        attributes: n.attributes,
                        color: layer_colors
                            .as_ref()
                            .and_then(|l| l.background_color.clone()),
                        label_color: layer_colors.as_ref().and_then(|l| l.text_color.clone()),
                    }
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
            layers,
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
        let projection = self.get(projection_id).await?.ok_or_else(|| {
            sea_orm::DbErr::RecordNotFound(format!("projection {}", projection_id))
        })?;

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

    pub async fn export_bundle(
        &self,
        projection_id: i32,
    ) -> Result<ProjectionExportBundle, sea_orm::DbErr> {
        let payload = self.export_payload(projection_id).await?;
        let projection = self.get(projection_id).await?.ok_or_else(|| {
            sea_orm::DbErr::RecordNotFound(format!("projection {}", projection_id))
        })?;

        let build_assets = Self::read_projection_build();
        let mut buffer: Vec<u8> = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut buffer);
            let mut zip = zip::ZipWriter::new(cursor);
            let options = FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(0o644);

            let force_graph_js = Self::read_force_graph_bundle();

            // Prefer built projection frontend if available
            if let Some(build) = build_assets.clone() {
                zip.start_file("index.html", options)
                    .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
                zip.write_all(build.index_html.as_bytes())
                    .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
                for (name, bytes) in build.assets {
                    let asset_path = format!("assets/{}", name);
                    zip.start_file(asset_path, options)
                        .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
                    zip.write_all(&bytes)
                        .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
                }
            } else {
                let index_html = r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Projection Viewer</title>
    <style>
      html, body, #root { margin: 0; padding: 0; width: 100%; height: 100%; background: #0b1021; color: #e9edf7; font-family: sans-serif; }
      .fallback { padding: 16px; white-space: pre-wrap; overflow: auto; }
      canvas { outline: none; }
    </style>
  </head>
  <body>
    <div id="root"></div>
    <script src="./3d-force-graph.min.js"></script>
    <script src="./data.js"></script>
    <script src="./projection.js"></script>
  </body>
</html>
"#;

                zip.start_file("index.html", options)
                    .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
                zip.write_all(index_html.as_bytes())
                    .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
            }

            let data_js = format!(
                "window.PROJECTION_EXPORT = {};\n",
                serde_json::to_string(&payload)
                    .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?
            );
            zip.start_file("data.js", options)
                .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
            zip.write_all(data_js.as_bytes())
                .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;

            let projection_js = r#"(() => {
  const data = window.PROJECTION_EXPORT || {};
  const root = document.getElementById('root');
  const graphData = data.graph || { nodes: [], edges: [] };
  if (window.ForceGraph3D) {
    const elem = document.createElement('div');
    elem.style.width = '100%';
    elem.style.height = '100%';
    root.appendChild(elem);
    const fg = ForceGraph3D()(elem)
      .graphData({
        nodes: graphData.nodes.map(n => ({ id: n.id, name: n.label || n.id, layer: n.layer })),
        links: graphData.edges.map(e => ({ id: e.id, source: e.source, target: e.target, label: e.label, layer: e.layer })),
      })
      .nodeLabel('name')
      .linkDirectionalParticles(0)
      .linkColor(() => '#6ddcff')
      .nodeColor(node => node.layer ? '#ffd166' : '#6ddcff')
      .backgroundColor('#0b1021')
      .showNavInfo(false);
    window.initProjection = () => fg; // allow override
    return;
  }
  // Fallback render as JSON
  const pre = document.createElement('pre');
  pre.className = 'fallback';
  pre.textContent = JSON.stringify(data, null, 2);
  root.appendChild(pre);
})();
"#;
            zip.start_file("projection.js", options)
                .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
            zip.write_all(projection_js.as_bytes())
                .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;

            if let Some(force_src) = force_graph_js {
                zip.start_file("3d-force-graph.min.js", options)
                    .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
                zip.write_all(force_src.as_bytes())
                    .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
            }

            zip.finish()
                .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
        }

        let filename = format!(
            "{}-projection.zip",
            projection
                .name
                .to_lowercase()
                .replace(' ', "-")
                .replace('/', "-")
        );

        Ok(ProjectionExportBundle {
            filename,
            bytes: buffer,
        })
    }

    fn read_force_graph_bundle() -> Option<String> {
        // Attempt to embed 3d-force-graph bundle for offline export; fallback to None if missing.
        let candidates = [
            "frontend/node_modules/3d-force-graph/dist/3d-force-graph.min.js",
            "node_modules/3d-force-graph/dist/3d-force-graph.min.js",
            "projections-frontend/node_modules/3d-force-graph/dist/3d-force-graph.min.js",
        ];

        for path in candidates {
            let p = Path::new(path);
            if p.exists() {
                if let Ok(contents) = fs::read_to_string(p) {
                    return Some(contents);
                }
            }
        }

        None
    }

    fn read_projection_build() -> Option<ProjectionBuildAssets> {
        let base = Path::new("projections-frontend/dist");
        if !base.exists() {
            return None;
        }

        let index_path = base.join("index.html");
        let index_html = fs::read_to_string(&index_path).ok()?;
        // Make asset URLs relative so exported bundle works offline
        let mut rewritten = index_html
            .replace("src=\"/assets/", "src=\"./assets/")
            .replace("src=\"/projections/viewer/assets/", "src=\"./assets/")
            .replace("href=\"/assets/", "href=\"./assets/")
            .replace("href=\"/projections/viewer/assets/", "href=\"./assets/")
            .replace("base href=\"/projections/viewer/\"", ""); // drop base to keep relative navigation

        rewritten = rewritten.replace(
            "</body>",
            r#"  <script src="./data.js"></script>
    <script src="./projection.js"></script>
  </body>"#,
        );

        let assets_dir = base.join("assets");
        let mut assets = Vec::new();
        if assets_dir.exists() {
            if let Ok(entries) = fs::read_dir(&assets_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Ok(bytes) = fs::read(&path) {
                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                assets.push((name.to_string(), bytes));
                            }
                        }
                    }
                }
            }
        }

        Some(ProjectionBuildAssets {
            index_html: rewritten,
            assets,
        })
    }

    async fn resolve_graph_id(
        &self,
        project_id: i32,
        graph_id: i32,
    ) -> Result<i32, sea_orm::DbErr> {
        // Happy path: graph_data already exists with this ID
        if let Some(graph) = graph_data::Entity::find_by_id(graph_id)
            .one(&self.db)
            .await?
        {
            if graph.project_id != project_id {
                return Err(sea_orm::DbErr::Custom(
                    "graph does not belong to project".to_string(),
                ));
            }
            return Ok(graph.id);
        }

        // Legacy path: lookup in old graphs table
        use crate::entities::graphs;
        let old_graph = graphs::Entity::find_by_id(graph_id).one(&self.db).await?;

        let Some(old_graph) = old_graph else {
            return Err(sea_orm::DbErr::RecordNotFound(format!(
                "graph_data or graphs {}",
                graph_id
            )));
        };

        if old_graph.project_id != project_id {
            return Err(sea_orm::DbErr::Custom(
                "graph does not belong to project".to_string(),
            ));
        }

        // If there is already a graph_data row for the same DAG node, reuse it
        if let Some(existing) = graph_data::Entity::find()
            .filter(graph_data::Column::ProjectId.eq(project_id))
            .filter(graph_data::Column::DagNodeId.eq(old_graph.node_id.clone()))
            .one(&self.db)
            .await?
        {
            return Ok(existing.id);
        }

        // Otherwise, materialize a minimal graph_data record so the FK insert succeeds
        let status = match old_graph.execution_state.as_str() {
            "ERROR" => "error",
            "COMPLETED" => "active",
            _ => "processing",
        };

        let annotations = if let Some(text) = old_graph.annotations.clone() {
            serde_json::from_str::<serde_json::Value>(&text)
                .unwrap_or_else(|_| serde_json::json!([text]))
        } else {
            serde_json::json!([])
        };

        let model = graph_data::ActiveModel {
            id: Set(old_graph.id),
            project_id: Set(old_graph.project_id),
            name: Set(old_graph.name.clone()),
            source_type: Set("computed".to_string()),
            dag_node_id: Set(Some(old_graph.node_id.clone())),
            file_format: Set(None),
            origin: Set(None),
            filename: Set(None),
            blob: Set(None),
            file_size: Set(None),
            processed_at: Set(old_graph.computed_date),
            source_hash: Set(old_graph.source_hash.clone()),
            computed_date: Set(old_graph.computed_date),
            last_edit_sequence: Set(old_graph.last_edit_sequence),
            has_pending_edits: Set(old_graph.has_pending_edits),
            last_replay_at: Set(old_graph.last_replay_at),
            node_count: Set(old_graph.node_count),
            edge_count: Set(old_graph.edge_count),
            error_message: Set(old_graph.error_message.clone()),
            metadata: Set(old_graph.metadata.clone()),
            annotations: Set(Some(annotations)),
            status: Set(status.to_string()),
            created_at: Set(old_graph.created_at),
            updated_at: Set(old_graph.updated_at),
        };

        let inserted = model.insert(&self.db).await?;
        Ok(inserted.id)
    }

    async fn ensure_graph_in_project(
        &self,
        project_id: i32,
        graph_id: i32,
    ) -> Result<(), sea_orm::DbErr> {
        // Try to find in graph_data first
        let graph = graph_data::Entity::find_by_id(graph_id)
            .one(&self.db)
            .await?;

        if let Some(graph) = graph {
            if graph.project_id != project_id {
                return Err(sea_orm::DbErr::Custom(
                    "graph does not belong to project".to_string(),
                ));
            }
            return Ok(());
        }

        // Fallback: Check if this ID exists in the old graphs table (pre-migration)
        // This allows projections to work before migration runs
        use crate::entities::graphs;
        let old_graph = graphs::Entity::find_by_id(graph_id).one(&self.db).await?;

        let Some(old_graph) = old_graph else {
            return Err(sea_orm::DbErr::RecordNotFound(format!(
                "graph_data or graphs {}",
                graph_id
            )));
        };

        if old_graph.project_id != project_id {
            return Err(sea_orm::DbErr::Custom(
                "graph does not belong to project".to_string(),
            ));
        }

        Ok(())
    }
}
