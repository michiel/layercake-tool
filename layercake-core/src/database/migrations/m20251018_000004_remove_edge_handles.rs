use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop source_handle column from plan_dag_edges table
        manager
            .alter_table(
                Table::alter()
                    .table(PlanDagEdges::Table)
                    .drop_column(PlanDagEdges::SourceHandle)
                    .to_owned(),
            )
            .await?;

        // Drop target_handle column from plan_dag_edges table (separate statement for SQLite)
        manager
            .alter_table(
                Table::alter()
                    .table(PlanDagEdges::Table)
                    .drop_column(PlanDagEdges::TargetHandle)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Re-add target_handle column if rolling back
        manager
            .alter_table(
                Table::alter()
                    .table(PlanDagEdges::Table)
                    .add_column(ColumnDef::new(PlanDagEdges::TargetHandle).string().null())
                    .to_owned(),
            )
            .await?;

        // Re-add source_handle column if rolling back (separate statement for SQLite)
        manager
            .alter_table(
                Table::alter()
                    .table(PlanDagEdges::Table)
                    .add_column(ColumnDef::new(PlanDagEdges::SourceHandle).string().null())
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
