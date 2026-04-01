use crate::cli::{AllowlistCommand, Cli};
use crate::client::signer_client;
use crate::config::RuntimeConfig;
use crate::contracts::ContractAllowlistContract;
use crate::util::send_and_wait;
use anyhow::Result;
use std::sync::Arc;

pub async fn run_allowlist(cli: &Cli, cfg: &RuntimeConfig, cmd: &AllowlistCommand) -> Result<()> {
    let p = Arc::new(crate::client::provider(cfg)?);
    match cmd {
        AllowlistCommand::Info => {
            let allow = ContractAllowlistContract::new(cfg.allowlist, p);
            let admin = allow.admin().call().await?;
            println!("allowlist: {:?}", cfg.allowlist);
            println!("admin: {:?}", admin);
        }
        AllowlistCommand::IsAllowed { target } => {
            let allow = ContractAllowlistContract::new(cfg.allowlist, p);
            let ok = allow.is_allowed(*target).call().await?;
            println!("target: {:?}", target);
            println!("allowed: {}", ok);
        }
        AllowlistCommand::Add { target } => {
            let client = signer_client(cli, cfg)?;
            let allow = ContractAllowlistContract::new(cfg.allowlist, client);
            let call = allow.add_contract(*target);
            let receipt = send_and_wait(call).await?;
            crate::util::print_receipt(&receipt);
        }
        AllowlistCommand::Remove { target } => {
            let client = signer_client(cli, cfg)?;
            let allow = ContractAllowlistContract::new(cfg.allowlist, client);
            let call = allow.remove_contract(*target);
            let receipt = send_and_wait(call).await?;
            crate::util::print_receipt(&receipt);
        }
        AllowlistCommand::TransferAdmin { new_admin } => {
            let client = signer_client(cli, cfg)?;
            let allow = ContractAllowlistContract::new(cfg.allowlist, client);
            let call = allow.transfer_admin(*new_admin);
            let receipt = send_and_wait(call).await?;
            crate::util::print_receipt(&receipt);
        }
    }
    Ok(())
}
