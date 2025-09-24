use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add Plan DAG JSON field to existing plans table
        manager
            .alter_table(
                Table::alter()
                    .table(Plans::Table)
                    .add_column(ColumnDef::new(Plans::PlanDagJson).text())
                    .to_owned(),
            )
            .await?;

        // Create plan_dag_nodes table for structured Plan DAG node storage
        manager
            .create_table(
                Table::create()
                    .table(PlanDagNodes::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(PlanDagNodes::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(PlanDagNodes::PlanId).integer().not_null())
                    .col(ColumnDef::new(PlanDagNodes::NodeType).string().not_null())
                    .col(ColumnDef::new(PlanDagNodes::PositionX).double().not_null())
                    .col(ColumnDef::new(PlanDagNodes::PositionY).double().not_null())
                    .col(ColumnDef::new(PlanDagNodes::MetadataJson).text().not_null())
                    .col(ColumnDef::new(PlanDagNodes::ConfigJson).text().not_null())
                    .col(ColumnDef::new(PlanDagNodes::CreatedAt).text().not_null())
                    .col(ColumnDef::new(PlanDagNodes::UpdatedAt).text().not_null())
                    .to_owned(),
            )
            .await?;

        // Create plan_dag_edges table for structured Plan DAG edge storage
        manager
            .create_table(
                Table::create()
                    .table(PlanDagEdges::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(PlanDagEdges::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(PlanDagEdges::PlanId).integer().not_null())
                    .col(ColumnDef::new(PlanDagEdges::SourceNodeId).string().not_null())
                    .col(ColumnDef::new(PlanDagEdges::TargetNodeId).string().not_null())
                    .col(ColumnDef::new(PlanDagEdges::MetadataJson).text().not_null())
                    .col(ColumnDef::new(PlanDagEdges::CreatedAt).text().not_null())
                    .col(ColumnDef::new(PlanDagEdges::UpdatedAt).text().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop Plan DAG tables
        manager
            .drop_table(Table::drop().table(PlanDagEdges::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PlanDagNodes::Table).to_owned())
            .await?;

        // Remove Plan DAG JSON column from plans table
        manager
            .alter_table(
                Table::alter()
                    .table(Plans::Table)
                    .drop_column(Plans::PlanDagJson)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

// Existing Plans table reference for adding column
#[derive(Iden)]
enum Plans {
    Table,
    #[allow(dead_code)]
    Id,
    PlanDagJson,
}

#[derive(Iden)]
enum PlanDagNodes {
    Table,
    Id,
    PlanId,
    NodeType,
    PositionX,
    PositionY,
    MetadataJson,
    ConfigJson,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum PlanDagEdges {
    Table,
    Id,
    PlanId,
    SourceNodeId,
    TargetNodeId,
    MetadataJson,
    CreatedAt,
    UpdatedAt,
}