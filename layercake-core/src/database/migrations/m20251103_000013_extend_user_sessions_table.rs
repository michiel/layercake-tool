use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add auth_method column
        manager
            .alter_table(
                Table::alter()
                    .table(UserSessions::Table)
                    .add_column(
                        ColumnDef::new(UserSessions::AuthMethod)
                            .string()
                            .not_null()
                            .default("password"),
                    )
                    .to_owned(),
            )
            .await?;

        // Add auth_context column for storing additional auth metadata as JSON text
        manager
            .alter_table(
                Table::alter()
                    .table(UserSessions::Table)
                    .add_column(ColumnDef::new(UserSessions::AuthContext).string())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(UserSessions::Table)
                    .drop_column(UserSessions::AuthContext)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(UserSessions::Table)
                    .drop_column(UserSessions::AuthMethod)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum UserSessions {
    Table,
    AuthMethod,
    AuthContext,
}
