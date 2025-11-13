use sea_orm::Statement;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Update node_type from DataSourceNode to DataSetNode (if any exist)
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "UPDATE plan_dag_nodes SET node_type = 'DataSetNode' WHERE node_type = 'DataSourceNode'".to_string(),
        ))
        .await?;

        // Update config_json: replace "dataSourceId" with "dataSetId"
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "UPDATE plan_dag_nodes SET config_json = REPLACE(config_json, '\"dataSourceId\"', '\"dataSetId\"') WHERE config_json LIKE '%\"dataSourceId\"%'".to_string(),
        ))
        .await?;

        // Update metadata_json: replace "dataSourceId" with "dataSetId"
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "UPDATE plan_dag_nodes SET metadata_json = REPLACE(metadata_json, '\"dataSourceId\"', '\"dataSetId\"') WHERE metadata_json LIKE '%\"dataSourceId\"%'".to_string(),
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Revert node_type from DataSetNode to DataSourceNode
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "UPDATE plan_dag_nodes SET node_type = 'DataSourceNode' WHERE node_type = 'DataSetNode'".to_string(),
        ))
        .await?;

        // Revert config_json: replace "dataSetId" with "dataSourceId"
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "UPDATE plan_dag_nodes SET config_json = REPLACE(config_json, '\"dataSetId\"', '\"dataSourceId\"') WHERE config_json LIKE '%\"dataSetId\"%'".to_string(),
        ))
        .await?;

        // Revert metadata_json: replace "dataSetId" with "dataSourceId"
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "UPDATE plan_dag_nodes SET metadata_json = REPLACE(metadata_json, '\"dataSetId\"', '\"dataSourceId\"') WHERE metadata_json LIKE '%\"dataSetId\"%'".to_string(),
        ))
        .await?;

        Ok(())
    }
}
