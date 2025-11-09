use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Rename uploaded data table
        manager
            .rename_table(
                Table::rename()
                    .table(DataSets::Table, DataSets::Table)
                    .to_owned(),
            )
            .await?;

        // Rename DAG dataset nodes table
        manager
            .rename_table(
                Table::rename()
                    .table(DatasetNodes::Table, DatasetNodes::Table)
                    .to_owned(),
            )
            .await?;

        // Rename dataset rows table
        manager
            .rename_table(
                Table::rename()
                    .table(DatasourceRows::Table, DatasetRows::Table)
                    .to_owned(),
            )
            .await?;

        // Rename foreign key columns referencing uploaded data
        manager
            .alter_table(
                Table::alter()
                    .table(GraphNodes::Table)
                    .rename_column(GraphNodes::DataSourceId, GraphNodes::DataSetId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GraphEdges::Table)
                    .rename_column(GraphEdges::DataSourceId, GraphEdges::DataSetId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GraphLayers::Table)
                    .rename_column(GraphLayers::DataSourceId, GraphLayers::DataSetId)
                    .to_owned(),
            )
            .await?;

        // Rename dataset_rows foreign key column
        manager
            .alter_table(
                Table::alter()
                    .table(DatasetRows::Table)
                    .rename_column(DatasetRows::DatasourceId, DatasetRows::DatasetNodeId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Revert dataset_rows column
        manager
            .alter_table(
                Table::alter()
                    .table(DatasetRows::Table)
                    .rename_column(DatasetRows::DatasetNodeId, DatasetRows::DatasourceId)
                    .to_owned(),
            )
            .await?;

        // Revert FK columns
        manager
            .alter_table(
                Table::alter()
                    .table(GraphNodes::Table)
                    .rename_column(GraphNodes::DataSetId, GraphNodes::DataSourceId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GraphEdges::Table)
                    .rename_column(GraphEdges::DataSetId, GraphEdges::DataSourceId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GraphLayers::Table)
                    .rename_column(GraphLayers::DataSetId, GraphLayers::DataSourceId)
                    .to_owned(),
            )
            .await?;

        // Revert table names
        manager
            .rename_table(
                Table::rename()
                    .table(DatasetRows::Table, DatasourceRows::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(DatasetNodes::Table, DatasetNodes::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(DataSets::Table, DataSets::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum DataSets {
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
    DatasourceId,
    DatasetNodeId,
}

#[derive(DeriveIden)]
enum GraphNodes {
    Table,
    DataSourceId,
    DataSetId,
}

#[derive(DeriveIden)]
enum GraphEdges {
    Table,
    DataSourceId,
    DataSetId,
}

#[derive(DeriveIden)]
enum GraphLayers {
    Table,
    DataSourceId,
    DataSetId,
}
