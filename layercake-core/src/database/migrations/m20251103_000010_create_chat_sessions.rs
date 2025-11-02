use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ChatSessions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChatSessions::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ChatSessions::SessionId)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(ChatSessions::ProjectId).integer().not_null())
                    .col(ColumnDef::new(ChatSessions::UserId).integer().not_null())
                    .col(ColumnDef::new(ChatSessions::Title).string())
                    .col(ColumnDef::new(ChatSessions::Provider).string().not_null())
                    .col(ColumnDef::new(ChatSessions::ModelName).string().not_null())
                    .col(ColumnDef::new(ChatSessions::SystemPrompt).text())
                    .col(
                        ColumnDef::new(ChatSessions::IsArchived)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(ChatSessions::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(ChatSessions::UpdatedAt).timestamp().not_null())
                    .col(
                        ColumnDef::new(ChatSessions::LastActivityAt)
                            .timestamp()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_chat_sessions_project_id")
                            .from(ChatSessions::Table, ChatSessions::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_chat_sessions_user_id")
                            .from(ChatSessions::Table, ChatSessions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(Index::create().name("idx_chat_sessions_project").col(ChatSessions::ProjectId))
                    .index(Index::create().name("idx_chat_sessions_user").col(ChatSessions::UserId))
                    .index(Index::create().name("idx_chat_sessions_activity").col(ChatSessions::LastActivityAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ChatSessions::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum ChatSessions {
    Table,
    Id,
    SessionId,
    ProjectId,
    UserId,
    Title,
    Provider,
    ModelName,
    SystemPrompt,
    IsArchived,
    CreatedAt,
    UpdatedAt,
    LastActivityAt,
}

#[derive(Iden)]
enum Projects {
    Table,
    Id,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
