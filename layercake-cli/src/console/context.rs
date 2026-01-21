#![cfg(feature = "console")]

use std::{fmt, sync::Arc};

use anyhow::{anyhow, Context as _, Result};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};

use layercake_core::database::entities::{graph_data, graph_data_edges, projects, users};
use layercake_core::services::system_settings_service::SystemSettingsService;

use super::{
    commands::SettingsCommand,
    output::{print_table, TableRow},
};

/// Get or create a default user for console/development use
async fn get_or_create_default_user(db: &DatabaseConnection) -> Result<users::Model> {
    use sea_orm::{ActiveModelTrait, Set};

    // Try to find existing default user
    if let Some(user) = users::Entity::find()
        .filter(users::Column::Username.eq("default"))
        .one(db)
        .await?
    {
        return Ok(user);
    }

    // Create default user if it doesn't exist
    let now = chrono::Utc::now();
    let default_user = users::ActiveModel {
        email: Set("default@layercake.local".to_string()),
        username: Set("default".to_string()),
        display_name: Set("Default User".to_string()),
        password_hash: Set("".to_string()), // No password for default user
        avatar_color: Set("#3b82f6".to_string()),
        is_active: Set(true),
        user_type: Set("human".to_string()),
        scoped_project_id: Set(None),
        api_key_hash: Set(None),
        organisation_id: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        last_login_at: Set(None),
        ..Default::default()
    };

    let user = default_user.insert(db).await?;
    Ok(user)
}

/// Active console runtime state shared across command handlers.
pub struct ConsoleContext {
    pub(crate) db: DatabaseConnection,
    pub(crate) system_settings: Arc<SystemSettingsService>,
    selected_project: Option<ProjectSelection>,
}

#[derive(Clone)]
struct ProjectSelection {
    id: i32,
    name: String,
}

impl fmt::Display for ProjectSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.id)
    }
}

impl ConsoleContext {
    pub async fn bootstrap(db: DatabaseConnection) -> Result<Self> {
        let system_settings = Arc::new(
            SystemSettingsService::new(db.clone())
                .await
                .map_err(anyhow::Error::from)?,
        );
        Ok(Self {
            db,
            system_settings,
            selected_project: None,
        })
    }

    pub fn prompt_label(&self) -> String {
        match &self.selected_project {
            Some(project) => format!("layercake({})", project.id),
            None => "layercake".to_string(),
        }
    }

    pub async fn list_projects(&self) -> Result<()> {
        let mut rows = Vec::new();
        let records = projects::Entity::find()
            .order_by_asc(projects::Column::Id)
            .all(&self.db)
            .await?;

        for project in records {
            rows.push(TableRow::from(vec![
                project.id.to_string(),
                project.name,
                project.description.unwrap_or_else(|| "-".to_string()),
            ]));
        }

        if rows.is_empty() {
            println!("No projects found.");
        } else {
            print_table(&["id", "name", "description"], &rows);
        }
        Ok(())
    }

    pub async fn select_project(&mut self, project_id: i32) -> Result<()> {
        let project = projects::Entity::find_by_id(project_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Project {project_id} not found"))?;

        self.selected_project = Some(ProjectSelection {
            id: project.id,
            name: project.name.clone(),
        });

        println!("Project set to {}.", project.name);
        Ok(())
    }

    pub async fn list_graphs(&self, project_override: Option<i32>) -> Result<()> {
        let project_id = project_override
            .or_else(|| self.selected_project.as_ref().map(|p| p.id))
            .ok_or_else(|| anyhow!("Select a project first or pass --project <id>"))?;

        let mut rows = Vec::new();
        let graphs = graph_data::Entity::find()
            .filter(graph_data::Column::ProjectId.eq(project_id))
            .order_by_asc(graph_data::Column::Id)
            .all(&self.db)
            .await?;

        for graph in graphs {
            rows.push(TableRow::from(vec![
                graph.id.to_string(),
                graph.name,
                graph.node_count.to_string(),
                graph.edge_count.to_string(),
                graph.status,
            ]));
        }

        if rows.is_empty() {
            println!("No graphs found for project {project_id}.");
        } else {
            print_table(&["id", "name", "nodes", "edges", "state"], &rows);
        }
        Ok(())
    }

    pub async fn show_graph(&self, graph_id: i32) -> Result<()> {
        let graph = graph_data::Entity::find_by_id(graph_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("GraphData {graph_id} not found"))?;

        let edge_count = graph_data_edges::Entity::find()
            .filter(graph_data_edges::Column::GraphDataId.eq(graph_id))
            .count(&self.db)
            .await?;

        println!("Graph {graph_id}");
        println!("  name: {}", graph.name);
        println!("  project: {}", graph.project_id);
        println!("  nodes: {}", graph.node_count);
        println!(
            "  edges: {} (recorded {} snapshots)",
            graph.edge_count, edge_count
        );
        println!("  status: {}", graph.status);
        if let Some(node_id) = &graph.dag_node_id {
            println!("  dag node: {node_id}");
        }
        if let Some(err) = graph.error_message {
            println!("  error: {err}");
        }

        Ok(())
    }

    pub async fn handle_settings_command(&self, command: SettingsCommand) -> Result<()> {
        match command {
            SettingsCommand::List => self.print_settings_overview().await,
            SettingsCommand::Show { key } => self.show_setting(&key).await,
            SettingsCommand::Set { key, value } => self.set_setting(&key, &value).await,
        }
    }

    async fn print_settings_overview(&self) -> Result<()> {
        let settings = self
            .system_settings
            .list_settings()
            .await
            .context("failed to list system settings")?;

        if settings.is_empty() {
            println!("No runtime settings are registered.");
            return Ok(());
        }

        let mut rows = Vec::new();
        for setting in settings {
            let display_value = setting.value.unwrap_or_else(|| "<hidden>".to_string());
            rows.push(TableRow::from(vec![
                setting.key,
                display_value,
                setting.category,
            ]));
        }

        print_table(&["key", "value", "category"], &rows);
        Ok(())
    }

    async fn show_setting(&self, key: &str) -> Result<()> {
        let setting = self
            .system_settings
            .get_setting(key)
            .await
            .with_context(|| format!("failed to load setting {}", key))?;

        println!("{} ({})", setting.label, setting.key);
        println!("category: {}", setting.category);
        if let Some(description) = &setting.description {
            println!("description: {}", description);
        }
        if setting.is_secret {
            println!("value: <hidden>");
        } else {
            println!("value: {}", setting.raw_value);
        }
        if !setting.allowed_values.is_empty() {
            println!("allowed values: {}", setting.allowed_values.join(", "));
        }
        println!(
            "updated: {} | read-only: {}",
            setting.updated_at, setting.is_read_only
        );
        Ok(())
    }

    async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let updated = self
            .system_settings
            .update_setting(key, value.to_string())
            .await
            .with_context(|| format!("failed to update setting {}", key))?;

        let display_value = updated
            .value
            .clone()
            .unwrap_or_else(|| "<hidden>".to_string());
        println!("{} set to {}", updated.label, display_value);
        Ok(())
    }
}
