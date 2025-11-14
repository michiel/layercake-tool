use sea_orm::{ConnectionTrait, Statement};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(LibraryItems::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LibraryItems::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(LibraryItems::ItemType)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(LibraryItems::Name).string().not_null())
                    .col(ColumnDef::new(LibraryItems::Description).string())
                    .col(
                        ColumnDef::new(LibraryItems::Tags)
                            .text()
                            .not_null()
                            .default("[]"),
                    )
                    .col(
                        ColumnDef::new(LibraryItems::Metadata)
                            .text()
                            .not_null()
                            .default("{}"),
                    )
                    .col(
                        ColumnDef::new(LibraryItems::ContentBlob)
                            .binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(LibraryItems::ContentSize).big_integer())
                    .col(ColumnDef::new(LibraryItems::ContentType).string())
                    .col(
                        ColumnDef::new(LibraryItems::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(LibraryItems::UpdatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-library-items-type-name")
                    .table(LibraryItems::Table)
                    .col(LibraryItems::ItemType)
                    .col(LibraryItems::Name)
                    .to_owned(),
            )
            .await?;

        if manager
            .has_table(LibrarySources::Table.to_string())
            .await?
        {
            manager
                .get_connection()
                .execute(Statement::from_string(
                    manager.get_database_backend(),
                    r#"
                    INSERT INTO library_items
                        (id, item_type, name, description, tags, metadata, content_blob, content_size, content_type, created_at, updated_at)
                    SELECT
                        id,
                        'dataset' AS item_type,
                        name,
                        description,
                        COALESCE(tags_column.tags_json, '[]'),
                        json_object(
                            'legacy_filename', filename,
                            'file_format', file_format,
                            'data_type', data_type,
                            'status', status,
                            'error_message', COALESCE(error_message, ''),
                            'processed_at', COALESCE(processed_at, ''),
                            'graph_json', graph_json
                        ),
                        blob,
                        file_size,
                        CASE
                            WHEN lower(file_format) = 'csv' THEN 'text/csv'
                            WHEN lower(file_format) = 'tsv' THEN 'text/tab-separated-values'
                            WHEN lower(file_format) = 'json' THEN 'application/json'
                            ELSE 'application/octet-stream'
                        END,
                        created_at,
                        updated_at
                    FROM (
                        SELECT ls.*, '[]' as tags_json
                        FROM library_sources ls
                    ) AS tags_column;
                "#,
                ))
                .await?;

            manager
                .drop_table(
                    Table::drop()
                        .table(LibrarySources::Table)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if !manager
            .has_table(LibrarySources::Table.to_string())
            .await?
        {
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
                .await?;
        }

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                r#"
                INSERT INTO library_sources
                    (id, name, description, filename, blob, graph_json, status, error_message, file_size, processed_at, file_format, data_type, created_at, updated_at)
                SELECT
                    id,
                    name,
                    description,
                    COALESCE(json_extract(metadata, '$.legacy_filename'), name || '.csv'),
                    content_blob,
                    COALESCE(json_extract(metadata, '$.graph_json'), '{}'),
                    COALESCE(json_extract(metadata, '$.status'), 'active'),
                    NULLIF(json_extract(metadata, '$.error_message'), ''),
                    COALESCE(content_size, length(content_blob)),
                    NULLIF(json_extract(metadata, '$.processed_at'), ''),
                    COALESCE(json_extract(metadata, '$.file_format'), 'csv'),
                    COALESCE(json_extract(metadata, '$.data_type'), 'graph'),
                    created_at,
                    updated_at
                FROM library_items
                WHERE item_type = 'dataset';
            "#,
            ))
            .await?;

        manager
            .drop_table(Table::drop().table(LibraryItems::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum LibraryItems {
    Table,
    Id,
    ItemType,
    Name,
    Description,
    Tags,
    Metadata,
    ContentBlob,
    ContentSize,
    ContentType,
    CreatedAt,
    UpdatedAt,
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
