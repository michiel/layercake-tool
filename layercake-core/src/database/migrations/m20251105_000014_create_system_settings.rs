use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SystemSettings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SystemSettings::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SystemSettings::Key)
                            .string_len(128)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(SystemSettings::Value)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(SystemSettings::ValueType)
                            .string_len(32)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SystemSettings::Label)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SystemSettings::Category)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(ColumnDef::new(SystemSettings::Description).text())
                    .col(
                        ColumnDef::new(SystemSettings::IsSecret)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(SystemSettings::IsReadOnly)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(SystemSettings::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SystemSettings::UpdatedAt)
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
            .drop_table(Table::drop().table(SystemSettings::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum SystemSettings {
    Table,
    Id,
    Key,
    Value,
    ValueType,
    Label,
    Category,
    Description,
    IsSecret,
    IsReadOnly,
    CreatedAt,
    UpdatedAt,
}
