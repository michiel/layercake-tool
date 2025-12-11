use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Projections::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Projections::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Projections::ProjectId).integer().not_null())
                    .col(ColumnDef::new(Projections::GraphId).integer().not_null())
                    .col(ColumnDef::new(Projections::Name).string().not_null())
                    .col(
                        ColumnDef::new(Projections::ProjectionType)
                            .string()
                            .not_null()
                            .default("force3d"),
                    )
                    .col(
                        ColumnDef::new(Projections::SettingsJson)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Projections::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Projections::UpdatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_projections_project")
                            .from(Projections::Table, Projections::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_projections_graph_data")
                            .from(Projections::Table, Projections::GraphId)
                            .to(GraphData::Table, GraphData::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_projections_project")
                    .table(Projections::Table)
                    .col(Projections::ProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_projections_graph")
                    .table(Projections::Table)
                    .col(Projections::GraphId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Projections::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Projections {
    Table,
    Id,
    ProjectId,
    GraphId,
    Name,
    ProjectionType,
    SettingsJson,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Projects {
    Table,
    Id,
}

#[derive(Iden)]
enum GraphData {
    Table,
    Id,
}
