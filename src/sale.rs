use crate::cli::{Cli, SaleCommand, SaleTarget};
use crate::client::signer_client;
use crate::config::RuntimeConfig;
use crate::contracts::{
    SaleContract, read_factory_project, read_sale_collateral, read_sale_project_id, token_meta,
};
use crate::formatters::format_token_amount;
use crate::util::{parse_amount_raw, parse_amount_units, send_and_wait};
use anyhow::{Result, bail};
use ethers::prelude::*;
use std::sync::Arc;

pub async fn run_sale(cli: &Cli, cfg: &RuntimeConfig, cmd: &SaleCommand) -> Result<()> {
    let p = Arc::new(crate::client::provider(cfg)?);
    match cmd {
        SaleCommand::Status(target) => {
            let sale_addr = resolve_sale_address(p.clone(), cfg.factory, target).await?;
            let sale = SaleContract::new(sale_addr, p.clone());
            let start = sale.start_time().call().await?;
            let end = sale.end_time().call().await?;
            let (total, accepted, finalized, failed) = sale.get_status().call().await?;
            let active = sale.is_active().call().await?;
            let remaining = sale.time_remaining().call().await?;
            let token = sale.token().call().await?;
            let collateral = read_sale_collateral(p.clone(), sale_addr).await?;
            let (decimals, symbol) = token_meta(p.clone(), collateral).await?;

            println!("sale: {:?}", sale_addr);
            if let Ok(project_id) = read_sale_project_id(p.clone(), sale_addr).await {
                println!("project_id: {}", project_id);
                if let Ok(approved) =
                    crate::contracts::AgentRaiseFactoryContract::new(cfg.factory, p.clone())
                        .is_project_approved(project_id)
                        .call()
                        .await
                {
                    println!("project_approved: {}", approved);
                }
            }
            println!("collateral: {:?} ({})", collateral, symbol);
            println!("start_time: {}", start);
            println!("end_time: {}", end);
            println!("is_active: {}", active);
            println!("time_remaining_seconds: {}", remaining);
            println!(
                "total_committed: {} ({})",
                format_token_amount(total, decimals),
                total
            );
            println!(
                "accepted_amount: {} ({})",
                format_token_amount(accepted, decimals),
                accepted
            );
            println!("finalized: {}", finalized);
            println!("failed: {}", failed);
            println!("share_token: {:?}", token);
        }
        SaleCommand::Claimable { target, user } => {
            let sale_addr = resolve_sale_address(p.clone(), cfg.factory, target).await?;
            let sale = SaleContract::new(sale_addr, p.clone());
            let collateral = read_sale_collateral(p.clone(), sale_addr).await?;
            let (decimals, symbol) = token_meta(p.clone(), collateral).await?;
            let (payout, refund) = sale.get_claimable(*user).call().await?;
            println!("sale: {:?}", sale_addr);
            println!("user: {:?}", user);
            println!("token: {}", symbol);
            println!(
                "payout: {} ({})",
                format_token_amount(payout, decimals),
                payout
            );
            println!(
                "refund: {} ({})",
                format_token_amount(refund, decimals),
                refund
            );
        }
        SaleCommand::Refundable { target, user } => {
            let sale_addr = resolve_sale_address(p.clone(), cfg.factory, target).await?;
            let sale = SaleContract::new(sale_addr, p.clone());
            let collateral = read_sale_collateral(p.clone(), sale_addr).await?;
            let (decimals, symbol) = token_meta(p.clone(), collateral).await?;
            let refundable = sale.get_refundable(*user).call().await?;
            println!("sale: {:?}", sale_addr);
            println!("user: {:?}", user);
            println!("token: {}", symbol);
            println!(
                "refundable: {} ({})",
                format_token_amount(refundable, decimals),
                refundable
            );
        }
        SaleCommand::Commitment { target, user } => {
            let sale_addr = resolve_sale_address(p.clone(), cfg.factory, target).await?;
            let sale = SaleContract::new(sale_addr, p.clone());
            let collateral = read_sale_collateral(p.clone(), sale_addr).await?;
            let (decimals, symbol) = token_meta(p.clone(), collateral).await?;
            let committed = sale.commitments(*user).call().await?;
            println!("sale: {:?}", sale_addr);
            println!("user: {:?}", user);
            println!("token: {}", symbol);
            println!(
                "committed: {} ({})",
                format_token_amount(committed, decimals),
                committed
            );
        }
        SaleCommand::ApproveCollateral {
            target,
            amount,
            raw,
        } => {
            let client = signer_client(cli, cfg)?;
            let owner = client.address();
            let sale_addr = resolve_sale_address(client.clone(), cfg.factory, target).await?;
            let collateral = read_sale_collateral(client.clone(), sale_addr).await?;
            let erc20 = crate::contracts::Erc20Metadata::new(collateral, client.clone());
            let (decimals, symbol) = token_meta(client.clone(), collateral).await?;

            let amount_wei = if *raw {
                parse_amount_raw(amount)?
            } else {
                parse_amount_units(amount, decimals)?
            };

            let call = erc20.approve(sale_addr, amount_wei);
            let receipt = send_and_wait(call).await?;
            println!("owner: {:?}", owner);
            println!(
                "approved: {} {} ({})",
                format_token_amount(amount_wei, decimals),
                symbol,
                amount_wei
            );
            crate::util::print_receipt(&receipt);
        }
        SaleCommand::Commit {
            target,
            amount,
            raw,
        } => {
            let client = signer_client(cli, cfg)?;
            let sender = client.address();
            let sale_addr = resolve_sale_address(client.clone(), cfg.factory, target).await?;
            let collateral = read_sale_collateral(client.clone(), sale_addr).await?;
            let sale = SaleContract::new(sale_addr, client.clone());
            let erc20 = crate::contracts::Erc20Metadata::new(collateral, client.clone());
            let (decimals, symbol) = token_meta(client.clone(), collateral).await?;

            let amount_wei = if *raw {
                parse_amount_raw(amount)?
            } else {
                parse_amount_units(amount, decimals)?
            };

            let allowance = erc20.allowance(sender, sale_addr).call().await?;
            if allowance < amount_wei {
                bail!(
                    "insufficient allowance: allowance={} required={}. Run `sale approve-collateral` first.",
                    format_token_amount(allowance, decimals),
                    format_token_amount(amount_wei, decimals)
                );
            }

            let balance = erc20.balance_of(sender).call().await?;
            if balance < amount_wei {
                bail!(
                    "insufficient balance: balance={} required={}",
                    format_token_amount(balance, decimals),
                    format_token_amount(amount_wei, decimals)
                );
            }

            let call = sale.commit(amount_wei);
            let receipt = send_and_wait(call).await?;
            println!("sender: {:?}", sender);
            println!(
                "committed: {} {} ({})",
                format_token_amount(amount_wei, decimals),
                symbol,
                amount_wei
            );
            crate::util::print_receipt(&receipt);
        }
        SaleCommand::Finalize(target) => {
            let client = signer_client(cli, cfg)?;
            let sale_addr = resolve_sale_address(client.clone(), cfg.factory, target).await?;
            let sale = SaleContract::new(sale_addr, client);
            let call = sale.finalize();
            let receipt = send_and_wait(call).await?;
            crate::util::print_receipt(&receipt);
        }
        SaleCommand::Claim(target) => {
            let client = signer_client(cli, cfg)?;
            let sale_addr = resolve_sale_address(client.clone(), cfg.factory, target).await?;
            let sale = SaleContract::new(sale_addr, client);
            let call = sale.claim();
            let receipt = send_and_wait(call).await?;
            crate::util::print_receipt(&receipt);
        }
        SaleCommand::Refund(target) => {
            let client = signer_client(cli, cfg)?;
            let sale_addr = resolve_sale_address(client.clone(), cfg.factory, target).await?;
            let sale = SaleContract::new(sale_addr, client);
            let call = sale.refund();
            let receipt = send_and_wait(call).await?;
            crate::util::print_receipt(&receipt);
        }
        SaleCommand::EmergencyRefund(target) => {
            let client = signer_client(cli, cfg)?;
            let sale_addr = resolve_sale_address(client.clone(), cfg.factory, target).await?;
            let sale = SaleContract::new(sale_addr, client);
            let call = sale.emergency_refund();
            let receipt = send_and_wait(call).await?;
            crate::util::print_receipt(&receipt);
        }
    }
    Ok(())
}

async fn resolve_sale_address<M: Middleware + 'static>(
    client: Arc<M>,
    factory_addr: Address,
    target: &SaleTarget,
) -> Result<Address> {
    if let Some(addr) = target.sale {
        return Ok(addr);
    }
    if let Some(project_id) = target.project_id {
        let project = read_factory_project(client, factory_addr, project_id).await?;
        return Ok(project.sale);
    }
    bail!("set --sale or --project-id")
}
