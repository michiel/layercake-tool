pub use sea_orm_migration::prelude::*;

mod m20251008_000000_create_initial_schema;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251008_000000_create_initial_schema::Migration),
        ]
    }
}