use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Rename yaml_content to plan_content for clarity
        manager
            .alter_table(
                Table::alter()
                    .table(Plans::Table)
                    .rename_column(Plans::YamlContent, Plans::PlanContent)
                    .to_owned(),
            )
            .await?;

        // Add plan_schema_version column to track JSON schema version
        manager
            .alter_table(
                Table::alter()
                    .table(Plans::Table)
                    .add_column(
                        ColumnDef::new(Plans::PlanSchemaVersion)
                            .string()
                            .not_null()
                            .default("1.0.0")
                    )
                    .to_owned(),
            )
            .await?;

        // Add plan_format column to indicate content format (json/yaml for backward compatibility)
        manager
            .alter_table(
                Table::alter()
                    .table(Plans::Table)
                    .add_column(
                        ColumnDef::new(Plans::PlanFormat)
                            .string()
                            .not_null()
                            .default("yaml")
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove the new columns
        manager
            .alter_table(
                Table::alter()
                    .table(Plans::Table)
                    .drop_column(Plans::PlanFormat)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Plans::Table)
                    .drop_column(Plans::PlanSchemaVersion)
                    .to_owned(),
            )
            .await?;

        // Rename back to yaml_content
        manager
            .alter_table(
                Table::alter()
                    .table(Plans::Table)
                    .rename_column(Plans::PlanContent, Plans::YamlContent)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Plans {
    Table,
    YamlContent,
    PlanContent,
    PlanSchemaVersion,
    PlanFormat,
}