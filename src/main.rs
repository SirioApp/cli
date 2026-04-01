mod chain;
mod cli;
mod commands;
mod config;
mod output;
mod types;
mod util;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};
use config::RuntimeConfig;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = RuntimeConfig::resolve(&cli)?;

    match &cli.command {
        Command::Network => commands::network::run_network(&cfg),
        Command::Factory { cmd } => commands::factory::run_factory(&cli, &cfg, cmd).await?,
        Command::Sale { cmd } => commands::sale::run_sale(&cli, &cfg, cmd).await?,
        Command::Allowlist { cmd } => commands::allowlist::run_allowlist(&cli, &cfg, cmd).await?,
    }

    Ok(())
}
