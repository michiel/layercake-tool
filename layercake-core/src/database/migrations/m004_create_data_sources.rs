use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create data_sources table
        manager
            .create_table(
                Table::create()
                    .table(DataSources::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DataSources::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(DataSources::ProjectId).integer().not_null())
                    .col(ColumnDef::new(DataSources::Name).string().not_null())
                    .col(ColumnDef::new(DataSources::Description).string())
                    .col(ColumnDef::new(DataSources::SourceType).string().not_null())
                    .col(ColumnDef::new(DataSources::Filename).string().not_null())
                    .col(ColumnDef::new(DataSources::Blob).binary().not_null())
                    .col(ColumnDef::new(DataSources::GraphJson).text().not_null())
                    .col(
                        ColumnDef::new(DataSources::Status)
                            .string()
                            .not_null()
                            .default("processing"),
                    )
                    .col(ColumnDef::new(DataSources::ErrorMessage).string())
                    .col(ColumnDef::new(DataSources::FileSize).big_integer().not_null())
                    .col(ColumnDef::new(DataSources::ProcessedAt).timestamp())
                    .col(ColumnDef::new(DataSources::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(DataSources::UpdatedAt).timestamp().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_data_sources_project_id")
                            .from(DataSources::Table, DataSources::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for better query performance
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_data_sources_project_id")
                    .table(DataSources::Table)
                    .col(DataSources::ProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_data_sources_status")
                    .table(DataSources::Table)
                    .col(DataSources::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_data_sources_source_type")
                    .table(DataSources::Table)
                    .col(DataSources::SourceType)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indexes first
        manager
            .drop_index(Index::drop().name("idx_data_sources_source_type").to_owned())
            .await
            .ok();

        manager
            .drop_index(Index::drop().name("idx_data_sources_status").to_owned())
            .await
            .ok();

        manager
            .drop_index(Index::drop().name("idx_data_sources_project_id").to_owned())
            .await
            .ok();

        // Drop the table
        manager
            .drop_table(Table::drop().table(DataSources::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum DataSources {
    Table,
    Id,
    ProjectId,
    Name,
    Description,
    SourceType,
    Filename,
    Blob,
    GraphJson,
    Status,
    ErrorMessage,
    FileSize,
    ProcessedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
}