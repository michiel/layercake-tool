use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add foreign key constraints for plan_dag_nodes
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_plan_dag_nodes_plan_id")
                    .from(PlanDagNodes::Table, PlanDagNodes::PlanId)
                    .to(Plans::Table, Plans::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraints for plan_dag_edges
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_plan_dag_edges_plan_id")
                    .from(PlanDagEdges::Table, PlanDagEdges::PlanId)
                    .to(Plans::Table, Plans::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        // Create indexes for better query performance
        manager
            .create_index(
                Index::create()
                    .name("idx_plan_dag_nodes_plan_id")
                    .table(PlanDagNodes::Table)
                    .col(PlanDagNodes::PlanId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_plan_dag_edges_plan_id")
                    .table(PlanDagEdges::Table)
                    .col(PlanDagEdges::PlanId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indexes
        manager
            .drop_index(
                Index::drop()
                    .name("idx_plan_dag_edges_plan_id")
                    .table(PlanDagEdges::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_plan_dag_nodes_plan_id")
                    .table(PlanDagNodes::Table)
                    .to_owned(),
            )
            .await?;

        // Drop foreign key constraints
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_plan_dag_edges_plan_id")
                    .table(PlanDagEdges::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_plan_dag_nodes_plan_id")
                    .table(PlanDagNodes::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Plans {
    Table,
    Id,
}

#[derive(Iden)]
enum PlanDagNodes {
    Table,
    PlanId,
}

#[derive(Iden)]
enum PlanDagEdges {
    Table,
    PlanId,
}
