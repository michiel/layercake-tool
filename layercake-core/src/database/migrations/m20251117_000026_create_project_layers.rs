use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProjectLayers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProjectLayers::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ProjectLayers::ProjectId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProjectLayers::LayerId).string().not_null())
                    .col(ColumnDef::new(ProjectLayers::Name).string().not_null())
                    .col(
                        ColumnDef::new(ProjectLayers::BackgroundColor)
                            .string()
                            .not_null()
                            .default("FFFFFF"),
                    )
                    .col(
                        ColumnDef::new(ProjectLayers::TextColor)
                            .string()
                            .not_null()
                            .default("000000"),
                    )
                    .col(
                        ColumnDef::new(ProjectLayers::BorderColor)
                            .string()
                            .not_null()
                            .default("000000"),
                    )
                    .col(
                        ColumnDef::new(ProjectLayers::SourceDatasetId)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ProjectLayers::Enabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(ProjectLayers::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        ColumnDef::new(ProjectLayers::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .index(
                        Index::create()
                            .name("idx_project_layers_project_layer_source")
                            .col(ProjectLayers::ProjectId)
                            .col(ProjectLayers::LayerId)
                            .col(ProjectLayers::SourceDatasetId)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_layers_project")
                            .from(ProjectLayers::Table, ProjectLayers::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_layers_dataset")
                            .from(ProjectLayers::Table, ProjectLayers::SourceDatasetId)
                            .to(DataSets::Table, DataSets::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProjectLayers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ProjectLayers {
    Table,
    Id,
    ProjectId,
    LayerId,
    Name,
    BackgroundColor,
    TextColor,
    BorderColor,
    SourceDatasetId,
    Enabled,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum DataSets {
    Table,
    Id,
}
