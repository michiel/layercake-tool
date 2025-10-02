use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Step 1: Add new columns with defaults
        manager
            .alter_table(
                Table::alter()
                    .table(DataSources::Table)
                    .add_column(
                        ColumnDef::new(DataSources::FileFormat)
                            .string()
                            .not_null()
                            .default("csv")
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(DataSources::Table)
                    .add_column(
                        ColumnDef::new(DataSources::DataType)
                            .string()
                            .not_null()
                            .default("nodes")
                    )
                    .to_owned(),
            )
            .await?;

        // Step 2: Migrate existing data based on source_type
        // csv_nodes -> format: csv, type: nodes
        manager
            .exec_stmt(
                Query::update()
                    .table(DataSources::Table)
                    .values([
                        (DataSources::FileFormat, "csv".into()),
                        (DataSources::DataType, "nodes".into()),
                    ])
                    .and_where(Expr::col(DataSources::SourceType).eq("csv_nodes"))
                    .to_owned(),
            )
            .await?;

        // csv_edges -> format: csv, type: edges
        manager
            .exec_stmt(
                Query::update()
                    .table(DataSources::Table)
                    .values([
                        (DataSources::FileFormat, "csv".into()),
                        (DataSources::DataType, "edges".into()),
                    ])
                    .and_where(Expr::col(DataSources::SourceType).eq("csv_edges"))
                    .to_owned(),
            )
            .await?;

        // csv_layers -> format: csv, type: layers
        manager
            .exec_stmt(
                Query::update()
                    .table(DataSources::Table)
                    .values([
                        (DataSources::FileFormat, "csv".into()),
                        (DataSources::DataType, "layers".into()),
                    ])
                    .and_where(Expr::col(DataSources::SourceType).eq("csv_layers"))
                    .to_owned(),
            )
            .await?;

        // json_graph -> format: json, type: graph
        manager
            .exec_stmt(
                Query::update()
                    .table(DataSources::Table)
                    .values([
                        (DataSources::FileFormat, "json".into()),
                        (DataSources::DataType, "graph".into()),
                    ])
                    .and_where(Expr::col(DataSources::SourceType).eq("json_graph"))
                    .to_owned(),
            )
            .await?;

        // Step 3: Drop old source_type column
        // Note: Keeping this commented for safety - can be enabled after verification
        // manager
        //     .alter_table(
        //         Table::alter()
        //             .table(DataSources::Table)
        //             .drop_column(DataSources::SourceType)
        //             .to_owned(),
        //     )
        //     .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Restore source_type column if it was dropped
        // manager
        //     .alter_table(
        //         Table::alter()
        //             .table(DataSources::Table)
        //             .add_column(ColumnDef::new(DataSources::SourceType).string().not_null())
        //             .to_owned(),
        //     )
        //     .await?;

        // Migrate data back to source_type format
        manager
            .exec_stmt(
                Query::update()
                    .table(DataSources::Table)
                    .value(DataSources::SourceType, Expr::value("csv_nodes"))
                    .and_where(
                        Expr::col(DataSources::FileFormat).eq("csv")
                            .and(Expr::col(DataSources::DataType).eq("nodes"))
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .exec_stmt(
                Query::update()
                    .table(DataSources::Table)
                    .value(DataSources::SourceType, Expr::value("csv_edges"))
                    .and_where(
                        Expr::col(DataSources::FileFormat).eq("csv")
                            .and(Expr::col(DataSources::DataType).eq("edges"))
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .exec_stmt(
                Query::update()
                    .table(DataSources::Table)
                    .value(DataSources::SourceType, Expr::value("csv_layers"))
                    .and_where(
                        Expr::col(DataSources::FileFormat).eq("csv")
                            .and(Expr::col(DataSources::DataType).eq("layers"))
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .exec_stmt(
                Query::update()
                    .table(DataSources::Table)
                    .value(DataSources::SourceType, Expr::value("json_graph"))
                    .and_where(
                        Expr::col(DataSources::FileFormat).eq("json")
                            .and(Expr::col(DataSources::DataType).eq("graph"))
                    )
                    .to_owned(),
            )
            .await?;

        // Drop new columns
        manager
            .alter_table(
                Table::alter()
                    .table(DataSources::Table)
                    .drop_column(DataSources::DataType)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(DataSources::Table)
                    .drop_column(DataSources::FileFormat)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum DataSources {
    Table,
    SourceType,
    FileFormat,
    DataType,
}
