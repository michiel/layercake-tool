use sea_orm_migration::prelude::*;

/// Drop the orphaned `code_analysis_profiles` table.
///
/// The code-analysis feature was removed. Its original creation migration
/// (`m20251208_000003_create_code_analysis_profiles`) is kept in the migrator
/// so already-applied databases stay valid, but the table itself is no longer
/// used. This forward migration removes it. `if_exists()` makes it a no-op on
/// databases that never had the table.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .if_exists()
                    .table(CodeAnalysisProfiles::Table)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Recreate the table so a rollback restores the prior schema exactly
        // (matches m20251208_000003_create_code_analysis_profiles).
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
                    .col(
                        ColumnDef::new(CodeAnalysisProfiles::NoInfra)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(CodeAnalysisProfiles::Options).text().null())
                    .to_owned(),
            )
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
    NoInfra,
    Options,
}
