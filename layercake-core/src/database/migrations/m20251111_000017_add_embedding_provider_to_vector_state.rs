use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(VectorIndexState::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(VectorIndexState::EmbeddingProvider)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(VectorIndexState::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(VectorIndexState::EmbeddingModel)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(VectorIndexState::Table)
                    .drop_column(VectorIndexState::EmbeddingModel)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(VectorIndexState::Table)
                    .drop_column(VectorIndexState::EmbeddingProvider)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum VectorIndexState {
    Table,
    EmbeddingProvider,
    EmbeddingModel,
}
