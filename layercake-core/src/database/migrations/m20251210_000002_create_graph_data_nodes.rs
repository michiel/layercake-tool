use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GraphDataNodes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GraphDataNodes::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(GraphDataNodes::GraphDataId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GraphDataNodes::ExternalId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(GraphDataNodes::Label).string().null())
                    .col(ColumnDef::new(GraphDataNodes::Layer).string().null())
                    .col(ColumnDef::new(GraphDataNodes::Weight).double().null())
                    .col(
                        ColumnDef::new(GraphDataNodes::IsPartition)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(GraphDataNodes::BelongsTo).string().null())
                    .col(ColumnDef::new(GraphDataNodes::Comment).string().null())
                    .col(
                        ColumnDef::new(GraphDataNodes::SourceDatasetId)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(GraphDataNodes::Attributes)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(GraphDataNodes::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_graph_data_nodes_graph_data")
                            .from(GraphDataNodes::Table, GraphDataNodes::GraphDataId)
                            .to(GraphData::Table, GraphData::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique constraint on (graph_data_id, external_id)
        manager
            .create_index(
                Index::create()
                    .name("idx_nodes_graph_external_unique")
                    .table(GraphDataNodes::Table)
                    .col(GraphDataNodes::GraphDataId)
                    .col(GraphDataNodes::ExternalId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create indexes for performance
        manager
            .create_index(
                Index::create()
                    .name("idx_nodes_graph")
                    .table(GraphDataNodes::Table)
                    .col(GraphDataNodes::GraphDataId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_nodes_layer")
                    .table(GraphDataNodes::Table)
                    .col(GraphDataNodes::Layer)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_nodes_belongs_to")
                    .table(GraphDataNodes::Table)
                    .col(GraphDataNodes::GraphDataId)
                    .col(GraphDataNodes::BelongsTo)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GraphDataNodes::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum GraphDataNodes {
    Table,
    Id,
    GraphDataId,
    ExternalId,
    Label,
    Layer,
    Weight,
    IsPartition,
    BelongsTo,
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
