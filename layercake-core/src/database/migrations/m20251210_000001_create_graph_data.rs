use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GraphData::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GraphData::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(GraphData::ProjectId).integer().not_null())
                    .col(ColumnDef::new(GraphData::Name).string().not_null())
                    // Source and lifecycle
                    .col(ColumnDef::new(GraphData::SourceType).string().not_null())
                    .col(ColumnDef::new(GraphData::DagNodeId).string().null())
                    // Dataset-specific fields
                    .col(ColumnDef::new(GraphData::FileFormat).string().null())
                    .col(ColumnDef::new(GraphData::Origin).string().null())
                    .col(ColumnDef::new(GraphData::Filename).string().null())
                    .col(ColumnDef::new(GraphData::Blob).binary().null())
                    .col(ColumnDef::new(GraphData::FileSize).integer().null())
                    .col(ColumnDef::new(GraphData::ProcessedAt).timestamp().null())
                    // Computed graph-specific fields
                    .col(ColumnDef::new(GraphData::SourceHash).string().null())
                    .col(ColumnDef::new(GraphData::ComputedDate).timestamp().null())
                    // Edit tracking (for computed graphs only)
                    .col(
                        ColumnDef::new(GraphData::LastEditSequence)
                            .integer()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(GraphData::HasPendingEdits)
                            .boolean()
                            .default(false),
                    )
                    .col(ColumnDef::new(GraphData::LastReplayAt).timestamp().null())
                    // Common metadata
                    .col(
                        ColumnDef::new(GraphData::NodeCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(GraphData::EdgeCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(GraphData::ErrorMessage).string().null())
                    .col(ColumnDef::new(GraphData::Metadata).json_binary().null())
                    .col(ColumnDef::new(GraphData::Annotations).json_binary().null())
                    .col(ColumnDef::new(GraphData::Status).string().not_null())
                    // Timestamps
                    .col(ColumnDef::new(GraphData::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(GraphData::UpdatedAt).timestamp().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_graph_data_project")
                            .from(GraphData::Table, GraphData::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for performance
        manager
            .create_index(
                Index::create()
                    .name("idx_graph_data_project")
                    .table(GraphData::Table)
                    .col(GraphData::ProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_graph_data_dag_node")
                    .table(GraphData::Table)
                    .col(GraphData::DagNodeId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_graph_data_source_type")
                    .table(GraphData::Table)
                    .col(GraphData::ProjectId)
                    .col(GraphData::SourceType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_graph_data_status")
                    .table(GraphData::Table)
                    .col(GraphData::Status)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GraphData::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum GraphData {
    Table,
    Id,
    ProjectId,
    Name,
    SourceType,
    DagNodeId,
    FileFormat,
    Origin,
    Filename,
    Blob,
    FileSize,
    ProcessedAt,
    SourceHash,
    ComputedDate,
    LastEditSequence,
    HasPendingEdits,
    LastReplayAt,
    NodeCount,
    EdgeCount,
    ErrorMessage,
    Metadata,
    Annotations,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Projects {
    Table,
    Id,
}
