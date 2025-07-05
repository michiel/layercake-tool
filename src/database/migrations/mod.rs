use sea_orm_migration::prelude::*;

mod m001_create_tables;
mod m002_yaml_to_json_plan;
mod m003_plan_execution_tracking;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m001_create_tables::Migration),
            Box::new(m002_yaml_to_json_plan::Migration),
            Box::new(m003_plan_execution_tracking::Migration),
        ]
    }
}