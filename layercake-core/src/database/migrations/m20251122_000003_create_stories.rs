use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Stories::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Stories::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Stories::ProjectId).integer().not_null())
                    .col(ColumnDef::new(Stories::Name).string().not_null())
                    .col(ColumnDef::new(Stories::Description).text())
                    .col(
                        ColumnDef::new(Stories::Tags)
                            .text()
                            .not_null()
                            .default("[]"),
                    )
                    .col(
                        ColumnDef::new(Stories::EnabledDatasetIds)
                            .text()
                            .not_null()
                            .default("[]"),
                    )
                    .col(
                        ColumnDef::new(Stories::LayerConfig)
                            .text()
                            .not_null()
                            .default("{}"),
                    )
                    .col(ColumnDef::new(Stories::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Stories::UpdatedAt).timestamp().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_stories_project_id")
                            .from(Stories::Table, Stories::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_stories_project_id")
                    .table(Stories::Table)
                    .col(Stories::ProjectId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Stories::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Stories {
    Table,
    Id,
    ProjectId,
    Name,
    Description,
    Tags,
    EnabledDatasetIds,
    LayerConfig,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Projects {
    Table,
    Id,
}
