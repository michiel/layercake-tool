// src/db/migrate.rs
use sea_orm_migration::prelude::*;
use sea_orm::{DatabaseConnection, Statement};
use anyhow::Result;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240320_000001_create_tables::Migration),
        ]
    }
}

mod m20240320_000001_create_tables {
    use sea_orm_migration::prelude::*;
    use sea_orm::Statement;

    pub struct Migration;

    impl MigrationName for Migration {
        fn name(&self) -> &str {
            "m20240320_000001_create_tables"
        }
    }

    #[async_trait::async_trait]
    impl MigrationTrait for Migration {
        async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            // Create tables
            let stmts = vec![
                // Projects table
                Statement::from_string(
                    manager.get_database_backend(),
                    r#"
                    CREATE TABLE IF NOT EXISTS project (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        name TEXT NOT NULL,
                        description TEXT,
                        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
                    )
                    "#.to_string(),
                ),
                
                // Plans table
                Statement::from_string(
                    manager.get_database_backend(),
                    r#"
                    CREATE TABLE IF NOT EXISTS plan (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        project_id INTEGER NOT NULL,
                        plan_data TEXT NOT NULL,
                        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        FOREIGN KEY (project_id) REFERENCES project(id)
                    )
                    "#.to_string(),
                ),
                
                // Graphs table
                Statement::from_string(
                    manager.get_database_backend(),
                    r#"
                    CREATE TABLE IF NOT EXISTS graph (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        project_id INTEGER NOT NULL,
                        graph_data TEXT NOT NULL,
                        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                        FOREIGN KEY (project_id) REFERENCES project(id)
                    )
                    "#.to_string(),
                ),
                
                // Add triggers for updated_at timestamps
                Statement::from_string(
                    manager.get_database_backend(),
                    r#"
                    CREATE TRIGGER IF NOT EXISTS update_project_timestamp 
                    AFTER UPDATE ON project
                    BEGIN
                        UPDATE project SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
                    END;
                    "#.to_string(),
                ),
                
                Statement::from_string(
                    manager.get_database_backend(),
                    r#"
                    CREATE TRIGGER IF NOT EXISTS update_plan_timestamp 
                    AFTER UPDATE ON plan
                    BEGIN
                        UPDATE plan SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
                    END;
                    "#.to_string(),
                ),
                
                Statement::from_string(
                    manager.get_database_backend(),
                    r#"
                    CREATE TRIGGER IF NOT EXISTS update_graph_timestamp 
                    AFTER UPDATE ON graph
                    BEGIN
                        UPDATE graph SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
                    END;
                    "#.to_string(),
                ),
            ];
            
            // Execute each statement
            for stmt in stmts {
                manager.get_connection().execute(stmt).await?;
            }
            
            Ok(())
        }

        async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            // Drop tables in reverse order
            let stmts = vec![
                Statement::from_string(
                    manager.get_database_backend(),
                    "DROP TABLE IF EXISTS graph".to_string(),
                ),
                Statement::from_string(
                    manager.get_database_backend(),
                    "DROP TABLE IF EXISTS plan".to_string(),
                ),
                Statement::from_string(
                    manager.get_database_backend(),
                    "DROP TABLE IF EXISTS project".to_string(),
                ),
            ];
            
            for stmt in stmts {
                manager.get_connection().execute(stmt).await?;
            }
            
            Ok(())
        }
    }
}

// Function to run migrations manually
pub async fn run_migrations(db: &DatabaseConnection) -> Result<()> {
    Migrator::up(db, None).await?;
    Ok(())
}
