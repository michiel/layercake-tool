use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create plan_nodes table
        manager
            .create_table(
                Table::create()
                    .table(PlanNodes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PlanNodes::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PlanNodes::PlanId).integer().not_null())
                    .col(ColumnDef::new(PlanNodes::NodeType).string().not_null())
                    .col(ColumnDef::new(PlanNodes::Name).string().not_null())
                    .col(ColumnDef::new(PlanNodes::Description).string())
                    .col(ColumnDef::new(PlanNodes::Configuration).text().not_null())
                    .col(ColumnDef::new(PlanNodes::GraphId).string())
                    .col(ColumnDef::new(PlanNodes::PositionX).double())
                    .col(ColumnDef::new(PlanNodes::PositionY).double())
                    .col(ColumnDef::new(PlanNodes::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(PlanNodes::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-plan_nodes-plan_id")
                            .from(PlanNodes::Table, PlanNodes::PlanId)
                            .to(Plans::Table, Plans::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create graphs table
        manager
            .create_table(
                Table::create()
                    .table(Graphs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Graphs::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Graphs::PlanId).integer().not_null())
                    .col(ColumnDef::new(Graphs::PlanNodeId).string().not_null())
                    .col(ColumnDef::new(Graphs::Name).string().not_null())
                    .col(ColumnDef::new(Graphs::Description).string())
                    .col(ColumnDef::new(Graphs::GraphData).text().not_null())
                    .col(ColumnDef::new(Graphs::Metadata).text())
                    .col(ColumnDef::new(Graphs::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Graphs::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-graphs-plan_id")
                            .from(Graphs::Table, Graphs::PlanId)
                            .to(Plans::Table, Plans::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-graphs-plan_node_id")
                            .from(Graphs::Table, Graphs::PlanNodeId)
                            .to(PlanNodes::Table, PlanNodes::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Note: Foreign key from plan_nodes.graph_id to graphs.id will be added
        // in a separate migration after both tables are created and populated

        // Create indexes for performance
        manager
            .create_index(
                Index::create()
                    .name("idx-plan_nodes-plan_id")
                    .table(PlanNodes::Table)
                    .col(PlanNodes::PlanId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-plan_nodes-node_type")
                    .table(PlanNodes::Table)
                    .col(PlanNodes::NodeType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-graphs-plan_id")
                    .table(Graphs::Table)
                    .col(Graphs::PlanId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-graphs-plan_node_id")
                    .table(Graphs::Table)
                    .col(Graphs::PlanNodeId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        // Drop indexes
        manager
            .drop_index(Index::drop().name("idx-graphs-plan_node_id").to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx-graphs-plan_id").to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx-plan_nodes-node_type").to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx-plan_nodes-plan_id").to_owned())
            .await?;

        // Drop tables
        manager
            .drop_table(Table::drop().table(Graphs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PlanNodes::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum PlanNodes {
    Table,
    Id,
    PlanId,
    NodeType,
    Name,
    Description,
    Configuration,
    GraphId,
    PositionX,
    PositionY,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Graphs {
    Table,
    Id,
    PlanId,
    PlanNodeId,
    Name,
    Description,
    GraphData,
    Metadata,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Plans {
    Table,
    Id,
}