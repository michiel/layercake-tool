use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create users table
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Users::Email).string().not_null().unique_key())
                    .col(ColumnDef::new(Users::Username).string().not_null().unique_key())
                    .col(ColumnDef::new(Users::DisplayName).string().not_null())
                    .col(ColumnDef::new(Users::PasswordHash).string().not_null())
                    .col(ColumnDef::new(Users::AvatarColor).string().not_null())
                    .col(ColumnDef::new(Users::IsActive).boolean().not_null().default(true))
                    .col(ColumnDef::new(Users::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Users::UpdatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Users::LastLoginAt).timestamp_with_time_zone().null())
                    .to_owned(),
            )
            .await?;

        // Create user_sessions table
        manager
            .create_table(
                Table::create()
                    .table(UserSessions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserSessions::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserSessions::SessionId).string().not_null().unique_key())
                    .col(ColumnDef::new(UserSessions::UserId).integer().not_null())
                    .col(ColumnDef::new(UserSessions::UserName).string().not_null())
                    .col(ColumnDef::new(UserSessions::ProjectId).integer().not_null())
                    .col(ColumnDef::new(UserSessions::LayercakeGraphId).integer().null())
                    .col(ColumnDef::new(UserSessions::CursorPosition).string().null())
                    .col(ColumnDef::new(UserSessions::SelectedNodeId).string().null())
                    .col(ColumnDef::new(UserSessions::LastActivity).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(UserSessions::IsActive).boolean().not_null().default(true))
                    .col(ColumnDef::new(UserSessions::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(UserSessions::ExpiresAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_sessions_user_id")
                            .from(UserSessions::Table, UserSessions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_sessions_project_id")
                            .from(UserSessions::Table, UserSessions::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        // Create project_collaborators table
        manager
            .create_table(
                Table::create()
                    .table(ProjectCollaborators::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProjectCollaborators::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ProjectCollaborators::ProjectId).integer().not_null())
                    .col(ColumnDef::new(ProjectCollaborators::UserId).integer().not_null())
                    .col(ColumnDef::new(ProjectCollaborators::Role).string().not_null())
                    .col(ColumnDef::new(ProjectCollaborators::Permissions).string().not_null())
                    .col(ColumnDef::new(ProjectCollaborators::InvitedBy).integer().null())
                    .col(ColumnDef::new(ProjectCollaborators::InvitationStatus).string().not_null().default("pending"))
                    .col(ColumnDef::new(ProjectCollaborators::InvitedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(ProjectCollaborators::JoinedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(ProjectCollaborators::LastActiveAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(ProjectCollaborators::IsActive).boolean().not_null().default(true))
                    .col(ColumnDef::new(ProjectCollaborators::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(ProjectCollaborators::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_collaborators_project_id")
                            .from(ProjectCollaborators::Table, ProjectCollaborators::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_collaborators_user_id")
                            .from(ProjectCollaborators::Table, ProjectCollaborators::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_collaborators_invited_by")
                            .from(ProjectCollaborators::Table, ProjectCollaborators::InvitedBy)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                    )
                    .index(
                        Index::create()
                            .name("idx_project_collaborators_project_user")
                            .col(ProjectCollaborators::ProjectId)
                            .col(ProjectCollaborators::UserId)
                            .unique()
                    )
                    .to_owned(),
            )
            .await?;

        // Create user_presence table
        manager
            .create_table(
                Table::create()
                    .table(UserPresence::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserPresence::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserPresence::UserId).integer().not_null())
                    .col(ColumnDef::new(UserPresence::ProjectId).integer().not_null())
                    .col(ColumnDef::new(UserPresence::SessionId).string().not_null())
                    .col(ColumnDef::new(UserPresence::LayercakeGraphId).integer().null())
                    .col(ColumnDef::new(UserPresence::CursorPosition).string().null())
                    .col(ColumnDef::new(UserPresence::SelectedNodeId).string().null())
                    .col(ColumnDef::new(UserPresence::ViewportPosition).string().null())
                    .col(ColumnDef::new(UserPresence::CurrentTool).string().null())
                    .col(ColumnDef::new(UserPresence::IsOnline).boolean().not_null().default(true))
                    .col(ColumnDef::new(UserPresence::LastSeen).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(UserPresence::LastHeartbeat).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(UserPresence::Status).string().not_null().default("active"))
                    .col(ColumnDef::new(UserPresence::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(UserPresence::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_presence_user_id")
                            .from(UserPresence::Table, UserPresence::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_presence_project_id")
                            .from(UserPresence::Table, UserPresence::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_presence_session_id")
                            .from(UserPresence::Table, UserPresence::SessionId)
                            .to(UserSessions::Table, UserSessions::SessionId)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .index(
                        Index::create()
                            .name("idx_user_presence_project_user")
                            .col(UserPresence::ProjectId)
                            .col(UserPresence::UserId)
                            .unique()
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserPresence::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ProjectCollaborators::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(UserSessions::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Email,
    Username,
    DisplayName,
    PasswordHash,
    AvatarColor,
    IsActive,
    CreatedAt,
    UpdatedAt,
    LastLoginAt,
}

#[derive(DeriveIden)]
enum UserSessions {
    Table,
    Id,
    SessionId,
    UserId,
    UserName,
    ProjectId,
    LayercakeGraphId,
    CursorPosition,
    SelectedNodeId,
    LastActivity,
    IsActive,
    CreatedAt,
    ExpiresAt,
}

#[derive(DeriveIden)]
enum ProjectCollaborators {
    Table,
    Id,
    ProjectId,
    UserId,
    Role,
    Permissions,
    InvitedBy,
    InvitationStatus,
    InvitedAt,
    JoinedAt,
    LastActiveAt,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum UserPresence {
    Table,
    Id,
    UserId,
    ProjectId,
    SessionId,
    LayercakeGraphId,
    CursorPosition,
    SelectedNodeId,
    ViewportPosition,
    CurrentTool,
    IsOnline,
    LastSeen,
    LastHeartbeat,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
}