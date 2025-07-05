use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create transformation_pipelines table
        manager
            .create_table(
                Table::create()
                    .table(TransformationPipelines::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TransformationPipelines::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TransformationPipelines::Name)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransformationPipelines::Description)
                            .text(),
                    )
                    .col(
                        ColumnDef::new(TransformationPipelines::PipelineData)
                            .json()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransformationPipelines::Enabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(TransformationPipelines::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(TransformationPipelines::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create transformation_rules table
        manager
            .create_table(
                Table::create()
                    .table(TransformationRules::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TransformationRules::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TransformationRules::PipelineId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransformationRules::Name)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransformationRules::Description)
                            .text(),
                    )
                    .col(
                        ColumnDef::new(TransformationRules::RuleData)
                            .json()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransformationRules::Enabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(TransformationRules::OrderIndex)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(TransformationRules::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(TransformationRules::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_transformation_rules_pipeline_id")
                            .from(TransformationRules::Table, TransformationRules::PipelineId)
                            .to(TransformationPipelines::Table, TransformationPipelines::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create transformation_executions table for tracking execution history
        manager
            .create_table(
                Table::create()
                    .table(TransformationExecutions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TransformationExecutions::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TransformationExecutions::PipelineId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransformationExecutions::GraphId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransformationExecutions::Success)
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransformationExecutions::ExecutionData)
                            .json()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransformationExecutions::ExecutionTimeMs)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransformationExecutions::Error)
                            .text(),
                    )
                    .col(
                        ColumnDef::new(TransformationExecutions::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_transformation_executions_pipeline_id")
                            .from(TransformationExecutions::Table, TransformationExecutions::PipelineId)
                            .to(TransformationPipelines::Table, TransformationPipelines::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_transformation_executions_graph_id")
                            .from(TransformationExecutions::Table, TransformationExecutions::GraphId)
                            .to(Alias::new("graphs"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for better performance
        manager
            .create_index(
                Index::create()
                    .name("idx_transformation_rules_pipeline_id")
                    .table(TransformationRules::Table)
                    .col(TransformationRules::PipelineId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_transformation_rules_enabled")
                    .table(TransformationRules::Table)
                    .col(TransformationRules::Enabled)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_transformation_executions_pipeline_id")
                    .table(TransformationExecutions::Table)
                    .col(TransformationExecutions::PipelineId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_transformation_executions_graph_id")
                    .table(TransformationExecutions::Table)
                    .col(TransformationExecutions::GraphId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order of creation
        manager
            .drop_table(Table::drop().table(TransformationExecutions::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(TransformationRules::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(TransformationPipelines::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum TransformationPipelines {
    Table,
    Id,
    Name,
    Description,
    PipelineData,
    Enabled,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum TransformationRules {
    Table,
    Id,
    PipelineId,
    Name,
    Description,
    RuleData,
    Enabled,
    OrderIndex,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum TransformationExecutions {
    Table,
    Id,
    PipelineId,
    GraphId,
    Success,
    ExecutionData,
    ExecutionTimeMs,
    Error,
    CreatedAt,
}