mod allowlist;
mod cli;
mod client;
mod config;
mod contracts;
mod factory;
mod formatters;
mod sale;
mod util;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};
use config::RuntimeConfig;

fn print_network(cfg: &RuntimeConfig) {
    println!("network: {:?}", cfg.network);
    println!("label: {}", cfg.network_label);
    println!("chain_id: {}", cfg.chain_id);
    println!("rpc: {}", cfg.rpc_url);
    println!("factory: {:?}", cfg.factory);
    println!("allowlist: {:?}", cfg.allowlist);
    println!(
        "default_collateral: {}",
        cfg.default_collateral
            .map(|a| format!("{:?}", a))
            .unwrap_or_else(|| "<none>".to_string())
    );
    println!("deployment_file: {}", cfg.deployment_path.display());
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = RuntimeConfig::resolve(&cli)?;

    match &cli.command {
        Command::Network => print_network(&cfg),
        Command::Factory { cmd } => factory::run_factory(&cli, &cfg, &cmd).await?,
        Command::Sale { cmd } => sale::run_sale(&cli, &cfg, &cmd).await?,
        Command::Allowlist { cmd } => allowlist::run_allowlist(&cli, &cfg, &cmd).await?,
    }

    Ok(())
}
