use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create the table first
        manager
            .create_table(
                Table::create()
                    .table(LayerAliases::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LayerAliases::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(LayerAliases::ProjectId).integer().not_null())
                    .col(
                        ColumnDef::new(LayerAliases::AliasLayerId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(LayerAliases::TargetLayerId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(LayerAliases::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_layer_aliases_project")
                            .from(LayerAliases::Table, LayerAliases::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_layer_aliases_target_layer")
                            .from(LayerAliases::Table, LayerAliases::TargetLayerId)
                            .to(ProjectLayers::Table, ProjectLayers::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes separately
        manager
            .create_index(
                Index::create()
                    .name("idx_layer_aliases_project")
                    .table(LayerAliases::Table)
                    .col(LayerAliases::ProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_layer_aliases_target")
                    .table(LayerAliases::Table)
                    .col(LayerAliases::TargetLayerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_layer_aliases_project_alias_unique")
                    .table(LayerAliases::Table)
                    .col(LayerAliases::ProjectId)
                    .col(LayerAliases::AliasLayerId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(LayerAliases::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum LayerAliases {
    Table,
    Id,
    ProjectId,
    AliasLayerId,
    TargetLayerId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum ProjectLayers {
    Table,
    Id,
}
