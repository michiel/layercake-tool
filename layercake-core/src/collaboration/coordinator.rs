use tokio::sync::mpsc;
use std::collections::HashMap;
use tracing::{info, debug, warn};

use super::project_actor::ProjectActor;
use super::types::{CoordinatorCommand, ProjectHealthReport};

/// Central coordinator for all collaboration - runs as single-threaded actor
pub struct CollaborationCoordinator {
    projects: HashMap<i32, ProjectActor>,
    command_rx: mpsc::Receiver<CoordinatorCommand>,
}

impl CollaborationCoordinator {
    pub fn spawn() -> CoordinatorHandle {
        let (tx, rx) = mpsc::channel(1000);
        let coordinator = Self {
            projects: HashMap::new(),
            command_rx: rx,
        };

        tokio::spawn(async move {
            coordinator.run().await;
        });

        info!("CollaborationCoordinator spawned");
        CoordinatorHandle { command_tx: tx }
    }

    async fn run(mut self) {
        info!("CollaborationCoordinator event loop started");

        while let Some(cmd) = self.command_rx.recv().await {
            match cmd {
                CoordinatorCommand::JoinProject {
                    project_id,
                    user_id,
                    user_name,
                    avatar_color,
                    sender,
                    response
                } => {
                    debug!("User {} joining project {}", user_id, project_id);

                    let project = self.projects.entry(project_id)
                        .or_insert_with(|| ProjectActor::spawn(project_id));

                    let result = project.join(user_id, user_name, avatar_color, sender).await;
                    let _ = response.send(result);
                }

                CoordinatorCommand::LeaveProject { project_id, user_id, response } => {
                    debug!("User {} leaving project {}", user_id, project_id);

                    if let Some(project) = self.projects.get_mut(&project_id) {
                        let result = project.leave(&user_id).await;

                        // Remove project if empty
                        if project.is_empty().await {
                            debug!("Project {} is empty, removing", project_id);
                            self.projects.remove(&project_id);
                        }

                        let _ = response.send(result);
                    } else {
                        let _ = response.send(Err("Project not found".to_string()));
                    }
                }

                CoordinatorCommand::UpdateCursor {
                    project_id,
                    user_id,
                    document_id,
                    position,
                    selected_node_id
                } => {
                    if let Some(project) = self.projects.get(&project_id) {
                        project.update_cursor(user_id, document_id, position, selected_node_id).await;
                    } else {
                        warn!("Cursor update for non-existent project {}", project_id);
                    }
                }

                CoordinatorCommand::SwitchDocument {
                    project_id,
                    user_id,
                    document_id,
                    document_type,
                } => {
                    if let Some(project) = self.projects.get(&project_id) {
                        project.switch_document(user_id, document_id, document_type).await;
                    } else {
                        warn!("Document switch for non-existent project {}", project_id);
                    }
                }

                CoordinatorCommand::GetProjectHealth { project_id, response } => {
                    let report = if let Some(project) = self.projects.get(&project_id) {
                        project.health_report().await
                    } else {
                        ProjectHealthReport::not_found()
                    };
                    let _ = response.send(report);
                }

                CoordinatorCommand::Shutdown { response } => {
                    info!("CollaborationCoordinator shutting down {} projects", self.projects.len());

                    // Graceful shutdown all projects
                    for (project_id, project) in self.projects.drain() {
                        debug!("Shutting down project {}", project_id);
                        project.shutdown().await;
                    }

                    let _ = response.send(());
                    break;
                }
            }
        }

        info!("CollaborationCoordinator event loop ended");
    }
}

/// Handle to send commands to the CollaborationCoordinator
#[derive(Clone)]
pub struct CoordinatorHandle {
    command_tx: mpsc::Sender<CoordinatorCommand>,
}

impl CoordinatorHandle {
    pub async fn join_project(
        &self,
        project_id: i32,
        user_id: String,
        user_name: String,
        avatar_color: Option<String>,
        sender: mpsc::Sender<crate::server::websocket::types::ServerMessage>,
    ) -> Result<(), String> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.command_tx.send(CoordinatorCommand::JoinProject {
            project_id,
            user_id,
            user_name,
            avatar_color,
            sender,
            response: tx,
        }).await.map_err(|_| "Coordinator unavailable".to_string())?;

        rx.await.map_err(|_| "Response channel closed".to_string())?
    }

    pub async fn leave_project(
        &self,
        project_id: i32,
        user_id: String,
    ) -> Result<(), String> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.command_tx.send(CoordinatorCommand::LeaveProject {
            project_id,
            user_id,
            response: tx,
        }).await.map_err(|_| "Coordinator unavailable".to_string())?;

        rx.await.map_err(|_| "Response channel closed".to_string())?
    }

    pub async fn update_cursor(
        &self,
        project_id: i32,
        user_id: String,
        document_id: String,
        position: crate::server::websocket::types::CursorPosition,
        selected_node_id: Option<String>,
    ) {
        let _ = self.command_tx.send(CoordinatorCommand::UpdateCursor {
            project_id,
            user_id,
            document_id,
            position,
            selected_node_id,
        }).await;
    }

    pub async fn switch_document(
        &self,
        project_id: i32,
        user_id: String,
        document_id: String,
        document_type: crate::server::websocket::types::DocumentType,
    ) {
        let _ = self.command_tx.send(CoordinatorCommand::SwitchDocument {
            project_id,
            user_id,
            document_id,
            document_type,
        }).await;
    }

    #[allow(dead_code)]
    pub async fn get_project_health(&self, project_id: i32) -> ProjectHealthReport {
        let (tx, rx) = tokio::sync::oneshot::channel();
        if self.command_tx.send(CoordinatorCommand::GetProjectHealth {
            project_id,
            response: tx,
        }).await.is_err() {
            return ProjectHealthReport::not_found();
        }

        rx.await.unwrap_or_else(|_| ProjectHealthReport::not_found())
    }

    #[allow(dead_code)]
    pub async fn shutdown(self) {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let _ = self.command_tx.send(CoordinatorCommand::Shutdown { response: tx }).await;
        let _ = rx.await;
    }
}