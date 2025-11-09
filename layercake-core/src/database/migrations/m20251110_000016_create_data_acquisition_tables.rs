use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_tags_table(manager).await?;
        create_files_table(manager).await?;
        create_file_tags_table(manager).await?;
        create_dataset_tags_table(manager).await?;
        create_graph_node_tags_table(manager).await?;
        create_graph_edge_tags_table(manager).await?;
        create_kb_documents_table(manager).await?;
        create_vector_index_state_table(manager).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(VectorIndexState::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(KbDocuments::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(GraphEdgeTags::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(GraphNodeTags::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(DatasetTags::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(FileTags::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Files::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Tags::Table).to_owned())
            .await?;
        Ok(())
    }
}

async fn create_tags_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Tags::Table)
                .if_not_exists()
                .col(ColumnDef::new(Tags::Id).uuid().not_null().primary_key())
                .col(
                    ColumnDef::new(Tags::Name)
                        .string()
                        .not_null()
                        .extra("COLLATE NOCASE"),
                )
                .col(ColumnDef::new(Tags::Scope).string().not_null())
                .col(ColumnDef::new(Tags::Color).string())
                .col(ColumnDef::new(Tags::CreatedAt).timestamp().not_null())
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_tags_name_scope_unique")
                .table(Tags::Table)
                .col(Tags::Name)
                .col(Tags::Scope)
                .unique()
                .to_owned(),
        )
        .await
}

async fn create_files_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Files::Table)
                .if_not_exists()
                .col(ColumnDef::new(Files::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(Files::ProjectId).integer().not_null())
                .col(ColumnDef::new(Files::Filename).string().not_null())
                .col(ColumnDef::new(Files::MediaType).string().not_null())
                .col(ColumnDef::new(Files::SizeBytes).big_integer().not_null())
                .col(ColumnDef::new(Files::Blob).binary().not_null())
                .col(ColumnDef::new(Files::Checksum).string().not_null())
                .col(ColumnDef::new(Files::CreatedBy).integer())
                .col(ColumnDef::new(Files::CreatedAt).timestamp().not_null())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_files_project")
                        .from(Files::Table, Files::ProjectId)
                        .to(Projects::Table, Projects::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_files_project_created")
                .table(Files::Table)
                .col(Files::ProjectId)
                .col(Files::CreatedAt)
                .to_owned(),
        )
        .await
}

async fn create_file_tags_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(FileTags::Table)
                .if_not_exists()
                .col(ColumnDef::new(FileTags::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(FileTags::FileId).uuid().not_null())
                .col(ColumnDef::new(FileTags::TagId).uuid().not_null())
                .col(ColumnDef::new(FileTags::CreatedAt).timestamp().not_null())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_file_tags_file")
                        .from(FileTags::Table, FileTags::FileId)
                        .to(Files::Table, Files::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_file_tags_tag")
                        .from(FileTags::Table, FileTags::TagId)
                        .to(Tags::Table, Tags::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_file_tags_file_tag_unique")
                .table(FileTags::Table)
                .col(FileTags::FileId)
                .col(FileTags::TagId)
                .unique()
                .to_owned(),
        )
        .await
}

async fn create_dataset_tags_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(DatasetTags::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(DatasetTags::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(DatasetTags::DatasetId).integer().not_null())
                .col(ColumnDef::new(DatasetTags::TagId).uuid().not_null())
                .col(
                    ColumnDef::new(DatasetTags::CreatedAt)
                        .timestamp()
                        .not_null(),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_dataset_tags_dataset")
                        .from(DatasetTags::Table, DatasetTags::DatasetId)
                        .to(DataSources::Table, DataSources::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_dataset_tags_tag")
                        .from(DatasetTags::Table, DatasetTags::TagId)
                        .to(Tags::Table, Tags::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_dataset_tags_dataset_tag_unique")
                .table(DatasetTags::Table)
                .col(DatasetTags::DatasetId)
                .col(DatasetTags::TagId)
                .unique()
                .to_owned(),
        )
        .await
}

async fn create_graph_node_tags_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(GraphNodeTags::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(GraphNodeTags::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(GraphNodeTags::GraphId).integer().not_null())
                .col(ColumnDef::new(GraphNodeTags::NodeId).string().not_null())
                .col(ColumnDef::new(GraphNodeTags::TagId).uuid().not_null())
                .col(
                    ColumnDef::new(GraphNodeTags::CreatedAt)
                        .timestamp()
                        .not_null(),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_graph_node_tags_tag")
                        .from(GraphNodeTags::Table, GraphNodeTags::TagId)
                        .to(Tags::Table, Tags::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_graph_node_tags_node_tag")
                .table(GraphNodeTags::Table)
                .col(GraphNodeTags::GraphId)
                .col(GraphNodeTags::NodeId)
                .col(GraphNodeTags::TagId)
                .unique()
                .to_owned(),
        )
        .await
}

async fn create_graph_edge_tags_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(GraphEdgeTags::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(GraphEdgeTags::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(GraphEdgeTags::GraphId).integer().not_null())
                .col(ColumnDef::new(GraphEdgeTags::EdgeId).string().not_null())
                .col(ColumnDef::new(GraphEdgeTags::TagId).uuid().not_null())
                .col(
                    ColumnDef::new(GraphEdgeTags::CreatedAt)
                        .timestamp()
                        .not_null(),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_graph_edge_tags_tag")
                        .from(GraphEdgeTags::Table, GraphEdgeTags::TagId)
                        .to(Tags::Table, Tags::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_graph_edge_tags_edge_tag")
                .table(GraphEdgeTags::Table)
                .col(GraphEdgeTags::GraphId)
                .col(GraphEdgeTags::EdgeId)
                .col(GraphEdgeTags::TagId)
                .unique()
                .to_owned(),
        )
        .await
}

async fn create_kb_documents_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(KbDocuments::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(KbDocuments::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(KbDocuments::ProjectId).integer().not_null())
                .col(ColumnDef::new(KbDocuments::FileId).uuid())
                .col(ColumnDef::new(KbDocuments::ChunkId).string().not_null())
                .col(ColumnDef::new(KbDocuments::MediaType).string().not_null())
                .col(ColumnDef::new(KbDocuments::ChunkText).text().not_null())
                .col(ColumnDef::new(KbDocuments::Metadata).json_binary())
                .col(ColumnDef::new(KbDocuments::EmbeddingModel).string())
                .col(ColumnDef::new(KbDocuments::Embedding).binary())
                .col(
                    ColumnDef::new(KbDocuments::CreatedAt)
                        .timestamp()
                        .not_null(),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_kb_documents_project")
                        .from(KbDocuments::Table, KbDocuments::ProjectId)
                        .to(Projects::Table, Projects::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_kb_documents_file")
                        .from(KbDocuments::Table, KbDocuments::FileId)
                        .to(Files::Table, Files::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_kb_documents_project_chunk")
                .table(KbDocuments::Table)
                .col(KbDocuments::ProjectId)
                .col(KbDocuments::ChunkId)
                .to_owned(),
        )
        .await
}

async fn create_vector_index_state_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(VectorIndexState::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(VectorIndexState::Id)
                        .uuid()
                        .not_null()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(VectorIndexState::ProjectId)
                        .integer()
                        .not_null(),
                )
                .col(ColumnDef::new(VectorIndexState::Status).string().not_null())
                .col(ColumnDef::new(VectorIndexState::LastIndexedAt).timestamp_with_time_zone())
                .col(ColumnDef::new(VectorIndexState::LastError).text())
                .col(ColumnDef::new(VectorIndexState::Config).json_binary())
                .col(
                    ColumnDef::new(VectorIndexState::UpdatedAt)
                        .timestamp_with_time_zone()
                        .not_null(),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_vector_index_state_project")
                        .from(VectorIndexState::Table, VectorIndexState::ProjectId)
                        .to(Projects::Table, Projects::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .name("idx_vector_index_state_project_unique")
                .table(VectorIndexState::Table)
                .col(VectorIndexState::ProjectId)
                .unique()
                .to_owned(),
        )
        .await
}

#[derive(DeriveIden)]
enum Tags {
    Table,
    Id,
    Name,
    Scope,
    Color,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Files {
    Table,
    Id,
    ProjectId,
    Filename,
    MediaType,
    SizeBytes,
    Blob,
    Checksum,
    CreatedBy,
    CreatedAt,
}

#[derive(DeriveIden)]
enum FileTags {
    Table,
    Id,
    FileId,
    TagId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum DatasetTags {
    Table,
    Id,
    DatasetId,
    TagId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum GraphNodeTags {
    Table,
    Id,
    GraphId,
    NodeId,
    TagId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum GraphEdgeTags {
    Table,
    Id,
    GraphId,
    EdgeId,
    TagId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum KbDocuments {
    Table,
    Id,
    ProjectId,
    FileId,
    ChunkId,
    MediaType,
    ChunkText,
    Metadata,
    EmbeddingModel,
    Embedding,
    CreatedAt,
}

#[derive(DeriveIden)]
enum VectorIndexState {
    Table,
    Id,
    ProjectId,
    Status,
    LastIndexedAt,
    LastError,
    Config,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum DataSources {
    Table,
    Id,
}
