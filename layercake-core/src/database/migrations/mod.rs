pub use sea_orm_migration::prelude::*;

mod m20251008_000000_create_initial_schema;
mod m20251009_000001_add_belongs_to_to_graph_nodes;
mod m20251010_000002_create_graph_edits;
mod m20251011_000003_add_node_handle_positions;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251008_000000_create_initial_schema::Migration),
            Box::new(m20251009_000001_add_belongs_to_to_graph_nodes::Migration),
            Box::new(m20251010_000002_create_graph_edits::Migration),
            Box::new(m20251011_000003_add_node_handle_positions::Migration),
        ]
    }
}
