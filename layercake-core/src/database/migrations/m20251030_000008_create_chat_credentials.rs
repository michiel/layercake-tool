use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ChatCredentials::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChatCredentials::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ChatCredentials::Provider)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(ChatCredentials::ApiKey).text())
                    .col(ColumnDef::new(ChatCredentials::BaseUrl).text())
                    .col(
                        ColumnDef::new(ChatCredentials::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChatCredentials::UpdatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ChatCredentials::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum ChatCredentials {
    Table,
    Id,
    Provider,
    ApiKey,
    BaseUrl,
    CreatedAt,
    UpdatedAt,
}
