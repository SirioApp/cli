use crate::chain::client::{provider, signer_client};
use crate::chain::contracts::ContractAllowlistContract;
use crate::cli::{AllowlistCommand, Cli};
use crate::config::RuntimeConfig;
use crate::output::print_receipt;
use crate::util::send_and_wait;
use anyhow::Result;
use std::sync::Arc;

pub async fn run_allowlist(cli: &Cli, cfg: &RuntimeConfig, cmd: &AllowlistCommand) -> Result<()> {
    let read_client = Arc::new(provider(cfg)?);

    match cmd {
        AllowlistCommand::Info => {
            let allowlist = ContractAllowlistContract::new(cfg.allowlist, read_client);
            println!("allowlist: {:?}", cfg.allowlist);
            println!("admin: {:?}", allowlist.admin().call().await?);
        }
        AllowlistCommand::IsAllowed { target } => {
            let allowlist = ContractAllowlistContract::new(cfg.allowlist, read_client);
            println!("target: {:?}", target);
            println!("allowed: {}", allowlist.is_allowed(*target).call().await?);
        }
        AllowlistCommand::Add { target } => {
            let write_client = signer_client(cli, cfg)?;
            let call =
                ContractAllowlistContract::new(cfg.allowlist, write_client).add_contract(*target);
            print_receipt(&send_and_wait(call).await?);
        }
        AllowlistCommand::Remove { target } => {
            let write_client = signer_client(cli, cfg)?;
            let call = ContractAllowlistContract::new(cfg.allowlist, write_client)
                .remove_contract(*target);
            print_receipt(&send_and_wait(call).await?);
        }
        AllowlistCommand::TransferAdmin { new_admin } => {
            let write_client = signer_client(cli, cfg)?;
            let call = ContractAllowlistContract::new(cfg.allowlist, write_client)
                .transfer_admin(*new_admin);
            print_receipt(&send_and_wait(call).await?);
        }
    }

    Ok(())
}
