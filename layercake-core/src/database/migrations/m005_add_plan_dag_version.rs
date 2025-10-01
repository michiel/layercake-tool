use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add version column to plans table for optimistic locking
        // This enables delta-based updates with conflict detection
        manager
            .alter_table(
                Table::alter()
                    .table(Plans::Table)
                    .add_column(
                        ColumnDef::new(Plans::Version)
                            .integer()
                            .not_null()
                            .default(1)
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Plans::Table)
                    .drop_column(Plans::Version)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Plans {
    Table,
    Version,
}
