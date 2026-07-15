use sea_orm_migration::prelude::*;

/// Add `enabled_graph_ids` to stories so a story can source edges from a
/// computed graph (a GraphNode's output in `graph_data`) in addition to raw
/// datasets. JSON array of `graph_data` ids; defaults to `[]`.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Stories::Table)
                    .add_column(
                        ColumnDef::new(Stories::EnabledGraphIds)
                            .text()
                            .not_null()
                            .default("[]"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Stories::Table)
                    .drop_column(Stories::EnabledGraphIds)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Stories {
    Table,
    EnabledGraphIds,
}
