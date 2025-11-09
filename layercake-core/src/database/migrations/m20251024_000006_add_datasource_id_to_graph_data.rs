use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add dataset_id to graph_nodes
        manager
            .alter_table(
                Table::alter()
                    .table(GraphNodes::Table)
                    .add_column(ColumnDef::new(GraphNodes::DataSetId).integer())
                    .to_owned(),
            )
            .await?;

        // Add dataset_id to graph_edges
        manager
            .alter_table(
                Table::alter()
                    .table(GraphEdges::Table)
                    .add_column(ColumnDef::new(GraphEdges::DataSetId).integer())
                    .to_owned(),
            )
            .await?;

        // Add dataset_id to layers
        manager
            .alter_table(
                Table::alter()
                    .table(Layers::Table)
                    .add_column(ColumnDef::new(Layers::DataSetId).integer())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop dataset_id from graph_nodes
        manager
            .alter_table(
                Table::alter()
                    .table(GraphNodes::Table)
                    .drop_column(GraphNodes::DataSetId)
                    .to_owned(),
            )
            .await?;

        // Drop dataset_id from graph_edges
        manager
            .alter_table(
                Table::alter()
                    .table(GraphEdges::Table)
                    .drop_column(GraphEdges::DataSetId)
                    .to_owned(),
            )
            .await?;

        // Drop dataset_id from layers
        manager
            .alter_table(
                Table::alter()
                    .table(Layers::Table)
                    .drop_column(Layers::DataSetId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum GraphNodes {
    Table,
    DataSetId,
}

#[derive(DeriveIden)]
enum GraphEdges {
    Table,
    DataSetId,
}

#[derive(DeriveIden)]
enum Layers {
    Table,
    DataSetId,
}
