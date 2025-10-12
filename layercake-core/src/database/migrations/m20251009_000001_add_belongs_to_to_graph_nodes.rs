use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(GraphNodes::Table)
                    .add_column(ColumnDef::new(GraphNodes::BelongsTo).string())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(GraphNodes::Table)
                    .drop_column(GraphNodes::BelongsTo)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum GraphNodes {
    Table,
    BelongsTo,
}
