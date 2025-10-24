use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Add comment columns to graph_nodes, graph_edges
        manager
            .alter_table(
                Table::alter()
                    .table(GraphNodes::Table)
                    .add_column(ColumnDef::new(GraphNodes::Comment).text())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GraphEdges::Table)
                    .add_column(ColumnDef::new(GraphEdges::Comment).text())
                    .to_owned(),
            )
            .await?;

        // 2. Rename layers table to graph_layers
        manager
            .rename_table(
                Table::rename()
                    .table(Layers::Table, GraphLayers::Table)
                    .to_owned(),
            )
            .await?;

        // 3. Add color columns and comment to graph_layers (one at a time for SQLite)
        manager
            .alter_table(
                Table::alter()
                    .table(GraphLayers::Table)
                    .add_column(ColumnDef::new(GraphLayers::BackgroundColor).string())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GraphLayers::Table)
                    .add_column(ColumnDef::new(GraphLayers::TextColor).string())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GraphLayers::Table)
                    .add_column(ColumnDef::new(GraphLayers::BorderColor).string())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GraphLayers::Table)
                    .add_column(ColumnDef::new(GraphLayers::Comment).text())
                    .to_owned(),
            )
            .await?;

        // 4. Drop old color column from graph_layers
        manager
            .alter_table(
                Table::alter()
                    .table(GraphLayers::Table)
                    .drop_column(GraphLayers::Color)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Reverse: Add color column back
        manager
            .alter_table(
                Table::alter()
                    .table(GraphLayers::Table)
                    .add_column(ColumnDef::new(GraphLayers::Color).string())
                    .to_owned(),
            )
            .await?;

        // Drop new color columns and comment from graph_layers (one at a time for SQLite)
        manager
            .alter_table(
                Table::alter()
                    .table(GraphLayers::Table)
                    .drop_column(GraphLayers::Comment)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GraphLayers::Table)
                    .drop_column(GraphLayers::BorderColor)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GraphLayers::Table)
                    .drop_column(GraphLayers::TextColor)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GraphLayers::Table)
                    .drop_column(GraphLayers::BackgroundColor)
                    .to_owned(),
            )
            .await?;

        // Rename graph_layers back to layers
        manager
            .rename_table(
                Table::rename()
                    .table(GraphLayers::Table, Layers::Table)
                    .to_owned(),
            )
            .await?;

        // Drop comment columns from graph_edges and graph_nodes
        manager
            .alter_table(
                Table::alter()
                    .table(GraphEdges::Table)
                    .drop_column(GraphEdges::Comment)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GraphNodes::Table)
                    .drop_column(GraphNodes::Comment)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum GraphNodes {
    Table,
    Comment,
}

#[derive(DeriveIden)]
enum GraphEdges {
    Table,
    Comment,
}

#[derive(DeriveIden)]
enum Layers {
    Table,
}

#[derive(DeriveIden)]
enum GraphLayers {
    Table,
    Color,
    BackgroundColor,
    TextColor,
    BorderColor,
    Comment,
}
