use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Sequences::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Sequences::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Sequences::StoryId).integer().not_null())
                    .col(ColumnDef::new(Sequences::Name).string().not_null())
                    .col(ColumnDef::new(Sequences::Description).text())
                    .col(
                        ColumnDef::new(Sequences::EnabledDatasetIds)
                            .text()
                            .not_null()
                            .default("[]"),
                    )
                    .col(
                        ColumnDef::new(Sequences::EdgeOrder)
                            .text()
                            .not_null()
                            .default("[]"),
                    )
                    .col(ColumnDef::new(Sequences::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Sequences::UpdatedAt).timestamp().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sequences_story_id")
                            .from(Sequences::Table, Sequences::StoryId)
                            .to(Stories::Table, Stories::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_sequences_story_id")
                    .table(Sequences::Table)
                    .col(Sequences::StoryId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Sequences::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Sequences {
    Table,
    Id,
    StoryId,
    Name,
    Description,
    EnabledDatasetIds,
    EdgeOrder,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Stories {
    Table,
    Id,
}
