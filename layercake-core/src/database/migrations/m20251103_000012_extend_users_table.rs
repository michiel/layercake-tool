use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add user_type column
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::UserType)
                            .string()
                            .not_null()
                            .default("human"),
                    )
                    .to_owned(),
            )
            .await?;

        // Add scoped_project_id column (nullable for human users)
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(ColumnDef::new(Users::ScopedProjectId).integer())
                    .to_owned(),
            )
            .await?;

        // Add api_key_hash column (nullable for human users)
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(ColumnDef::new(Users::ApiKeyHash).text())
                    .to_owned(),
            )
            .await?;

        // Add organisation_id column (nullable, for future use)
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(ColumnDef::new(Users::OrganisationId).integer())
                    .to_owned(),
            )
            .await?;

        // Create index on scoped_project_id for MCP agents
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_users_scoped_project")
                    .table(Users::Table)
                    .col(Users::ScopedProjectId)
                    .to_owned(),
            )
            .await?;

        // Create index on organisation_id for future multi-tenancy
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_users_organisation")
                    .table(Users::Table)
                    .col(Users::OrganisationId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indices
        manager
            .drop_index(
                Index::drop()
                    .name("idx_users_organisation")
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_users_scoped_project")
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        // Drop columns
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::OrganisationId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::ApiKeyHash)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::ScopedProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::UserType)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Users {
    Table,
    UserType,
    ScopedProjectId,
    ApiKeyHash,
    OrganisationId,
}
