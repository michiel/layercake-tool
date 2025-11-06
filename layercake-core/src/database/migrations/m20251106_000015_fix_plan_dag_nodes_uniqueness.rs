use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add unique constraint on (plan_id, id) to enforce that node IDs are unique per plan
        // This provides the logical constraint while keeping 'id' as the primary key for
        // simplicity of foreign key references.
        //
        // Going forward, ID generation will use UUIDs to ensure global uniqueness,
        // preventing the UNIQUE constraint error that occurred with sequential IDs.
        manager
            .create_index(
                Index::create()
                    .name("idx_plan_dag_nodes_plan_id_unique")
                    .table(PlanDagNodes::Table)
                    .col(PlanDagNodes::PlanId)
                    .col(PlanDagNodes::Id)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Do the same for plan_dag_edges
        manager
            .create_index(
                Index::create()
                    .name("idx_plan_dag_edges_plan_id_unique")
                    .table(PlanDagEdges::Table)
                    .col(PlanDagEdges::PlanId)
                    .col(PlanDagEdges::Id)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_plan_dag_nodes_plan_id_unique")
                    .table(PlanDagNodes::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_plan_dag_edges_plan_id_unique")
                    .table(PlanDagEdges::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum PlanDagNodes {
    Table,
    Id,
    PlanId,
}

#[derive(Iden)]
enum PlanDagEdges {
    Table,
    Id,
    PlanId,
}
