use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create projects table
        manager
            .create_table(
                Table::create()
                    .table(Projects::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Projects::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Projects::Name).string().not_null())
                    .col(ColumnDef::new(Projects::Description).string())
                    .col(ColumnDef::new(Projects::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Projects::UpdatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await?;

        // Create plans table
        manager
            .create_table(
                Table::create()
                    .table(Plans::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Plans::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Plans::ProjectId).integer().not_null())
                    .col(ColumnDef::new(Plans::Name).string().not_null())
                    .col(ColumnDef::new(Plans::YamlContent).text().not_null())
                    .col(ColumnDef::new(Plans::Dependencies).string())
                    .col(ColumnDef::new(Plans::Status).string().not_null().default("pending"))
                    .col(ColumnDef::new(Plans::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Plans::UpdatedAt).timestamp().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_plans_project_id")
                            .from(Plans::Table, Plans::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create nodes table
        manager
            .create_table(
                Table::create()
                    .table(Nodes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Nodes::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Nodes::ProjectId).integer().not_null())
                    .col(ColumnDef::new(Nodes::NodeId).string().not_null())
                    .col(ColumnDef::new(Nodes::Label).string().not_null())
                    .col(ColumnDef::new(Nodes::LayerId).string())
                    .col(ColumnDef::new(Nodes::Properties).text())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_nodes_project_id")
                            .from(Nodes::Table, Nodes::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_nodes_project_node_id")
                            .table(Nodes::Table)
                            .col(Nodes::ProjectId)
                            .col(Nodes::NodeId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create edges table
        manager
            .create_table(
                Table::create()
                    .table(Edges::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Edges::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Edges::ProjectId).integer().not_null())
                    .col(ColumnDef::new(Edges::SourceNodeId).string().not_null())
                    .col(ColumnDef::new(Edges::TargetNodeId).string().not_null())
                    .col(ColumnDef::new(Edges::Properties).text())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_edges_project_id")
                            .from(Edges::Table, Edges::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create layers table
        manager
            .create_table(
                Table::create()
                    .table(Layers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Layers::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Layers::ProjectId).integer().not_null())
                    .col(ColumnDef::new(Layers::LayerId).string().not_null())
                    .col(ColumnDef::new(Layers::Name).string().not_null())
                    .col(ColumnDef::new(Layers::Color).string())
                    .col(ColumnDef::new(Layers::Properties).text())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_layers_project_id")
                            .from(Layers::Table, Layers::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_layers_project_layer_id")
                            .table(Layers::Table)
                            .col(Layers::ProjectId)
                            .col(Layers::LayerId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Layers::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Edges::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Nodes::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Plans::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Projects::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Projects {
    Table,
    Id,
    Name,
    Description,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Plans {
    Table,
    Id,
    ProjectId,
    Name,
    YamlContent,
    Dependencies,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Nodes {
    Table,
    Id,
    ProjectId,
    NodeId,
    Label,
    LayerId,
    Properties,
}

#[derive(Iden)]
enum Edges {
    Table,
    Id,
    ProjectId,
    SourceNodeId,
    TargetNodeId,
    Properties,
}

#[derive(Iden)]
enum Layers {
    Table,
    Id,
    ProjectId,
    LayerId,
    Name,
    Color,
    Properties,
}