use sea_orm_migration::prelude::*;

mod m001_create_tables;
mod m002_plan_dag_tables;
mod m003_user_authentication;
mod m004_create_data_sources;
mod m005_add_plan_dag_version;
mod m007_remove_unused_plan_dag_json;
mod m008_add_edge_handles;
mod m009_refactor_data_source_types;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m001_create_tables::Migration),
            Box::new(m002_plan_dag_tables::Migration),
            Box::new(m003_user_authentication::Migration),
            Box::new(m004_create_data_sources::Migration),
            Box::new(m005_add_plan_dag_version::Migration),
            Box::new(m007_remove_unused_plan_dag_json::Migration),
            Box::new(m008_add_edge_handles::Migration),
            Box::new(m009_refactor_data_source_types::Migration),
        ]
    }
}