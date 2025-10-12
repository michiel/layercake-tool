use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create graph_edits table
        manager
            .create_table(
                Table::create()
                    .table(GraphEdits::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GraphEdits::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(GraphEdits::GraphId).integer().not_null())
                    .col(ColumnDef::new(GraphEdits::TargetType).string().not_null())
                    .col(ColumnDef::new(GraphEdits::TargetId).string().not_null())
                    .col(ColumnDef::new(GraphEdits::Operation).string().not_null())
                    .col(ColumnDef::new(GraphEdits::FieldName).string())
                    .col(ColumnDef::new(GraphEdits::OldValue).json())
                    .col(ColumnDef::new(GraphEdits::NewValue).json())
                    .col(ColumnDef::new(GraphEdits::SequenceNumber).integer().not_null())
                    .col(ColumnDef::new(GraphEdits::Applied).boolean().not_null().default(false))
                    .col(ColumnDef::new(GraphEdits::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(GraphEdits::CreatedBy).integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_graph_edits_graph_id")
                            .from(GraphEdits::Table, GraphEdits::GraphId)
                            .to(Graphs::Table, Graphs::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on graph_id
        manager
            .create_index(
                Index::create()
                    .name("idx_graph_edits_graph_id")
                    .table(GraphEdits::Table)
                    .col(GraphEdits::GraphId)
                    .to_owned(),
            )
            .await?;

        // Create index on graph_id + target_type + target_id
        manager
            .create_index(
                Index::create()
                    .name("idx_graph_edits_target")
                    .table(GraphEdits::Table)
                    .col(GraphEdits::GraphId)
                    .col(GraphEdits::TargetType)
                    .col(GraphEdits::TargetId)
                    .to_owned(),
            )
            .await?;

        // Create index on graph_id + sequence_number
        manager
            .create_index(
                Index::create()
                    .name("idx_graph_edits_sequence")
                    .table(GraphEdits::Table)
                    .col(GraphEdits::GraphId)
                    .col(GraphEdits::SequenceNumber)
                    .to_owned(),
            )
            .await?;

        // Create unique constraint on graph_id + sequence_number
        manager
            .create_index(
                Index::create()
                    .name("uq_graph_edits_graph_sequence")
                    .table(GraphEdits::Table)
                    .col(GraphEdits::GraphId)
                    .col(GraphEdits::SequenceNumber)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Add new columns to graphs table
        manager
            .alter_table(
                Table::alter()
                    .table(Graphs::Table)
                    .add_column(ColumnDef::new(Graphs::LastEditSequence).integer().not_null().default(0))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Graphs::Table)
                    .add_column(ColumnDef::new(Graphs::HasPendingEdits).boolean().not_null().default(false))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Graphs::Table)
                    .add_column(ColumnDef::new(Graphs::LastReplayAt).timestamp())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove columns from graphs table
        manager
            .alter_table(
                Table::alter()
                    .table(Graphs::Table)
                    .drop_column(Graphs::LastReplayAt)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Graphs::Table)
                    .drop_column(Graphs::HasPendingEdits)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Graphs::Table)
                    .drop_column(Graphs::LastEditSequence)
                    .to_owned(),
            )
            .await?;

        // Drop graph_edits table
        manager
            .drop_table(Table::drop().table(GraphEdits::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum GraphEdits {
    Table,
    Id,
    GraphId,
    TargetType,
    TargetId,
    Operation,
    FieldName,
    OldValue,
    NewValue,
    SequenceNumber,
    Applied,
    CreatedAt,
    CreatedBy,
}

#[derive(DeriveIden)]
enum Graphs {
    Table,
    Id,
    LastEditSequence,
    HasPendingEdits,
    LastReplayAt,
}
