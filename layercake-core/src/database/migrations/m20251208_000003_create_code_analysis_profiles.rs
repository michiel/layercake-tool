use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CodeAnalysisProfiles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CodeAnalysisProfiles::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(CodeAnalysisProfiles::ProjectId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CodeAnalysisProfiles::FilePath)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CodeAnalysisProfiles::DatasetId)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(CodeAnalysisProfiles::LastRun)
                            .timestamp()
                            .null(),
                    )
                    .col(ColumnDef::new(CodeAnalysisProfiles::Report).text().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CodeAnalysisProfiles::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum CodeAnalysisProfiles {
    Table,
    Id,
    ProjectId,
    FilePath,
    DatasetId,
    LastRun,
    Report,
}
