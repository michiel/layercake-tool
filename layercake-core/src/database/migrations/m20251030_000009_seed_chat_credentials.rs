use sea_orm::{ConnectionTrait, Statement, Value};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const PROVIDERS: &[&str] = &["ollama", "openai", "gemini", "claude"];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        for provider in PROVIDERS {
            let stmt = Statement::from_sql_and_values(
                db.get_database_backend(),
                "INSERT INTO chat_credentials (provider, created_at, updated_at) VALUES (?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) ON CONFLICT(provider) DO NOTHING",
                vec![Value::from(*provider)],
            );
            db.execute(stmt).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        for provider in PROVIDERS {
            let stmt = Statement::from_sql_and_values(
                db.get_database_backend(),
                "DELETE FROM chat_credentials WHERE provider = ?",
                vec![Value::from(*provider)],
            );
            db.execute(stmt).await?;
        }
        Ok(())
    }
}
