use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // No-op migration: Database tables already use correct naming (data_sets, dataset_nodes, dataset_rows)
        // This migration was created during code-level refactoring but no database changes were needed.
        // The tables have been using the "dataset" naming since m20251110_000016_create_data_acquisition_tables.
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // No-op: nothing to revert
        Ok(())
    }
}

// Enums removed - not needed for no-op migration
