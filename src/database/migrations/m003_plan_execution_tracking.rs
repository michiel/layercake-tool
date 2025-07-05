use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create plan_executions table
        manager
            .create_table(
                Table::create()
                    .table(PlanExecutions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PlanExecutions::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PlanExecutions::PlanId).integer().not_null())
                    .col(ColumnDef::new(PlanExecutions::ExecutionId).text().not_null())
                    .col(
                        ColumnDef::new(PlanExecutions::Status)
                            .text()
                            .not_null()
                            .default("queued"),
                    )
                    .col(ColumnDef::new(PlanExecutions::Progress).integer().default(0))
                    .col(ColumnDef::new(PlanExecutions::StartedAt).text())
                    .col(ColumnDef::new(PlanExecutions::CompletedAt).text())
                    .col(ColumnDef::new(PlanExecutions::Error).text())
                    .col(ColumnDef::new(PlanExecutions::CreatedAt).text().not_null())
                    .col(ColumnDef::new(PlanExecutions::UpdatedAt).text().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_plan_executions_plan_id")
                            .from(PlanExecutions::Table, PlanExecutions::PlanId)
                            .to(Plans::Table, Plans::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create execution_logs table
        manager
            .create_table(
                Table::create()
                    .table(ExecutionLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ExecutionLogs::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ExecutionLogs::ExecutionId).text().not_null())
                    .col(
                        ColumnDef::new(ExecutionLogs::Level)
                            .text()
                            .not_null()
                            .default("info"),
                    )
                    .col(ColumnDef::new(ExecutionLogs::Message).text().not_null())
                    .col(ColumnDef::new(ExecutionLogs::Details).text())
                    .col(ColumnDef::new(ExecutionLogs::Timestamp).text().not_null())
                    .to_owned(),
            )
            .await?;

        // Create execution_outputs table
        manager
            .create_table(
                Table::create()
                    .table(ExecutionOutputs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ExecutionOutputs::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ExecutionOutputs::ExecutionId).text().not_null())
                    .col(ColumnDef::new(ExecutionOutputs::FileName).text().not_null())
                    .col(ColumnDef::new(ExecutionOutputs::FileType).text().not_null())
                    .col(ColumnDef::new(ExecutionOutputs::FilePath).text())
                    .col(ColumnDef::new(ExecutionOutputs::FileSize).integer())
                    .col(ColumnDef::new(ExecutionOutputs::CreatedAt).text().not_null())
                    .to_owned(),
            )
            .await?;

        // Create indexes for better query performance
        manager
            .create_index(
                Index::create()
                    .name("idx_plan_executions_plan_id")
                    .table(PlanExecutions::Table)
                    .col(PlanExecutions::PlanId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_plan_executions_execution_id")
                    .table(PlanExecutions::Table)
                    .col(PlanExecutions::ExecutionId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_execution_logs_execution_id")
                    .table(ExecutionLogs::Table)
                    .col(ExecutionLogs::ExecutionId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_execution_outputs_execution_id")
                    .table(ExecutionOutputs::Table)
                    .col(ExecutionOutputs::ExecutionId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ExecutionOutputs::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ExecutionLogs::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(PlanExecutions::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum PlanExecutions {
    Table,
    Id,
    PlanId,
    ExecutionId,
    Status,
    Progress,
    StartedAt,
    CompletedAt,
    Error,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum ExecutionLogs {
    Table,
    Id,
    ExecutionId,
    Level,
    Message,
    Details,
    Timestamp,
}

#[derive(Iden)]
enum ExecutionOutputs {
    Table,
    Id,
    ExecutionId,
    FileName,
    FileType,
    FilePath,
    FileSize,
    CreatedAt,
}

#[derive(Iden)]
enum Plans {
    Table,
    Id,
}