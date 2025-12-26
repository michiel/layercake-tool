#![cfg(feature = "console")]

use anyhow::{anyhow, Result};
use chrono::Utc;
use clap::{Args, Subcommand};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};

use layercake_server::chat::ChatProvider;
use layercake_core::database::{
    connection::{establish_connection, get_database_url},
    entities::chat_credentials,
    migrations::Migrator,
};
use sea_orm_migration::MigratorTrait;

#[derive(Debug, Args)]
pub struct ChatCredentialOptions {
    /// Path to the sqlite database. Defaults to layercake.db
    #[clap(long)]
    pub database: Option<String>,

    #[clap(subcommand)]
    pub command: ChatCredentialCommand,
}

#[derive(Debug, Subcommand)]
pub enum ChatCredentialCommand {
    /// List stored provider credentials
    List,
    /// Set (or update) credentials for a provider
    Set {
        provider: ChatProvider,
        #[clap(long)]
        api_key: Option<String>,
        #[clap(long)]
        base_url: Option<String>,
    },
    /// Clear credentials for a provider
    Clear { provider: ChatProvider },
}

pub async fn run(options: ChatCredentialOptions) -> Result<()> {
    let database_url = get_database_url(options.database.as_deref());
    let db = establish_connection(&database_url).await?;

    // Ensure migrations applied before touching the table
    Migrator::up(&db, None).await?;

    match options.command {
        ChatCredentialCommand::List => list_credentials(&db).await?,
        ChatCredentialCommand::Set {
            provider,
            api_key,
            base_url,
        } => set_credentials(&db, provider, api_key, base_url).await?,
        ChatCredentialCommand::Clear { provider } => clear_credentials(&db, provider).await?,
    }

    Ok(())
}

async fn list_credentials(db: &DatabaseConnection) -> Result<()> {
    let entries = chat_credentials::Entity::find()
        .order_by_asc(chat_credentials::Column::Provider)
        .all(db)
        .await?;

    if entries.is_empty() {
        println!("No chat credentials found. Use `chat-credentials set` to add one.");
        return Ok(());
    }

    println!("provider\tapi_key\tbase_url");
    for entry in entries {
        let key = entry.api_key.as_deref().map(|_| "********").unwrap_or("-");
        let url = entry.base_url.as_deref().unwrap_or("-");
        println!("{}\t{}\t{}", entry.provider, key, url);
    }

    Ok(())
}

async fn set_credentials(
    db: &DatabaseConnection,
    provider: ChatProvider,
    api_key: Option<String>,
    base_url: Option<String>,
) -> Result<()> {
    let provider_key = provider.to_string();

    let existing = chat_credentials::Entity::find()
        .filter(chat_credentials::Column::Provider.eq(provider_key.clone()))
        .one(db)
        .await?;

    match existing {
        Some(model) => {
            let mut active: chat_credentials::ActiveModel = model.into();
            active.api_key = Set(api_key);
            active.base_url = Set(base_url);
            active.updated_at = Set(Utc::now());
            active.update(db).await?;
        }
        None => {
            let active = chat_credentials::ActiveModel {
                provider: Set(provider_key),
                api_key: Set(api_key),
                base_url: Set(base_url),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
                ..Default::default()
            };
            active.insert(db).await?;
        }
    }

    println!("Updated credentials for {}.", provider.display_name());
    Ok(())
}

async fn clear_credentials(db: &DatabaseConnection, provider: ChatProvider) -> Result<()> {
    let provider_key = provider.to_string();

    let existing = chat_credentials::Entity::find()
        .filter(chat_credentials::Column::Provider.eq(provider_key.clone()))
        .one(db)
        .await?;

    if let Some(model) = existing {
        let mut active: chat_credentials::ActiveModel = model.into();
        active.api_key = Set(None);
        active.base_url = Set(None);
        active.updated_at = Set(Utc::now());
        active.update(db).await?;
        println!("Cleared credentials for {}.", provider.display_name());
        Ok(())
    } else {
        Err(anyhow!(
            "No credentials stored for {}",
            provider.display_name()
        ))
    }
}
