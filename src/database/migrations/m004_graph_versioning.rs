use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create graph_snapshots table
        manager
            .create_table(
                Table::create()
                    .table(GraphSnapshots::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GraphSnapshots::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(GraphSnapshots::ProjectId).integer().not_null())
                    .col(ColumnDef::new(GraphSnapshots::Name).text().not_null())
                    .col(ColumnDef::new(GraphSnapshots::Description).text())
                    .col(ColumnDef::new(GraphSnapshots::Version).integer().not_null())
                    .col(ColumnDef::new(GraphSnapshots::IsAutomatic).boolean().not_null().default(false))
                    .col(ColumnDef::new(GraphSnapshots::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(GraphSnapshots::CreatedBy).text())
                    .col(ColumnDef::new(GraphSnapshots::NodeCount).integer().not_null().default(0))
                    .col(ColumnDef::new(GraphSnapshots::EdgeCount).integer().not_null().default(0))
                    .col(ColumnDef::new(GraphSnapshots::LayerCount).integer().not_null().default(0))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_graph_snapshots_project_id")
                            .from(GraphSnapshots::Table, GraphSnapshots::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        // Create graph_versions table for tracking changes
        manager
            .create_table(
                Table::create()
                    .table(GraphVersions::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GraphVersions::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(GraphVersions::ProjectId).integer().not_null())
                    .col(ColumnDef::new(GraphVersions::SnapshotId).integer())
                    .col(ColumnDef::new(GraphVersions::ChangeType).text().not_null())
                    .col(ColumnDef::new(GraphVersions::EntityType).text().not_null())
                    .col(ColumnDef::new(GraphVersions::EntityId).text().not_null())
                    .col(ColumnDef::new(GraphVersions::OldData).json())
                    .col(ColumnDef::new(GraphVersions::NewData).json())
                    .col(ColumnDef::new(GraphVersions::ChangedAt).timestamp().not_null())
                    .col(ColumnDef::new(GraphVersions::ChangedBy).text())
                    .col(ColumnDef::new(GraphVersions::ChangeDescription).text())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_graph_versions_project_id")
                            .from(GraphVersions::Table, GraphVersions::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_graph_versions_snapshot_id")
                            .from(GraphVersions::Table, GraphVersions::SnapshotId)
                            .to(GraphSnapshots::Table, GraphSnapshots::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                    )
                    .to_owned(),
            )
            .await?;

        // Create snapshot_data table for storing graph data at snapshot time
        manager
            .create_table(
                Table::create()
                    .table(SnapshotData::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(SnapshotData::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(SnapshotData::SnapshotId).integer().not_null())
                    .col(ColumnDef::new(SnapshotData::EntityType).text().not_null())
                    .col(ColumnDef::new(SnapshotData::EntityId).text().not_null())
                    .col(ColumnDef::new(SnapshotData::EntityData).json().not_null())
                    .col(ColumnDef::new(SnapshotData::CreatedAt).timestamp().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_snapshot_data_snapshot_id")
                            .from(SnapshotData::Table, SnapshotData::SnapshotId)
                            .to(GraphSnapshots::Table, GraphSnapshots::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for performance
        manager
            .create_index(
                Index::create()
                    .name("idx_graph_snapshots_project_id")
                    .table(GraphSnapshots::Table)
                    .col(GraphSnapshots::ProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_graph_versions_project_id")
                    .table(GraphVersions::Table)
                    .col(GraphVersions::ProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_graph_versions_entity")
                    .table(GraphVersions::Table)
                    .col(GraphVersions::EntityType)
                    .col(GraphVersions::EntityId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_snapshot_data_snapshot_id")
                    .table(SnapshotData::Table)
                    .col(SnapshotData::SnapshotId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SnapshotData::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(GraphVersions::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(GraphSnapshots::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum GraphSnapshots {
    Table,
    Id,
    ProjectId,
    Name,
    Description,
    Version,
    IsAutomatic,
    CreatedAt,
    CreatedBy,
    NodeCount,
    EdgeCount,
    LayerCount,
}

#[derive(DeriveIden)]
enum GraphVersions {
    Table,
    Id,
    ProjectId,
    SnapshotId,
    ChangeType,
    EntityType,
    EntityId,
    OldData,
    NewData,
    ChangedAt,
    ChangedBy,
    ChangeDescription,
}

#[derive(DeriveIden)]
enum SnapshotData {
    Table,
    Id,
    SnapshotId,
    EntityType,
    EntityId,
    EntityData,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
}
