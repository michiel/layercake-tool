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
                    .col(
                        ColumnDef::new(Plans::ProjectId)
                            .integer()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Plans::Name).string().not_null())
                    .col(ColumnDef::new(Plans::YamlContent).text().not_null())
                    .col(ColumnDef::new(Plans::Dependencies).string())
                    .col(
                        ColumnDef::new(Plans::Status)
                            .string()
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        ColumnDef::new(Plans::Version)
                            .integer()
                            .not_null()
                            .default(1),
                    )
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

        // Create plan_dag_nodes table
        manager
            .create_table(
                Table::create()
                    .table(PlanDagNodes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PlanDagNodes::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PlanDagNodes::PlanId).integer().not_null())
                    .col(ColumnDef::new(PlanDagNodes::NodeType).string().not_null())
                    .col(ColumnDef::new(PlanDagNodes::PositionX).double().not_null())
                    .col(ColumnDef::new(PlanDagNodes::PositionY).double().not_null())
                    .col(ColumnDef::new(PlanDagNodes::MetadataJson).text().not_null())
                    .col(ColumnDef::new(PlanDagNodes::ConfigJson).text().not_null())
                    .col(ColumnDef::new(PlanDagNodes::CreatedAt).text().not_null())
                    .col(ColumnDef::new(PlanDagNodes::UpdatedAt).text().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_plan_dag_nodes_plan_id")
                            .from(PlanDagNodes::Table, PlanDagNodes::PlanId)
                            .to(Plans::Table, Plans::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create plan_dag_edges table
        manager
            .create_table(
                Table::create()
                    .table(PlanDagEdges::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PlanDagEdges::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PlanDagEdges::PlanId).integer().not_null())
                    .col(
                        ColumnDef::new(PlanDagEdges::SourceNodeId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PlanDagEdges::TargetNodeId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PlanDagEdges::SourceHandle).string())
                    .col(ColumnDef::new(PlanDagEdges::TargetHandle).string())
                    .col(ColumnDef::new(PlanDagEdges::MetadataJson).text().not_null())
                    .col(ColumnDef::new(PlanDagEdges::CreatedAt).text().not_null())
                    .col(ColumnDef::new(PlanDagEdges::UpdatedAt).text().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_plan_dag_edges_plan_id")
                            .from(PlanDagEdges::Table, PlanDagEdges::PlanId)
                            .to(Plans::Table, Plans::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create users table
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Users::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Users::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::DisplayName).string().not_null())
                    .col(ColumnDef::new(Users::PasswordHash).string().not_null())
                    .col(ColumnDef::new(Users::AvatarColor).string().not_null())
                    .col(
                        ColumnDef::new(Users::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Users::LastLoginAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create user_sessions table
        manager
            .create_table(
                Table::create()
                    .table(UserSessions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserSessions::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UserSessions::SessionId)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(UserSessions::UserId).integer().not_null())
                    .col(ColumnDef::new(UserSessions::UserName).string().not_null())
                    .col(ColumnDef::new(UserSessions::ProjectId).integer().not_null())
                    .col(
                        ColumnDef::new(UserSessions::LayercakeGraphId)
                            .integer()
                            .null(),
                    )
                    .col(ColumnDef::new(UserSessions::CursorPosition).string().null())
                    .col(ColumnDef::new(UserSessions::SelectedNodeId).string().null())
                    .col(
                        ColumnDef::new(UserSessions::LastActivity)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserSessions::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(UserSessions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserSessions::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_sessions_user_id")
                            .from(UserSessions::Table, UserSessions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_sessions_project_id")
                            .from(UserSessions::Table, UserSessions::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create project_collaborators table
        manager
            .create_table(
                Table::create()
                    .table(ProjectCollaborators::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProjectCollaborators::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ProjectCollaborators::ProjectId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectCollaborators::UserId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectCollaborators::Role)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectCollaborators::Permissions)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectCollaborators::InvitedBy)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ProjectCollaborators::InvitationStatus)
                            .string()
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        ColumnDef::new(ProjectCollaborators::InvitedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectCollaborators::JoinedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ProjectCollaborators::LastActiveAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ProjectCollaborators::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(ProjectCollaborators::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectCollaborators::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_collaborators_project_id")
                            .from(ProjectCollaborators::Table, ProjectCollaborators::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_collaborators_user_id")
                            .from(ProjectCollaborators::Table, ProjectCollaborators::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_project_collaborators_invited_by")
                            .from(ProjectCollaborators::Table, ProjectCollaborators::InvitedBy)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .index(
                        Index::create()
                            .name("idx_project_collaborators_project_user")
                            .col(ProjectCollaborators::ProjectId)
                            .col(ProjectCollaborators::UserId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create user_presence table
        manager
            .create_table(
                Table::create()
                    .table(UserPresence::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserPresence::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserPresence::UserId).integer().not_null())
                    .col(ColumnDef::new(UserPresence::ProjectId).integer().not_null())
                    .col(ColumnDef::new(UserPresence::SessionId).string().not_null())
                    .col(
                        ColumnDef::new(UserPresence::LayercakeGraphId)
                            .integer()
                            .null(),
                    )
                    .col(ColumnDef::new(UserPresence::CursorPosition).string().null())
                    .col(ColumnDef::new(UserPresence::SelectedNodeId).string().null())
                    .col(
                        ColumnDef::new(UserPresence::ViewportPosition)
                            .string()
                            .null(),
                    )
                    .col(ColumnDef::new(UserPresence::CurrentTool).string().null())
                    .col(
                        ColumnDef::new(UserPresence::IsOnline)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(UserPresence::LastSeen)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserPresence::LastHeartbeat)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserPresence::Status)
                            .string()
                            .not_null()
                            .default("active"),
                    )
                    .col(
                        ColumnDef::new(UserPresence::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserPresence::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_presence_user_id")
                            .from(UserPresence::Table, UserPresence::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_presence_project_id")
                            .from(UserPresence::Table, UserPresence::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_presence_session_id")
                            .from(UserPresence::Table, UserPresence::SessionId)
                            .to(UserSessions::Table, UserSessions::SessionId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_user_presence_project_user")
                            .col(UserPresence::ProjectId)
                            .col(UserPresence::UserId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

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
                    .col(
                        ColumnDef::new(DataSources::FileSize)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(DataSources::ProcessedAt).timestamp())
                    .col(
                        ColumnDef::new(DataSources::FileFormat)
                            .string()
                            .not_null()
                            .default("csv"),
                    )
                    .col(
                        ColumnDef::new(DataSources::DataType)
                            .string()
                            .not_null()
                            .default("nodes"),
                    )
                    .col(
                        ColumnDef::new(DataSources::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DataSources::UpdatedAt)
                            .timestamp()
                            .not_null(),
                    )
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

        // Create datasources table (for DAG DataSourceNode entities)
        manager
            .create_table(
                Table::create()
                    .table(Datasources::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Datasources::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Datasources::ProjectId).integer().not_null())
                    .col(ColumnDef::new(Datasources::NodeId).string().not_null())
                    .col(ColumnDef::new(Datasources::Name).string().not_null())
                    .col(ColumnDef::new(Datasources::FilePath).string().not_null())
                    .col(ColumnDef::new(Datasources::FileType).string().not_null()) // 'nodes' or 'edges'
                    .col(ColumnDef::new(Datasources::ImportDate).timestamp())
                    .col(ColumnDef::new(Datasources::RowCount).integer())
                    .col(ColumnDef::new(Datasources::ColumnInfo).json_binary()) // Schema: [{name, type, nullable}, ...]
                    .col(
                        ColumnDef::new(Datasources::ExecutionState)
                            .string()
                            .not_null()
                            .default("not_started"),
                    )
                    .col(ColumnDef::new(Datasources::ErrorMessage).text())
                    .col(ColumnDef::new(Datasources::Metadata).json_binary())
                    .col(
                        ColumnDef::new(Datasources::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Datasources::UpdatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_datasources_project_id")
                            .from(Datasources::Table, Datasources::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_datasources_node_id")
                            .from(Datasources::Table, Datasources::NodeId)
                            .to(PlanDagNodes::Table, PlanDagNodes::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create datasource_rows table
        manager
            .create_table(
                Table::create()
                    .table(DatasourceRows::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DatasourceRows::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(DatasourceRows::DatasourceId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DatasourceRows::RowNumber)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DatasourceRows::Data)
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DatasourceRows::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_datasource_rows_datasource_id")
                            .from(DatasourceRows::Table, DatasourceRows::DatasourceId)
                            .to(Datasources::Table, Datasources::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create graphs table
        manager
            .create_table(
                Table::create()
                    .table(Graphs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Graphs::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Graphs::ProjectId).integer().not_null())
                    .col(ColumnDef::new(Graphs::NodeId).string().not_null())
                    .col(ColumnDef::new(Graphs::Name).string().not_null())
                    .col(
                        ColumnDef::new(Graphs::ExecutionState)
                            .string()
                            .not_null()
                            .default("not_started"),
                    )
                    .col(ColumnDef::new(Graphs::ComputedDate).timestamp())
                    .col(ColumnDef::new(Graphs::SourceHash).string()) // Hash of upstream data
                    .col(
                        ColumnDef::new(Graphs::NodeCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Graphs::EdgeCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(Graphs::ErrorMessage).text())
                    .col(ColumnDef::new(Graphs::Metadata).json_binary())
                    .col(ColumnDef::new(Graphs::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Graphs::UpdatedAt).timestamp().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_graphs_project_id")
                            .from(Graphs::Table, Graphs::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_graphs_node_id")
                            .from(Graphs::Table, Graphs::NodeId)
                            .to(PlanDagNodes::Table, PlanDagNodes::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create graph_nodes table
        manager
            .create_table(
                Table::create()
                    .table(GraphNodes::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GraphNodes::Id).string().not_null())
                    .col(ColumnDef::new(GraphNodes::GraphId).integer().not_null())
                    .col(ColumnDef::new(GraphNodes::Label).string())
                    .col(ColumnDef::new(GraphNodes::Layer).string())
                    .col(ColumnDef::new(GraphNodes::Weight).double())
                    .col(
                        ColumnDef::new(GraphNodes::IsPartition)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(GraphNodes::Attrs).json_binary())
                    .col(ColumnDef::new(GraphNodes::CreatedAt).timestamp().not_null())
                    .primary_key(
                        Index::create()
                            .name("pk_graph_nodes")
                            .col(GraphNodes::GraphId)
                            .col(GraphNodes::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_graph_nodes_graph_id")
                            .from(GraphNodes::Table, GraphNodes::GraphId)
                            .to(Graphs::Table, Graphs::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create graph_edges table
        manager
            .create_table(
                Table::create()
                    .table(GraphEdges::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GraphEdges::Id).string().not_null())
                    .col(ColumnDef::new(GraphEdges::GraphId).integer().not_null())
                    .col(ColumnDef::new(GraphEdges::Source).string().not_null())
                    .col(ColumnDef::new(GraphEdges::Target).string().not_null())
                    .col(ColumnDef::new(GraphEdges::Label).string())
                    .col(ColumnDef::new(GraphEdges::Layer).string())
                    .col(ColumnDef::new(GraphEdges::Weight).double())
                    .col(ColumnDef::new(GraphEdges::Attrs).json_binary())
                    .col(ColumnDef::new(GraphEdges::CreatedAt).timestamp().not_null())
                    .primary_key(
                        Index::create()
                            .name("pk_graph_edges")
                            .col(GraphEdges::GraphId)
                            .col(GraphEdges::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_graph_edges_graph_id")
                            .from(GraphEdges::Table, GraphEdges::GraphId)
                            .to(Graphs::Table, Graphs::Id)
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
                    .col(ColumnDef::new(Layers::GraphId).integer().not_null())
                    .col(ColumnDef::new(Layers::LayerId).string().not_null())
                    .col(ColumnDef::new(Layers::Name).string().not_null())
                    .col(ColumnDef::new(Layers::Color).string())
                    .col(ColumnDef::new(Layers::Properties).text())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_layers_graph_id")
                            .from(Layers::Table, Layers::GraphId)
                            .to(Graphs::Table, Graphs::Id)
                            .on_delete(ForeignKeyAction::Cascade),
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
            .drop_table(Table::drop().table(GraphEdges::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(GraphNodes::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Graphs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(DatasourceRows::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Datasources::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(DataSources::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserPresence::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ProjectCollaborators::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserSessions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PlanDagEdges::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PlanDagNodes::Table).to_owned())
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
    Version,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum PlanDagNodes {
    Table,
    Id,
    PlanId,
    NodeType,
    PositionX,
    PositionY,
    MetadataJson,
    ConfigJson,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum PlanDagEdges {
    Table,
    Id,
    PlanId,
    SourceNodeId,
    TargetNodeId,
    SourceHandle,
    TargetHandle,
    MetadataJson,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
    Email,
    Username,
    DisplayName,
    PasswordHash,
    AvatarColor,
    IsActive,
    CreatedAt,
    UpdatedAt,
    LastLoginAt,
}

#[derive(Iden)]
enum UserSessions {
    Table,
    Id,
    SessionId,
    UserId,
    UserName,
    ProjectId,
    LayercakeGraphId,
    CursorPosition,
    SelectedNodeId,
    LastActivity,
    IsActive,
    CreatedAt,
    ExpiresAt,
}

#[derive(Iden)]
enum ProjectCollaborators {
    Table,
    Id,
    ProjectId,
    UserId,
    Role,
    Permissions,
    InvitedBy,
    InvitationStatus,
    InvitedAt,
    JoinedAt,
    LastActiveAt,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum UserPresence {
    Table,
    Id,
    UserId,
    ProjectId,
    SessionId,
    LayercakeGraphId,
    CursorPosition,
    SelectedNodeId,
    ViewportPosition,
    CurrentTool,
    IsOnline,
    LastSeen,
    LastHeartbeat,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum DataSources {
    Table,
    Id,
    ProjectId,
    Name,
    Description,
    Filename,
    Blob,
    GraphJson,
    Status,
    ErrorMessage,
    FileSize,
    ProcessedAt,
    FileFormat,
    DataType,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Datasources {
    Table,
    Id,
    ProjectId,
    NodeId,
    Name,
    FilePath,
    FileType,
    ImportDate,
    RowCount,
    ColumnInfo,
    ExecutionState,
    ErrorMessage,
    Metadata,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum DatasourceRows {
    Table,
    Id,
    DatasourceId,
    RowNumber,
    Data,
    CreatedAt,
}

#[derive(Iden)]
enum Graphs {
    Table,
    Id,
    ProjectId,
    NodeId,
    Name,
    ExecutionState,
    ComputedDate,
    SourceHash,
    NodeCount,
    EdgeCount,
    ErrorMessage,
    Metadata,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum GraphNodes {
    Table,
    Id,
    GraphId,
    Label,
    Layer,
    Weight,
    IsPartition,
    Attrs,
    CreatedAt,
}

#[derive(Iden)]
enum GraphEdges {
    Table,
    Id,
    GraphId,
    Source,
    Target,
    Label,
    Layer,
    Weight,
    Attrs,
    CreatedAt,
}

#[derive(Iden)]
enum Layers {
    Table,
    Id,
    GraphId,
    LayerId,
    Name,
    Color,
    Properties,
}
