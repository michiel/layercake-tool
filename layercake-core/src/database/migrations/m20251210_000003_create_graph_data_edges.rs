use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GraphDataEdges::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GraphDataEdges::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(GraphDataEdges::GraphDataId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GraphDataEdges::ExternalId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GraphDataEdges::Source)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GraphDataEdges::Target)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(GraphDataEdges::Label).string().null())
                    .col(ColumnDef::new(GraphDataEdges::Layer).string().null())
                    .col(ColumnDef::new(GraphDataEdges::Weight).double().null())
                    .col(ColumnDef::new(GraphDataEdges::Comment).string().null())
                    .col(
                        ColumnDef::new(GraphDataEdges::SourceDatasetId)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(GraphDataEdges::Attributes)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(GraphDataEdges::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_graph_data_edges_graph_data")
                            .from(GraphDataEdges::Table, GraphDataEdges::GraphDataId)
                            .to(GraphData::Table, GraphData::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    // Composite FK for source node
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_graph_data_edges_source")
                            .from(
                                GraphDataEdges::Table,
                                (GraphDataEdges::GraphDataId, GraphDataEdges::Source),
                            )
                            .to(
                                GraphDataNodes::Table,
                                (GraphDataNodes::GraphDataId, GraphDataNodes::ExternalId),
                            )
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    // Composite FK for target node
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_graph_data_edges_target")
                            .from(
                                GraphDataEdges::Table,
                                (GraphDataEdges::GraphDataId, GraphDataEdges::Target),
                            )
                            .to(
                                GraphDataNodes::Table,
                                (GraphDataNodes::GraphDataId, GraphDataNodes::ExternalId),
                            )
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique constraint on (graph_data_id, external_id)
        manager
            .create_index(
                Index::create()
                    .name("idx_edges_graph_external_unique")
                    .table(GraphDataEdges::Table)
                    .col(GraphDataEdges::GraphDataId)
                    .col(GraphDataEdges::ExternalId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create indexes for performance
        manager
            .create_index(
                Index::create()
                    .name("idx_edges_graph")
                    .table(GraphDataEdges::Table)
                    .col(GraphDataEdges::GraphDataId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_edges_source")
                    .table(GraphDataEdges::Table)
                    .col(GraphDataEdges::GraphDataId)
                    .col(GraphDataEdges::Source)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_edges_target")
                    .table(GraphDataEdges::Table)
                    .col(GraphDataEdges::GraphDataId)
                    .col(GraphDataEdges::Target)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_edges_source_target")
                    .table(GraphDataEdges::Table)
                    .col(GraphDataEdges::Source)
                    .col(GraphDataEdges::Target)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_edges_layer")
                    .table(GraphDataEdges::Table)
                    .col(GraphDataEdges::Layer)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GraphDataEdges::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum GraphDataEdges {
    Table,
    Id,
    GraphDataId,
    ExternalId,
    Source,
    Target,
    Label,
    Layer,
    Weight,
    Comment,
    SourceDatasetId,
    Attributes,
    CreatedAt,
}

#[derive(Iden)]
enum GraphData {
    Table,
    Id,
}

#[derive(Iden)]
enum GraphDataNodes {
    Table,
    GraphDataId,
    ExternalId,
}
