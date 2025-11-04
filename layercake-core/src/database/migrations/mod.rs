pub use sea_orm_migration::prelude::*;

mod m20251008_000000_create_initial_schema;
mod m20251009_000001_add_belongs_to_to_graph_nodes;
mod m20251010_000002_create_graph_edits;
mod m20251011_000003_add_node_handle_positions;
mod m20251018_000004_remove_edge_handles;
mod m20251021_000005_create_library_sources;
mod m20251024_000006_add_datasource_id_to_graph_data;
mod m20251024_000007_normalize_graph_schema;
mod m20251030_000008_create_chat_credentials;
mod m20251030_000009_seed_chat_credentials;
mod m20251103_000010_create_chat_sessions;
mod m20251103_000011_create_chat_messages;
mod m20251103_000012_extend_users_table;
mod m20251103_000013_extend_user_sessions_table;
mod m20251105_000014_create_system_settings;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251008_000000_create_initial_schema::Migration),
            Box::new(m20251009_000001_add_belongs_to_to_graph_nodes::Migration),
            Box::new(m20251010_000002_create_graph_edits::Migration),
            Box::new(m20251011_000003_add_node_handle_positions::Migration),
            Box::new(m20251018_000004_remove_edge_handles::Migration),
            Box::new(m20251021_000005_create_library_sources::Migration),
            Box::new(m20251024_000006_add_datasource_id_to_graph_data::Migration),
            Box::new(m20251024_000007_normalize_graph_schema::Migration),
            Box::new(m20251030_000008_create_chat_credentials::Migration),
            Box::new(m20251030_000009_seed_chat_credentials::Migration),
            Box::new(m20251103_000010_create_chat_sessions::Migration),
            Box::new(m20251103_000011_create_chat_messages::Migration),
            Box::new(m20251103_000012_extend_users_table::Migration),
            Box::new(m20251103_000013_extend_user_sessions_table::Migration),
            Box::new(m20251105_000014_create_system_settings::Migration),
        ]
    }
}
