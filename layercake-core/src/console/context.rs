#![cfg(feature = "console")]

use std::fmt;

use anyhow::{anyhow, Context as _, Result};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};

use crate::database::entities::{graph_edges, graphs, projects};

use super::{
    chat::{ChatConfig, ChatProvider},
    output::{print_table, TableRow},
};

/// Active console runtime state shared across command handlers.
pub struct ConsoleContext {
    pub(crate) db: DatabaseConnection,
    pub(crate) chat_config: ChatConfig,
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
        let chat_config = ChatConfig::load(&db).await?;
        Ok(Self {
            db,
            chat_config,
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
        let graphs = graphs::Entity::find()
            .filter(graphs::Column::ProjectId.eq(project_id))
            .order_by_asc(graphs::Column::Id)
            .all(&self.db)
            .await?;

        for graph in graphs {
            rows.push(TableRow::from(vec![
                graph.id.to_string(),
                graph.name,
                graph.node_count.to_string(),
                graph.edge_count.to_string(),
                graph.execution_state,
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
        let graph = graphs::Entity::find_by_id(graph_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Graph {graph_id} not found"))?;

        let edge_count = graph_edges::Entity::find()
            .filter(graph_edges::Column::GraphId.eq(graph_id))
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
        println!("  state: {}", graph.execution_state);
        if let Some(err) = graph.error_message {
            println!("  error: {err}");
        }

        Ok(())
    }

    pub async fn start_chat(&mut self, provider_override: Option<ChatProvider>) -> Result<()> {
        let project = self
            .selected_project
            .clone()
            .ok_or_else(|| anyhow!("Select a project before starting chat"))?;

        let provider = provider_override.unwrap_or(self.chat_config.default_provider);

        let mut session =
            super::chat::ChatSession::new(self.db.clone(), project.id, provider, &self.chat_config)
                .await
                .context("failed to start chat session")?;

        session.interactive_loop().await
    }
}
