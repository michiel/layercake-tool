use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add source_position column to plan_dag_nodes table
        manager
            .alter_table(
                Table::alter()
                    .table(PlanDagNodes::Table)
                    .add_column(ColumnDef::new(PlanDagNodes::SourcePosition).string().null())
                    .to_owned(),
            )
            .await?;

        // Add target_position column to plan_dag_nodes table (separate statement for SQLite)
        manager
            .alter_table(
                Table::alter()
                    .table(PlanDagNodes::Table)
                    .add_column(ColumnDef::new(PlanDagNodes::TargetPosition).string().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop target_position column if rolling back
        manager
            .alter_table(
                Table::alter()
                    .table(PlanDagNodes::Table)
                    .drop_column(PlanDagNodes::TargetPosition)
                    .to_owned(),
            )
            .await?;

        // Drop source_position column if rolling back (separate statement for SQLite)
        manager
            .alter_table(
                Table::alter()
                    .table(PlanDagNodes::Table)
                    .drop_column(PlanDagNodes::SourcePosition)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum PlanDagNodes {
    Table,
    SourcePosition,
    TargetPosition,
}
