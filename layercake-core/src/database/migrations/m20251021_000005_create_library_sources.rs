use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(LibrarySources::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LibrarySources::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(LibrarySources::Name).string().not_null())
                    .col(ColumnDef::new(LibrarySources::Description).string())
                    .col(ColumnDef::new(LibrarySources::Filename).string().not_null())
                    .col(ColumnDef::new(LibrarySources::Blob).binary().not_null())
                    .col(ColumnDef::new(LibrarySources::GraphJson).text().not_null())
                    .col(
                        ColumnDef::new(LibrarySources::Status)
                            .string()
                            .not_null()
                            .default("processing"),
                    )
                    .col(ColumnDef::new(LibrarySources::ErrorMessage).string())
                    .col(
                        ColumnDef::new(LibrarySources::FileSize)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(LibrarySources::ProcessedAt).timestamp())
                    .col(
                        ColumnDef::new(LibrarySources::FileFormat)
                            .string()
                            .not_null()
                            .default("csv"),
                    )
                    .col(
                        ColumnDef::new(LibrarySources::DataType)
                            .string()
                            .not_null()
                            .default("nodes"),
                    )
                    .col(
                        ColumnDef::new(LibrarySources::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(LibrarySources::UpdatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(LibrarySources::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum LibrarySources {
    Table,
    Id,
    Name,
    Description,
    Filename,
    Blob,
    GraphJson,
    Status,
    ErrorMessage,
    FileSize,
    ProcessedAt,
    FileFormat,
    DataType,
    CreatedAt,
    UpdatedAt,
}
