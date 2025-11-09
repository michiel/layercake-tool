use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Rename table from data_sources to data_sets
        manager
            .rename_table(
                Table::rename()
                    .table(DataSources::Table, DataSets::Table)
                    .to_owned(),
            )
            .await?;

        // Rename table from datasources to dataset_nodes
        manager
            .rename_table(
                Table::rename()
                    .table(Datasources::Table, DatasetNodes::Table)
                    .to_owned(),
            )
            .await?;

        // Rename table from datasource_rows to dataset_rows
        manager
            .rename_table(
                Table::rename()
                    .table(DatasourceRows::Table, DatasetRows::Table)
                    .to_owned(),
            )
            .await?;

        // Rename column data_source_id to dataset_id in graph_nodes
        manager
            .alter_table(
                Table::alter()
                    .table(GraphNodes::Table)
                    .rename_column(GraphNodes::DataSourceId, GraphNodes::DatasetId)
                    .to_owned(),
            )
            .await?;

        // Rename column data_source_id to dataset_id in graph_edges
        manager
            .alter_table(
                Table::alter()
                    .table(GraphEdges::Table)
                    .rename_column(GraphEdges::DataSourceId, GraphEdges::DatasetId)
                    .to_owned(),
            )
            .await?;

        // Rename column data_source_id to dataset_id in graph_layers
        manager
            .alter_table(
                Table::alter()
                    .table(GraphLayers::Table)
                    .rename_column(GraphLayers::DataSourceId, GraphLayers::DatasetId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Rename column dataset_id back to data_source_id in graph_layers
        manager
            .alter_table(
                Table::alter()
                    .table(GraphLayers::Table)
                    .rename_column(GraphLayers::DatasetId, GraphLayers::DataSourceId)
                    .to_owned(),
            )
            .await?;

        // Rename column dataset_id back to data_source_id in graph_edges
        manager
            .alter_table(
                Table::alter()
                    .table(GraphEdges::Table)
                    .rename_column(GraphEdges::DatasetId, GraphEdges::DataSourceId)
                    .to_owned(),
            )
            .await?;

        // Rename column dataset_id back to data_source_id in graph_nodes
        manager
            .alter_table(
                Table::alter()
                    .table(GraphNodes::Table)
                    .rename_column(GraphNodes::DatasetId, GraphNodes::DataSourceId)
                    .to_owned(),
            )
            .await?;

        // Rename table back from dataset_rows to datasource_rows
        manager
            .rename_table(
                Table::rename()
                    .table(DatasetRows::Table, DatasourceRows::Table)
                    .to_owned(),
            )
            .await?;

        // Rename table back from dataset_nodes to datasources
        manager
            .rename_table(
                Table::rename()
                    .table(DatasetNodes::Table, Datasources::Table)
                    .to_owned(),
            )
            .await?;

        // Rename table back from data_sets to data_sources
        manager
            .rename_table(
                Table::rename()
                    .table(DataSets::Table, DataSources::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum DataSources {
    Table,
}

#[derive(DeriveIden)]
enum DataSets {
    Table,
}

#[derive(DeriveIden)]
enum Datasources {
    Table,
}

#[derive(DeriveIden)]
enum DatasetNodes {
    Table,
}

#[derive(DeriveIden)]
enum DatasourceRows {
    Table,
}

#[derive(DeriveIden)]
enum DatasetRows {
    Table,
}

#[derive(DeriveIden)]
enum GraphNodes {
    Table,
    DataSourceId,
    DatasetId,
}

#[derive(DeriveIden)]
enum GraphEdges {
    Table,
    DataSourceId,
    DatasetId,
}

#[derive(DeriveIden)]
enum GraphLayers {
    Table,
    DataSourceId,
    DatasetId,
}
