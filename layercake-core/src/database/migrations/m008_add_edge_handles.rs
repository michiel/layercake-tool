use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add source_handle column to plan_dag_edges table
        // SQLite doesn't support multiple ALTER TABLE operations in one statement
        manager
            .alter_table(
                Table::alter()
                    .table(PlanDagEdges::Table)
                    .add_column(ColumnDef::new(PlanDagEdges::SourceHandle).string().null())
                    .to_owned(),
            )
            .await?;

        // Add target_handle column to plan_dag_edges table
        manager
            .alter_table(
                Table::alter()
                    .table(PlanDagEdges::Table)
                    .add_column(ColumnDef::new(PlanDagEdges::TargetHandle).string().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove target_handle column if migration is rolled back
        // SQLite doesn't support multiple ALTER TABLE operations in one statement
        manager
            .alter_table(
                Table::alter()
                    .table(PlanDagEdges::Table)
                    .drop_column(PlanDagEdges::TargetHandle)
                    .to_owned(),
            )
            .await?;

        // Remove source_handle column
        manager
            .alter_table(
                Table::alter()
                    .table(PlanDagEdges::Table)
                    .drop_column(PlanDagEdges::SourceHandle)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum PlanDagEdges {
    Table,
    SourceHandle,
    TargetHandle,
}
