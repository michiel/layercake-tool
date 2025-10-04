use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create datasources table (for DAG DataSourceNode entities)
        manager
            .create_table(
                Table::create()
                    .table(Datasources::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Datasources::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Datasources::ProjectId).integer().not_null())
                    .col(ColumnDef::new(Datasources::NodeId).string().not_null())
                    .col(ColumnDef::new(Datasources::Name).string().not_null())
                    .col(ColumnDef::new(Datasources::FilePath).string().not_null())
                    .col(ColumnDef::new(Datasources::FileType).string().not_null()) // 'nodes' or 'edges'
                    .col(ColumnDef::new(Datasources::ImportDate).timestamp())
                    .col(ColumnDef::new(Datasources::RowCount).integer())
                    .col(ColumnDef::new(Datasources::ColumnInfo).json_binary()) // Schema: [{name, type, nullable}, ...]
                    .col(
                        ColumnDef::new(Datasources::ExecutionState)
                            .string()
                            .not_null()
                            .default("not_started"),
                    )
                    .col(ColumnDef::new(Datasources::ErrorMessage).text())
                    .col(ColumnDef::new(Datasources::Metadata).json_binary())
                    .col(ColumnDef::new(Datasources::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Datasources::UpdatedAt).timestamp().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_datasources_project_id")
                            .from(Datasources::Table, Datasources::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_datasources_node_id")
                            .from(Datasources::Table, Datasources::NodeId)
                            .to(PlanDagNodes::Table, PlanDagNodes::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique constraint on (project_id, node_id)
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_datasources_project_node")
                    .table(Datasources::Table)
                    .col(Datasources::ProjectId)
                    .col(Datasources::NodeId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create datasource_rows table (normalized storage of CSV data)
        manager
            .create_table(
                Table::create()
                    .table(DatasourceRows::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DatasourceRows::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(DatasourceRows::DatasourceId).integer().not_null())
                    .col(ColumnDef::new(DatasourceRows::RowNumber).integer().not_null())
                    .col(ColumnDef::new(DatasourceRows::Data).json_binary().not_null())
                    .col(ColumnDef::new(DatasourceRows::CreatedAt).timestamp().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_datasource_rows_datasource_id")
                            .from(DatasourceRows::Table, DatasourceRows::DatasourceId)
                            .to(Datasources::Table, Datasources::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique constraint on (datasource_id, row_number)
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_datasource_rows_datasource_row")
                    .table(DatasourceRows::Table)
                    .col(DatasourceRows::DatasourceId)
                    .col(DatasourceRows::RowNumber)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create index on datasource_id for faster lookups
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_datasource_rows_datasource")
                    .table(DatasourceRows::Table)
                    .col(DatasourceRows::DatasourceId)
                    .to_owned(),
            )
            .await?;

        // Create graphs table (for DAG GraphNode entities)
        manager
            .create_table(
                Table::create()
                    .table(Graphs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Graphs::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Graphs::ProjectId).integer().not_null())
                    .col(ColumnDef::new(Graphs::NodeId).string().not_null())
                    .col(ColumnDef::new(Graphs::Name).string().not_null())
                    .col(
                        ColumnDef::new(Graphs::ExecutionState)
                            .string()
                            .not_null()
                            .default("not_started"),
                    )
                    .col(ColumnDef::new(Graphs::ComputedDate).timestamp())
                    .col(ColumnDef::new(Graphs::SourceHash).string()) // Hash of upstream data
                    .col(ColumnDef::new(Graphs::NodeCount).integer().not_null().default(0))
                    .col(ColumnDef::new(Graphs::EdgeCount).integer().not_null().default(0))
                    .col(ColumnDef::new(Graphs::ErrorMessage).text())
                    .col(ColumnDef::new(Graphs::Metadata).json_binary())
                    .col(ColumnDef::new(Graphs::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Graphs::UpdatedAt).timestamp().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_graphs_project_id")
                            .from(Graphs::Table, Graphs::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_graphs_node_id")
                            .from(Graphs::Table, Graphs::NodeId)
                            .to(PlanDagNodes::Table, PlanDagNodes::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique constraint on (project_id, node_id)
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_graphs_project_node")
                    .table(Graphs::Table)
                    .col(Graphs::ProjectId)
                    .col(Graphs::NodeId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create graph_nodes table
        manager
            .create_table(
                Table::create()
                    .table(GraphNodes::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GraphNodes::Id).string().not_null())
                    .col(ColumnDef::new(GraphNodes::GraphId).integer().not_null())
                    .col(ColumnDef::new(GraphNodes::Label).string())
                    .col(ColumnDef::new(GraphNodes::Layer).string())
                    .col(ColumnDef::new(GraphNodes::Weight).double())
                    .col(ColumnDef::new(GraphNodes::IsPartition).boolean().not_null().default(false))
                    .col(ColumnDef::new(GraphNodes::Attrs).json_binary())
                    .col(ColumnDef::new(GraphNodes::CreatedAt).timestamp().not_null())
                    .primary_key(
                        Index::create()
                            .name("pk_graph_nodes")
                            .col(GraphNodes::GraphId)
                            .col(GraphNodes::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_graph_nodes_graph_id")
                            .from(GraphNodes::Table, GraphNodes::GraphId)
                            .to(Graphs::Table, Graphs::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on graph_id for faster lookups
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_graph_nodes_graph")
                    .table(GraphNodes::Table)
                    .col(GraphNodes::GraphId)
                    .to_owned(),
            )
            .await?;

        // Create graph_edges table
        manager
            .create_table(
                Table::create()
                    .table(GraphEdges::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GraphEdges::Id).string().not_null())
                    .col(ColumnDef::new(GraphEdges::GraphId).integer().not_null())
                    .col(ColumnDef::new(GraphEdges::Source).string().not_null())
                    .col(ColumnDef::new(GraphEdges::Target).string().not_null())
                    .col(ColumnDef::new(GraphEdges::Label).string())
                    .col(ColumnDef::new(GraphEdges::Layer).string())
                    .col(ColumnDef::new(GraphEdges::Weight).double())
                    .col(ColumnDef::new(GraphEdges::Attrs).json_binary())
                    .col(ColumnDef::new(GraphEdges::CreatedAt).timestamp().not_null())
                    .primary_key(
                        Index::create()
                            .name("pk_graph_edges")
                            .col(GraphEdges::GraphId)
                            .col(GraphEdges::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_graph_edges_graph_id")
                            .from(GraphEdges::Table, GraphEdges::GraphId)
                            .to(Graphs::Table, Graphs::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on graph_id for faster lookups
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_graph_edges_graph")
                    .table(GraphEdges::Table)
                    .col(GraphEdges::GraphId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order (children first)
        manager
            .drop_table(Table::drop().table(GraphEdges::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(GraphNodes::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Graphs::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(DatasourceRows::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Datasources::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum PlanDagNodes {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Datasources {
    Table,
    Id,
    ProjectId,
    NodeId,
    Name,
    FilePath,
    FileType,
    ImportDate,
    RowCount,
    ColumnInfo,
    ExecutionState,
    ErrorMessage,
    Metadata,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum DatasourceRows {
    Table,
    Id,
    DatasourceId,
    RowNumber,
    Data,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Graphs {
    Table,
    Id,
    ProjectId,
    NodeId,
    Name,
    ExecutionState,
    ComputedDate,
    SourceHash,
    NodeCount,
    EdgeCount,
    ErrorMessage,
    Metadata,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum GraphNodes {
    Table,
    Id,
    GraphId,
    Label,
    Layer,
    Weight,
    IsPartition,
    Attrs,
    CreatedAt,
}

#[derive(DeriveIden)]
enum GraphEdges {
    Table,
    Id,
    GraphId,
    Source,
    Target,
    Label,
    Layer,
    Weight,
    Attrs,
    CreatedAt,
}
