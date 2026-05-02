mod cli;
mod config;
mod deploy;
mod monitor;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "godo_vps=info".into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Bootstrap(args)       => deploy::bootstrap::run(args).await,
        Commands::GenServerConfig(args) => config::singbox::run(args).await,
        Commands::GenClientConfig(args) => config::clash::run(args).await,
        Commands::Sub(args)             => config::sub::run(args).await,
        Commands::Status(args)          => monitor::status::run(args).await,
        Commands::Monitor(args)         => monitor::daemon::run(args).await,
    }
}
