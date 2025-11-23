use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SequenceContexts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SequenceContexts::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SequenceContexts::ProjectId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SequenceContexts::NodeId)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(SequenceContexts::StoryId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SequenceContexts::ContextJson)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SequenceContexts::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SequenceContexts::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-sequence-contexts-project")
                            .from(SequenceContexts::Table, SequenceContexts::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SequenceContexts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum SequenceContexts {
    Table,
    Id,
    ProjectId,
    NodeId,
    StoryId,
    ContextJson,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
}
