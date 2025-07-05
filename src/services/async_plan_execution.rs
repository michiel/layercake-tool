use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use anyhow::{Result, Context};
use chrono::Utc;
use sea_orm::*;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{info, error, warn, debug};
use uuid::Uuid;
use async_trait::async_trait;

use crate::database::entities::{
    plans, plan_executions, execution_logs, execution_outputs,
    plan_executions::ExecutionStatus,
    execution_logs::LogLevel,
    execution_outputs::OutputFileType,
};

/// Progress reporter integration for real-time updates
#[async_trait]
pub trait ProgressReporter: Send + Sync {
    async fn report_progress(&self, execution_id: &str, progress: i32, message: &str);
    async fn report_log(&self, execution_id: &str, level: LogLevel, message: &str, details: Option<&str>);
    async fn report_error(&self, execution_id: &str, error: &str);
    async fn report_completion(&self, execution_id: &str, success: bool);
}

/// Default progress reporter that logs to console
pub struct DefaultProgressReporter;

#[async_trait]
impl ProgressReporter for DefaultProgressReporter {
    async fn report_progress(&self, execution_id: &str, progress: i32, message: &str) {
        info!("[{}] Progress: {}% - {}", execution_id, progress, message);
    }

    async fn report_log(&self, execution_id: &str, level: LogLevel, message: &str, details: Option<&str>) {
        match level {
            LogLevel::Info => info!("[{}] {}", execution_id, message),
            LogLevel::Warning => warn!("[{}] {}", execution_id, message),
            LogLevel::Success => info!("[{}] ✓ {}", execution_id, message),
            LogLevel::Error => error!("[{}] ✗ {}", execution_id, message),
            LogLevel::Debug => debug!("[{}] {}", execution_id, message),
        }
        if let Some(details) = details {
            debug!("[{}] Details: {}", execution_id, details);
        }
    }

    async fn report_error(&self, execution_id: &str, error: &str) {
        error!("[{}] Execution failed: {}", execution_id, error);
    }

    async fn report_completion(&self, execution_id: &str, success: bool) {
        if success {
            info!("[{}] Execution completed successfully", execution_id);
        } else {
            error!("[{}] Execution failed", execution_id);
        }
    }
}

/// Async plan execution service
#[derive(Clone)]
pub struct AsyncPlanExecutionService {
    db: DatabaseConnection,
    progress_reporter: Arc<dyn ProgressReporter>,
    active_executions: Arc<RwLock<HashMap<String, ExecutionHandle>>>,
}

struct ExecutionHandle {
    plan_id: i32,
    status: ExecutionStatus,
    task_handle: JoinHandle<Result<()>>,
}

impl AsyncPlanExecutionService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self::with_progress_reporter(db, Arc::new(DefaultProgressReporter))
    }

    pub fn with_progress_reporter(
        db: DatabaseConnection,
        progress_reporter: Arc<dyn ProgressReporter>,
    ) -> Self {
        Self {
            db,
            progress_reporter,
            active_executions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start async execution of a plan
    pub async fn execute_plan_async(&self, plan_id: i32) -> Result<String> {
        // Check if plan exists and is not already running
        let plan = plans::Entity::find_by_id(plan_id)
            .one(&self.db)
            .await?
            .context("Plan not found")?;

        // Check if there's already a running execution for this plan
        let running_execution = plan_executions::Entity::find()
            .filter(plan_executions::Column::PlanId.eq(plan_id))
            .filter(plan_executions::Column::Status.eq("running"))
            .one(&self.db)
            .await?;

        if running_execution.is_some() {
            return Err(anyhow::anyhow!("Plan is already being executed"));
        }

        // Generate unique execution ID
        let execution_id = Uuid::new_v4().to_string();

        // Create execution record
        let now = Utc::now();
        let execution = plan_executions::ActiveModel {
            plan_id: Set(plan_id),
            execution_id: Set(execution_id.clone()),
            status: Set(ExecutionStatus::Queued.into()),
            progress: Set(Some(0)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        plan_executions::Entity::insert(execution)
            .exec(&self.db)
            .await?;

        // Log execution start
        self.log_execution(&execution_id, LogLevel::Info, "Execution queued", None)
            .await?;

        // Spawn async execution task
        let task_handle = self.spawn_execution_task(execution_id.clone(), plan).await?;

        // Store execution handle
        let handle = ExecutionHandle {
            plan_id,
            status: ExecutionStatus::Queued,
            task_handle,
        };

        self.active_executions
            .write()
            .await
            .insert(execution_id.clone(), handle);

        Ok(execution_id)
    }

    /// Get execution status
    pub async fn get_execution_status(&self, execution_id: &str) -> Result<Option<plan_executions::Model>> {
        plan_executions::Entity::find()
            .filter(plan_executions::Column::ExecutionId.eq(execution_id))
            .one(&self.db)
            .await
            .map_err(Into::into)
    }

    /// Get execution logs
    pub async fn get_execution_logs(&self, execution_id: &str) -> Result<Vec<execution_logs::Model>> {
        execution_logs::Entity::find()
            .filter(execution_logs::Column::ExecutionId.eq(execution_id))
            .order_by_asc(execution_logs::Column::Timestamp)
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// Get execution outputs
    pub async fn get_execution_outputs(&self, execution_id: &str) -> Result<Vec<execution_outputs::Model>> {
        execution_outputs::Entity::find()
            .filter(execution_outputs::Column::ExecutionId.eq(execution_id))
            .order_by_asc(execution_outputs::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// Cancel execution
    pub async fn cancel_execution(&self, execution_id: &str) -> Result<bool> {
        let mut active_executions = self.active_executions.write().await;
        
        if let Some(handle) = active_executions.remove(execution_id) {
            handle.task_handle.abort();
            
            // Update status in database
            self.update_execution_status(
                execution_id,
                ExecutionStatus::Cancelled,
                None,
                Some("Execution cancelled by user"),
            )
            .await?;

            self.log_execution(execution_id, LogLevel::Warning, "Execution cancelled", None)
                .await?;

            self.progress_reporter.report_completion(execution_id, false).await;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// List all executions for a plan
    pub async fn list_plan_executions(&self, plan_id: i32) -> Result<Vec<plan_executions::Model>> {
        plan_executions::Entity::find()
            .filter(plan_executions::Column::PlanId.eq(plan_id))
            .order_by_desc(plan_executions::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// Spawn the actual execution task
    async fn spawn_execution_task(
        &self,
        execution_id: String,
        plan: plans::Model,
    ) -> Result<JoinHandle<Result<()>>> {
        let db = self.db.clone();
        let progress_reporter = self.progress_reporter.clone();
        let active_executions = self.active_executions.clone();

        let task = tokio::spawn(async move {
            let execution_service = AsyncPlanExecutionService {
                db: db.clone(),
                progress_reporter: progress_reporter.clone(),
                active_executions: active_executions.clone(),
            };

            execution_service
                .execute_plan_internal(execution_id.clone(), plan)
                .await
        });

        Ok(task)
    }

    /// Internal execution logic
    async fn execute_plan_internal(&self, execution_id: String, plan: plans::Model) -> Result<()> {
        // Update status to running
        self.update_execution_status(&execution_id, ExecutionStatus::Running, None, None)
            .await?;

        self.log_execution(&execution_id, LogLevel::Info, "Starting plan execution", None)
            .await?;

        self.progress_reporter
            .report_progress(&execution_id, 0, "Initializing execution")
            .await;

        let result = async {
            // Parse plan content
            self.progress_reporter
                .report_progress(&execution_id, 10, "Parsing plan content")
                .await;

            let plan_json = plan.get_plan_json()
                .context("Failed to parse plan content")?;

            self.log_execution(
                &execution_id,
                LogLevel::Info,
                "Plan content parsed successfully",
                None,
            )
            .await?;

            // Validate plan
            self.progress_reporter
                .report_progress(&execution_id, 20, "Validating plan")
                .await;

            plan.validate_plan_schema()
                .map_err(|e| anyhow::anyhow!("Plan validation failed: {}", e))?;

            self.log_execution(&execution_id, LogLevel::Success, "Plan validation passed", None)
                .await?;

            // Execute plan using existing plan execution logic
            self.progress_reporter
                .report_progress(&execution_id, 30, "Executing plan")
                .await;

            // For now, we'll use a simplified execution that creates output files
            // In a real implementation, this would integrate with the existing plan_execution module
            self.execute_plan_steps(&execution_id, &plan_json).await?;

            self.progress_reporter
                .report_progress(&execution_id, 100, "Execution completed")
                .await;

            Ok::<(), anyhow::Error>(())
        }
        .await;

        // Handle execution result
        match result {
            Ok(_) => {
                self.update_execution_status(&execution_id, ExecutionStatus::Completed, None, None)
                    .await?;

                self.log_execution(
                    &execution_id,
                    LogLevel::Success,
                    "Plan execution completed successfully",
                    None,
                )
                .await?;

                self.progress_reporter.report_completion(&execution_id, true).await;
            }
            Err(e) => {
                let error_msg = e.to_string();
                
                self.update_execution_status(
                    &execution_id,
                    ExecutionStatus::Failed,
                    None,
                    Some(&error_msg),
                )
                .await?;

                self.log_execution(
                    &execution_id,
                    LogLevel::Error,
                    "Plan execution failed",
                    Some(&error_msg),
                )
                .await?;

                self.progress_reporter.report_error(&execution_id, &error_msg).await;
                self.progress_reporter.report_completion(&execution_id, false).await;

                return Err(e);
            }
        }

        // Remove from active executions
        self.active_executions.write().await.remove(&execution_id);

        Ok(())
    }

    /// Execute plan steps (simplified implementation)
    async fn execute_plan_steps(&self, execution_id: &str, plan_json: &serde_json::Value) -> Result<()> {
        // This is a simplified implementation
        // In a real scenario, this would integrate with the existing plan_execution module

        self.progress_reporter
            .report_progress(execution_id, 40, "Processing imports")
            .await;

        // Simulate import processing
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        self.log_execution(execution_id, LogLevel::Info, "Import processing completed", None)
            .await?;

        self.progress_reporter
            .report_progress(execution_id, 60, "Generating exports")
            .await;

        // Simulate export generation
        if let Some(export_profiles) = plan_json.get("export").and_then(|e| e.get("profiles")) {
            if let Some(profiles_array) = export_profiles.as_array() {
                for (i, profile) in profiles_array.iter().enumerate() {
                    if let Some(filename) = profile.get("filename").and_then(|f| f.as_str()) {
                        let progress = 60 + (30 * (i + 1) / profiles_array.len()) as i32;
                        
                        self.progress_reporter
                            .report_progress(execution_id, progress, &format!("Generating {}", filename))
                            .await;

                        // Record output file
                        self.record_output_file(execution_id, filename).await?;

                        self.log_execution(
                            execution_id,
                            LogLevel::Success,
                            &format!("Generated output: {}", filename),
                            None,
                        )
                        .await?;

                        // Simulate processing time
                        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                    }
                }
            }
        }

        Ok(())
    }

    /// Record output file in database
    async fn record_output_file(&self, execution_id: &str, filename: &str) -> Result<()> {
        let file_type = OutputFileType::from(
            PathBuf::from(filename)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_string()
        );

        let output = execution_outputs::ActiveModel {
            execution_id: Set(execution_id.to_string()),
            file_name: Set(filename.to_string()),
            file_type: Set(file_type.into()),
            file_path: Set(Some(format!("outputs/{}", filename))),
            file_size: Set(Some(1024)), // Simulated file size
            created_at: Set(Utc::now()),
            ..Default::default()
        };

        execution_outputs::Entity::insert(output)
            .exec(&self.db)
            .await?;

        Ok(())
    }

    /// Update execution status in database
    async fn update_execution_status(
        &self,
        execution_id: &str,
        status: ExecutionStatus,
        progress: Option<i32>,
        error: Option<&str>,
    ) -> Result<()> {
        let now = Utc::now();

        let mut update = plan_executions::ActiveModel {
            status: Set(status.clone().into()),
            updated_at: Set(now),
            ..Default::default()
        };

        if let Some(progress) = progress {
            update.progress = Set(Some(progress));
        }

        if let Some(error) = error {
            update.error = Set(Some(error.to_string()));
        }

        match status {
            ExecutionStatus::Running => {
                update.started_at = Set(Some(now));
            }
            ExecutionStatus::Completed | ExecutionStatus::Failed | ExecutionStatus::Cancelled => {
                update.completed_at = Set(Some(now));
            }
            _ => {}
        }

        plan_executions::Entity::update_many()
            .set(update)
            .filter(plan_executions::Column::ExecutionId.eq(execution_id))
            .exec(&self.db)
            .await?;

        Ok(())
    }

    /// Log execution event
    async fn log_execution(
        &self,
        execution_id: &str,
        level: LogLevel,
        message: &str,
        details: Option<&str>,
    ) -> Result<()> {
        let log = execution_logs::ActiveModel {
            execution_id: Set(execution_id.to_string()),
            level: Set(level.clone().into()),
            message: Set(message.to_string()),
            details: Set(details.map(|d| d.to_string())),
            timestamp: Set(Utc::now()),
            ..Default::default()
        };

        execution_logs::Entity::insert(log)
            .exec(&self.db)
            .await?;

        self.progress_reporter
            .report_log(execution_id, level, message, details)
            .await;

        Ok(())
    }
}