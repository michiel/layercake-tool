use anyhow::Result;
use clap::Parser;
use layercake_server::server;
use tracing::info;
use tracing::Level;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[clap(author, version, about)]
struct ServerArgs {
    #[clap(short, long, global = true)]
    log_level: Option<String>,
    #[clap(short, long, default_value = "3000")]
    port: u16,
    #[clap(short, long, default_value = "layercake.db")]
    database: String,
    #[clap(long)]
    cors_origin: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = ServerArgs::parse();
    setup_logging(&args.log_level);

    info!("Starting server on port {}", args.port);
    server::start_server(args.port, &args.database, args.cors_origin.as_deref()).await?;

    Ok(())
}

fn setup_logging(log_level: &Option<String>) {
    let log_level = match log_level
        .as_ref()
        .unwrap_or(&"info".to_string())
        .to_lowercase()
        .as_str()
    {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(format!("handlebars=off,{}", log_level)))
        .without_time()
        .init();
}
