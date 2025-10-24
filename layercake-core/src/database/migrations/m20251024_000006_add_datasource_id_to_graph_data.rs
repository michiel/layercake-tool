use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add datasource_id to graph_nodes
        manager
            .alter_table(
                Table::alter()
                    .table(GraphNodes::Table)
                    .add_column(ColumnDef::new(GraphNodes::DataSourceId).integer())
                    .to_owned(),
            )
            .await?;

        // Add datasource_id to graph_edges
        manager
            .alter_table(
                Table::alter()
                    .table(GraphEdges::Table)
                    .add_column(ColumnDef::new(GraphEdges::DataSourceId).integer())
                    .to_owned(),
            )
            .await?;

        // Add datasource_id to layers
        manager
            .alter_table(
                Table::alter()
                    .table(Layers::Table)
                    .add_column(ColumnDef::new(Layers::DataSourceId).integer())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop datasource_id from graph_nodes
        manager
            .alter_table(
                Table::alter()
                    .table(GraphNodes::Table)
                    .drop_column(GraphNodes::DataSourceId)
                    .to_owned(),
            )
            .await?;

        // Drop datasource_id from graph_edges
        manager
            .alter_table(
                Table::alter()
                    .table(GraphEdges::Table)
                    .drop_column(GraphEdges::DataSourceId)
                    .to_owned(),
            )
            .await?;

        // Drop datasource_id from layers
        manager
            .alter_table(
                Table::alter()
                    .table(Layers::Table)
                    .drop_column(Layers::DataSourceId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum GraphNodes {
    Table,
    DataSourceId,
}

#[derive(DeriveIden)]
enum GraphEdges {
    Table,
    DataSourceId,
}

#[derive(DeriveIden)]
enum Layers {
    Table,
    DataSourceId,
}
