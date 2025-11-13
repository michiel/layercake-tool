use sea_orm::Statement;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

async fn table_exists(manager: &SchemaManager<'_>, name: &str) -> Result<bool, DbErr> {
    let db = manager.get_connection();
    let result = db
        .query_one(Statement::from_string(
            manager.get_database_backend(),
            format!(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='{}'",
                name
            ),
        ))
        .await?;
    Ok(result.is_some())
}

async fn column_exists(
    manager: &SchemaManager<'_>,
    table: &str,
    column: &str,
) -> Result<bool, DbErr> {
    let db = manager.get_connection();
    let rows = db
        .query_all(Statement::from_string(
            manager.get_database_backend(),
            format!("PRAGMA table_info({})", table),
        ))
        .await?;

    for row in rows {
        if let Ok(name) = row.try_get::<String>("", "name") {
            if name == column {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Rename table from data_sources to data_sets (if exists)
        if table_exists(manager, "data_sources").await? {
            manager
                .rename_table(
                    Table::rename()
                        .table(DataSources::Table, DataSets::Table)
                        .to_owned(),
                )
                .await?;
        }

        // Rename table from datasources to dataset_nodes (if exists)
        if table_exists(manager, "datasources").await? {
            manager
                .rename_table(
                    Table::rename()
                        .table(Datasources::Table, DatasetNodes::Table)
                        .to_owned(),
                )
                .await?;
        }

        // Rename table from datasource_rows to dataset_rows (if exists)
        if table_exists(manager, "datasource_rows").await? {
            manager
                .rename_table(
                    Table::rename()
                        .table(DatasourceRows::Table, DatasetRows::Table)
                        .to_owned(),
                )
                .await?;
        }

        // Rename column data_set_id to dataset_id in graph_nodes (if exists)
        if column_exists(manager, "graph_nodes", "data_set_id").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(GraphNodes::Table)
                        .rename_column(GraphNodes::DataSetId, GraphNodes::DatasetId)
                        .to_owned(),
                )
                .await?;
        }

        // Rename column data_set_id to dataset_id in graph_edges (if exists)
        if column_exists(manager, "graph_edges", "data_set_id").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(GraphEdges::Table)
                        .rename_column(GraphEdges::DataSetId, GraphEdges::DatasetId)
                        .to_owned(),
                )
                .await?;
        }

        // Rename column data_set_id to dataset_id in graph_layers (if exists)
        if column_exists(manager, "graph_layers", "data_set_id").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(GraphLayers::Table)
                        .rename_column(GraphLayers::DataSetId, GraphLayers::DatasetId)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Rename column dataset_id back to data_set_id in graph_layers (if exists)
        if column_exists(manager, "graph_layers", "dataset_id").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(GraphLayers::Table)
                        .rename_column(GraphLayers::DatasetId, GraphLayers::DataSetId)
                        .to_owned(),
                )
                .await?;
        }

        // Rename column dataset_id back to data_set_id in graph_edges (if exists)
        if column_exists(manager, "graph_edges", "dataset_id").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(GraphEdges::Table)
                        .rename_column(GraphEdges::DatasetId, GraphEdges::DataSetId)
                        .to_owned(),
                )
                .await?;
        }

        // Rename column dataset_id back to data_set_id in graph_nodes (if exists)
        if column_exists(manager, "graph_nodes", "dataset_id").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(GraphNodes::Table)
                        .rename_column(GraphNodes::DatasetId, GraphNodes::DataSetId)
                        .to_owned(),
                )
                .await?;
        }

        // Rename table back from dataset_rows to datasource_rows (if exists)
        if table_exists(manager, "dataset_rows").await? {
            manager
                .rename_table(
                    Table::rename()
                        .table(DatasetRows::Table, DatasourceRows::Table)
                        .to_owned(),
                )
                .await?;
        }

        // Rename table back from dataset_nodes to datasources (if exists)
        if table_exists(manager, "dataset_nodes").await? {
            manager
                .rename_table(
                    Table::rename()
                        .table(DatasetNodes::Table, Datasources::Table)
                        .to_owned(),
                )
                .await?;
        }

        // Rename table back from data_sets to data_sources (if exists)
        if table_exists(manager, "data_sets").await? {
            manager
                .rename_table(
                    Table::rename()
                        .table(DataSets::Table, DataSources::Table)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
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
    DataSetId,
    DatasetId,
}

#[derive(DeriveIden)]
enum GraphEdges {
    Table,
    DataSetId,
    DatasetId,
}

#[derive(DeriveIden)]
enum GraphLayers {
    Table,
    DataSetId,
    DatasetId,
}
