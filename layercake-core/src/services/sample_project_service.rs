use chrono::Utc;
use include_dir::{include_dir, Dir, File};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use serde_yaml::Value;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::auth::Actor;
use crate::database::entities::common_types::{DataType, FileFormat};
use crate::database::entities::data_sets::{self};
use crate::database::entities::{plan_dag_edges, plan_dag_nodes, plans, projects};
use crate::errors::{CoreError, CoreResult};
use crate::services::{data_set_service::DataSetService, graph_service::GraphService};

static SAMPLE_PROJECT_DIR: Dir<'_> = include_dir!("../resources/sample-v1");

#[derive(Debug, Clone)]
pub struct SampleProjectMetadata {
    pub key: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
struct SampleFile {
    filename: String,
    contents: Vec<u8>,
}

#[derive(Debug)]
struct SampleProjectAssets {
    metadata: SampleProjectMetadata,
    nodes: SampleFile,
    edges: SampleFile,
    layers: SampleFile,
}

pub struct SampleProjectService {
    db: DatabaseConnection,
}

impl SampleProjectService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// List all bundled sample projects discovered under `sample/`
    pub fn list_available_projects() -> Vec<SampleProjectMetadata> {
        SAMPLE_PROJECT_DIR
            .dirs()
            .iter()
            .filter_map(|dir| {
                let key = match dir.path().file_name() {
                    Some(name) => name.to_string_lossy().to_string(),
                    None => return None,
                };

                match Self::extract_metadata(dir, key.clone()) {
                    Ok(metadata) => Some(metadata),
                    Err(err) => {
                        warn!("Skipping sample {key}: {err}");
                        None
                    }
                }
            })
            .collect()
    }

    /// Create a new project seeded with the requested sample bundle.
    pub async fn create_sample_project(
        &self,
        _actor: &Actor,
        sample_key: &str,
    ) -> CoreResult<projects::Model> {
        let assets = self.load_assets(sample_key)?;
        debug!("Creating sample project for key {}", sample_key);

        // Create project
        let mut project = projects::ActiveModel::new();
        project.name = Set(assets.metadata.name.clone());
        project.description = Set(assets.metadata.description.clone());
        let project = project
            .insert(&self.db)
            .await
            .map_err(|e| CoreError::internal("Failed to create sample project").with_source(e))?;

        // Create default plan for project
        let now = Utc::now();
        let plan = plans::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            project_id: Set(project.id),
            name: Set(format!("{} Plan", assets.metadata.name)),
            description: Set(None),
            tags: Set("[]".to_string()),
            yaml_content: Set(String::new()),
            dependencies: Set(None),
            status: Set("draft".to_string()),
            version: Set(1),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(&self.db)
        .await
        .map_err(|e| CoreError::internal("Failed to create sample project plan").with_source(e))?;

        // Create data sources for nodes, edges, layers
        let data_set_service = DataSetService::new(self.db.clone());
        let nodes_ds = self
            .create_sample_dataset(
                &data_set_service,
                &project,
                &assets.metadata,
                &assets.nodes,
                DataType::Nodes,
                "Nodes",
            )
            .await?;
        let edges_ds = self
            .create_sample_dataset(
                &data_set_service,
                &project,
                &assets.metadata,
                &assets.edges,
                DataType::Edges,
                "Edges",
            )
            .await?;
        let layers_ds = self
            .create_sample_dataset(
                &data_set_service,
                &project,
                &assets.metadata,
                &assets.layers,
                DataType::Layers,
                "Layers",
            )
            .await?;

        // Create Plan DAG nodes
        let mut dataset_nodes = Vec::new();
        for (index, ds) in [nodes_ds, edges_ds, layers_ds].into_iter().enumerate() {
            let node_id = format!("datasetnode_{}_{}", sample_key, ds.id);
            let metadata_json =
                serde_json::json!({ "label": ds.name, "description": ds.description });
            let config_json = serde_json::json!({
                "dataSetId": ds.id,
                "displayMode": "summary",
                "filename": ds.filename,
                "dataType": data_type_display(&ds.data_type),
            });

            let node = plan_dag_nodes::ActiveModel {
                id: Set(node_id.clone()),
                plan_id: Set(plan.id),
                node_type: Set("DataSetNode".to_string()),
                position_x: Set(120.0),
                position_y: Set(120.0 + (index as f64 * 140.0)),
                source_position: Set(None),
                target_position: Set(None),
                metadata_json: Set(metadata_json.to_string()),
                config_json: Set(config_json.to_string()),
                created_at: Set(now),
                updated_at: Set(now),
            };
            node.insert(&self.db).await.map_err(|e| {
                CoreError::internal("Failed to create sample dataset node").with_source(e)
            })?;
            dataset_nodes.push(node_id);
        }

        // Merge node that collates all data sources
        let merge_node_id = format!("mergenode_{}", Uuid::new_v4().simple());
        let merge_metadata = serde_json::json!({ "label": "Combine Data Sources" });
        let merge_config = serde_json::json!({
            "mergeStrategy": "Union",
            "conflictResolution": "PreferLast"
        });

        plan_dag_nodes::ActiveModel {
            id: Set(merge_node_id.clone()),
            plan_id: Set(plan.id),
            node_type: Set("MergeNode".to_string()),
            position_x: Set(360.0),
            position_y: Set(220.0),
            source_position: Set(None),
            target_position: Set(None),
            metadata_json: Set(merge_metadata.to_string()),
            config_json: Set(merge_config.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(&self.db)
        .await
        .map_err(|e| CoreError::internal("Failed to create sample merge node").with_source(e))?;

        // Graph node that consumes merge output
        let graph_service = GraphService::new(self.db.clone());
        let graph_node_id = format!("graphnode_{}", Uuid::new_v4().simple());
        let graph_label = format!("{} Graph", assets.metadata.name);
        let graph_metadata_json = serde_json::json!({ "label": graph_label }).to_string();

        // Insert Plan DAG node first to satisfy FK constraints
        plan_dag_nodes::ActiveModel {
            id: Set(graph_node_id.clone()),
            plan_id: Set(plan.id),
            node_type: Set("GraphNode".to_string()),
            position_x: Set(620.0),
            position_y: Set(220.0),
            source_position: Set(None),
            target_position: Set(None),
            metadata_json: Set(graph_metadata_json.clone()),
            config_json: Set(serde_json::json!({}).to_string()),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(&self.db)
        .await
        .map_err(|e| CoreError::internal("Failed to create sample graph node").with_source(e))?;

        let graph = graph_service
            .create_graph(project.id, graph_label.clone(), Some(graph_node_id.clone()))
            .await?;

        // Update Plan DAG node config with graph metadata
        if let Some(existing_graph_node) = plan_dag_nodes::Entity::find_by_id(&graph_node_id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal("Failed to load sample graph node").with_source(e))?
        {
            let mut graph_node_active: plan_dag_nodes::ActiveModel = existing_graph_node.into();
            graph_node_active.config_json = Set(serde_json::json!({
                "projectId": project.id,
                "graphId": graph.id,
                "metadata": {
                    "nodeCount": graph.node_count,
                    "edgeCount": graph.edge_count,
                    "lastModified": graph.updated_at.to_rfc3339(),
                }
            })
            .to_string());
            graph_node_active.metadata_json = Set(graph_metadata_json);
            graph_node_active.updated_at = Set(Utc::now());
            graph_node_active.update(&self.db).await.map_err(|e| {
                CoreError::internal("Failed to update sample graph node").with_source(e)
            })?;
        }

        // Connect data sources -> merge node
        for ds_id in dataset_nodes {
            let edge_id = format!("edge_{}", Uuid::new_v4().simple());
            plan_dag_edges::ActiveModel {
                id: Set(edge_id),
                plan_id: Set(plan.id),
                source_node_id: Set(ds_id),
                target_node_id: Set(merge_node_id.clone()),
                // Removed source_handle and target_handle for floating edges
                metadata_json: Set(edge_metadata_json()),
                created_at: Set(now),
                updated_at: Set(now),
            }
            .insert(&self.db)
            .await
            .map_err(|e| CoreError::internal("Failed to connect sample datasets").with_source(e))?;
        }

        // Connect merge node -> graph node
        plan_dag_edges::ActiveModel {
            id: Set(format!("edge_{}", Uuid::new_v4().simple())),
            plan_id: Set(plan.id),
            source_node_id: Set(merge_node_id),
            target_node_id: Set(graph_node_id),
            // Removed source_handle and target_handle for floating edges
            metadata_json: Set(edge_metadata_json()),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(&self.db)
        .await
        .map_err(|e| CoreError::internal("Failed to connect sample graph").with_source(e))?;

        Ok(project)
    }

    fn load_assets(&self, sample_key: &str) -> CoreResult<SampleProjectAssets> {
        let dir = SAMPLE_PROJECT_DIR
            .get_dir(sample_key)
            .ok_or_else(|| CoreError::not_found("SampleProject", sample_key.to_string()))?;

        let metadata = Self::extract_metadata(&dir, sample_key.to_string())?;
        let nodes = Self::read_sample_file(&dir, &["nodes.csv"], sample_key, "nodes")?;
        let edges = Self::read_sample_file(&dir, &["edges.csv", "links.csv"], sample_key, "edges")?;
        let layers = Self::read_sample_file(&dir, &["layers.csv"], sample_key, "layers")?;

        Ok(SampleProjectAssets {
            metadata,
            nodes,
            edges,
            layers,
        })
    }

    fn extract_metadata(dir: &Dir<'_>, key: String) -> CoreResult<SampleProjectMetadata> {
        let plan_yaml = dir
            .get_file("plan.yaml")
            .and_then(|plan| serde_yaml::from_slice::<Value>(plan.contents()).ok());

        let (name, description) = plan_yaml
            .as_ref()
            .and_then(|yaml| yaml.get("metadata"))
            .map(|meta| {
                let name = meta
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| to_title_case(&key));
                let description = meta
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                (name, description)
            })
            .unwrap_or_else(|| (to_title_case(&key), None));

        Ok(SampleProjectMetadata {
            key,
            name,
            description,
        })
    }

    fn read_sample_file(
        dir: &Dir<'_>,
        candidate_names: &[&str],
        sample_key: &str,
        label: &str,
    ) -> CoreResult<SampleFile> {
        if let Some(file) = Self::find_file(dir, candidate_names) {
            let filename = file
                .path()
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| candidate_names.first().unwrap_or(&label).to_string());
            let contents = file.contents().to_vec();
            Ok(SampleFile { filename, contents })
        } else {
            Err(CoreError::not_found(
                "SampleProjectAsset",
                format!("{sample_key}:{label}"),
            ))
        }
    }

    fn find_file<'a>(dir: &'a Dir<'a>, candidate_names: &[&str]) -> Option<&'a File<'a>> {
        for file in dir.files() {
            if let Some(name) = file.path().file_name().and_then(|os| os.to_str()) {
                if candidate_names
                    .iter()
                    .any(|candidate| name.eq_ignore_ascii_case(candidate))
                {
                    return Some(file);
                }
            }
        }
        for subdir in dir.dirs() {
            if let Some(file) = Self::find_file(subdir, candidate_names) {
                return Some(file);
            }
        }
        None
    }

    async fn create_sample_dataset(
        &self,
        data_set_service: &DataSetService,
        project: &projects::Model,
        metadata: &SampleProjectMetadata,
        file: &SampleFile,
        data_type: DataType,
        label: &str,
    ) -> CoreResult<data_sets::Model> {
        let description = format!("{} {} sample dataset", metadata.name, label);
        data_set_service
            .create_from_file(
                project.id,
                format!("{} {}", metadata.name, label),
                Some(description),
                file.filename.clone(),
                FileFormat::Csv,
                file.contents.clone(),
                Some(data_type),
            )
            .await
    }
}

fn to_title_case(input: &str) -> String {
    input
        .split(|c: char| c == '_' || c == '-' || c.is_whitespace())
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => format!(
                    "{}{}",
                    first.to_uppercase().collect::<String>(),
                    chars.as_str().to_lowercase()
                ),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn data_type_display(data_type: &str) -> String {
    match data_type.to_lowercase().as_str() {
        "nodes" => "Nodes".to_string(),
        "edges" | "links" => "Edges".to_string(),
        "layers" => "Layers".to_string(),
        other => other.to_string(),
    }
}

fn edge_metadata_json() -> String {
    serde_json::json!({
        "label": null,
        "dataType": "GraphData"
    })
    .to_string()
}
