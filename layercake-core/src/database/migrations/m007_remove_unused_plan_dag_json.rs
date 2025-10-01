use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove unused plan_dag_json column from plans table
        // This column was intended for JSON storage but superseded by plan_dag_nodes/plan_dag_edges tables
        manager
            .alter_table(
                Table::alter()
                    .table(Plans::Table)
                    .drop_column(Plans::PlanDagJson)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Restore plan_dag_json column if migration is rolled back
        manager
            .alter_table(
                Table::alter()
                    .table(Plans::Table)
                    .add_column(ColumnDef::new(Plans::PlanDagJson).text().null())
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Plans {
    Table,
    PlanDagJson,
}
