use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ProjectLayers::Table)
                    .add_column(ColumnDef::new(ProjectLayers::Alias).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GraphLayers::Table)
                    .add_column(ColumnDef::new(GraphLayers::Alias).string().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ProjectLayers::Table)
                    .drop_column(ProjectLayers::Alias)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GraphLayers::Table)
                    .drop_column(GraphLayers::Alias)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ProjectLayers {
    Table,
    Alias,
}

#[derive(DeriveIden)]
enum GraphLayers {
    Table,
    Alias,
}
