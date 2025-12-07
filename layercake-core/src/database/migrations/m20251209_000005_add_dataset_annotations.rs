use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(DataSets::Table)
                    .add_column(ColumnDef::new(DataSets::Annotations).json_binary().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(DataSets::Table)
                    .drop_column(DataSets::Annotations)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum DataSets {
    Table,
    Annotations,
}
