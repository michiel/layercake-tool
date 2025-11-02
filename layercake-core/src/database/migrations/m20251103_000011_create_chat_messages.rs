use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ChatMessages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChatMessages::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ChatMessages::SessionId).integer().not_null())
                    .col(
                        ColumnDef::new(ChatMessages::MessageId)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(ChatMessages::Role).string().not_null())
                    .col(ColumnDef::new(ChatMessages::Content).text().not_null())
                    .col(ColumnDef::new(ChatMessages::ToolName).string())
                    .col(ColumnDef::new(ChatMessages::ToolCallId).string())
                    .col(ColumnDef::new(ChatMessages::MetadataJson).text())
                    .col(ColumnDef::new(ChatMessages::CreatedAt).timestamp().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_chat_messages_session_id")
                            .from(ChatMessages::Table, ChatMessages::SessionId)
                            .to(ChatSessions::Table, ChatSessions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(Index::create().name("idx_chat_messages_session").col(ChatMessages::SessionId))
                    .index(Index::create().name("idx_chat_messages_created").col(ChatMessages::CreatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ChatMessages::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum ChatMessages {
    Table,
    Id,
    SessionId,
    MessageId,
    Role,
    Content,
    ToolName,
    ToolCallId,
    MetadataJson,
    CreatedAt,
}

#[derive(Iden)]
enum ChatSessions {
    Table,
    Id,
}
