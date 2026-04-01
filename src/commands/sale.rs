use crate::chain::client::{provider, signer_client};
use crate::chain::contracts::{
    AgentRaiseFactoryContract, Erc20Metadata, SaleContract, read_factory_project,
    read_sale_collateral, read_sale_project_id, token_meta,
};
use crate::cli::{Cli, SaleCommand, SaleTarget};
use crate::config::RuntimeConfig;
use crate::output::{format_token_amount, print_receipt};
use crate::util::{parse_amount_raw, parse_amount_units, send_and_wait};
use anyhow::{Result, bail};
use ethers::prelude::*;
use std::sync::Arc;

pub async fn run_sale(cli: &Cli, cfg: &RuntimeConfig, cmd: &SaleCommand) -> Result<()> {
    let read_client = Arc::new(provider(cfg)?);

    match cmd {
        SaleCommand::Status(target) => show_status(cfg, read_client, target).await?,
        SaleCommand::Claimable { target, user } => {
            show_claimable(cfg, read_client, target, *user).await?
        }
        SaleCommand::Refundable { target, user } => {
            show_refundable(cfg, read_client, target, *user).await?
        }
        SaleCommand::Commitment { target, user } => {
            show_commitment(cfg, read_client, target, *user).await?
        }
        SaleCommand::ApproveCollateral {
            target,
            amount,
            raw,
        } => approve_collateral(cli, cfg, target, amount, *raw).await?,
        SaleCommand::Commit {
            target,
            amount,
            raw,
        } => commit(cli, cfg, target, amount, *raw).await?,
        SaleCommand::Finalize(target) => {
            run_sale_tx(cli, cfg, target, |sale| sale.finalize()).await?
        }
        SaleCommand::Claim(target) => run_sale_tx(cli, cfg, target, |sale| sale.claim()).await?,
        SaleCommand::Refund(target) => run_sale_tx(cli, cfg, target, |sale| sale.refund()).await?,
        SaleCommand::EmergencyRefund(target) => {
            run_sale_tx(cli, cfg, target, |sale| sale.emergency_refund()).await?
        }
    }

    Ok(())
}

async fn show_status(
    cfg: &RuntimeConfig,
    read_client: Arc<Provider<Http>>,
    target: &SaleTarget,
) -> Result<()> {
    let sale_address = resolve_sale_address(read_client.clone(), cfg.factory, target).await?;
    let sale = SaleContract::new(sale_address, read_client.clone());
    let collateral = read_sale_collateral(read_client.clone(), sale_address).await?;
    let (decimals, symbol) = token_meta(read_client.clone(), collateral).await?;
    let (total, accepted, finalized, failed) = sale.get_status().call().await?;

    println!("sale: {:?}", sale_address);
    if let Ok(project_id) = read_sale_project_id(read_client.clone(), sale_address).await {
        println!("project_id: {}", project_id);
        if let Ok(approved) = AgentRaiseFactoryContract::new(cfg.factory, read_client.clone())
            .is_project_approved(project_id)
            .call()
            .await
        {
            println!("project_approved: {}", approved);
        }
    }
    println!("collateral: {:?} ({})", collateral, symbol);
    println!("start_time: {}", sale.start_time().call().await?);
    println!("end_time: {}", sale.end_time().call().await?);
    println!("is_active: {}", sale.is_active().call().await?);
    println!(
        "time_remaining_seconds: {}",
        sale.time_remaining().call().await?
    );
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
    println!("share_token: {:?}", sale.token().call().await?);
    Ok(())
}

async fn show_claimable(
    cfg: &RuntimeConfig,
    read_client: Arc<Provider<Http>>,
    target: &SaleTarget,
    user: Address,
) -> Result<()> {
    let sale_address = resolve_sale_address(read_client.clone(), cfg.factory, target).await?;
    let sale = SaleContract::new(sale_address, read_client.clone());
    let collateral = read_sale_collateral(read_client.clone(), sale_address).await?;
    let (decimals, symbol) = token_meta(read_client, collateral).await?;
    let (payout, refund) = sale.get_claimable(user).call().await?;

    println!("sale: {:?}", sale_address);
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
    Ok(())
}

async fn show_refundable(
    cfg: &RuntimeConfig,
    read_client: Arc<Provider<Http>>,
    target: &SaleTarget,
    user: Address,
) -> Result<()> {
    let sale_address = resolve_sale_address(read_client.clone(), cfg.factory, target).await?;
    let sale = SaleContract::new(sale_address, read_client.clone());
    let collateral = read_sale_collateral(read_client.clone(), sale_address).await?;
    let (decimals, symbol) = token_meta(read_client, collateral).await?;
    let refundable = sale.get_refundable(user).call().await?;

    println!("sale: {:?}", sale_address);
    println!("user: {:?}", user);
    println!("token: {}", symbol);
    println!(
        "refundable: {} ({})",
        format_token_amount(refundable, decimals),
        refundable
    );
    Ok(())
}

async fn show_commitment(
    cfg: &RuntimeConfig,
    read_client: Arc<Provider<Http>>,
    target: &SaleTarget,
    user: Address,
) -> Result<()> {
    let sale_address = resolve_sale_address(read_client.clone(), cfg.factory, target).await?;
    let sale = SaleContract::new(sale_address, read_client.clone());
    let collateral = read_sale_collateral(read_client.clone(), sale_address).await?;
    let (decimals, symbol) = token_meta(read_client, collateral).await?;
    let committed = sale.commitments(user).call().await?;

    println!("sale: {:?}", sale_address);
    println!("user: {:?}", user);
    println!("token: {}", symbol);
    println!(
        "committed: {} ({})",
        format_token_amount(committed, decimals),
        committed
    );
    Ok(())
}

async fn approve_collateral(
    cli: &Cli,
    cfg: &RuntimeConfig,
    target: &SaleTarget,
    amount: &str,
    raw: bool,
) -> Result<()> {
    let write_client = signer_client(cli, cfg)?;
    let sale_address = resolve_sale_address(write_client.clone(), cfg.factory, target).await?;
    let collateral = read_sale_collateral(write_client.clone(), sale_address).await?;
    let token = Erc20Metadata::new(collateral, write_client.clone());
    let (decimals, symbol) = token_meta(write_client.clone(), collateral).await?;
    let amount_raw = parse_token_amount(amount, decimals, raw)?;

    let receipt = send_and_wait(token.approve(sale_address, amount_raw)).await?;
    println!("owner: {:?}", write_client.address());
    println!(
        "approved: {} {} ({})",
        format_token_amount(amount_raw, decimals),
        symbol,
        amount_raw
    );
    print_receipt(&receipt);
    Ok(())
}

async fn commit(
    cli: &Cli,
    cfg: &RuntimeConfig,
    target: &SaleTarget,
    amount: &str,
    raw: bool,
) -> Result<()> {
    let write_client = signer_client(cli, cfg)?;
    let sale_address = resolve_sale_address(write_client.clone(), cfg.factory, target).await?;
    let collateral = read_sale_collateral(write_client.clone(), sale_address).await?;
    let sale = SaleContract::new(sale_address, write_client.clone());
    let token = Erc20Metadata::new(collateral, write_client.clone());
    let (decimals, symbol) = token_meta(write_client.clone(), collateral).await?;
    let amount_raw = parse_token_amount(amount, decimals, raw)?;
    let sender = write_client.address();

    let allowance = token.allowance(sender, sale_address).call().await?;
    if allowance < amount_raw {
        bail!(
            "insufficient allowance: allowance={} required={}. Run `sale approve-collateral` first.",
            format_token_amount(allowance, decimals),
            format_token_amount(amount_raw, decimals)
        );
    }

    let balance = token.balance_of(sender).call().await?;
    if balance < amount_raw {
        bail!(
            "insufficient balance: balance={} required={}",
            format_token_amount(balance, decimals),
            format_token_amount(amount_raw, decimals)
        );
    }

    let receipt = send_and_wait(sale.commit(amount_raw)).await?;
    println!("sender: {:?}", sender);
    println!(
        "committed: {} {} ({})",
        format_token_amount(amount_raw, decimals),
        symbol,
        amount_raw
    );
    print_receipt(&receipt);
    Ok(())
}

async fn run_sale_tx<F, D>(
    cli: &Cli,
    cfg: &RuntimeConfig,
    target: &SaleTarget,
    build: F,
) -> Result<()>
where
    F: FnOnce(
        SaleContract<crate::chain::contracts::SignerClient>,
    )
        -> ethers::contract::builders::ContractCall<crate::chain::contracts::SignerClient, D>,
    D: ethers::abi::Detokenize,
{
    let write_client = signer_client(cli, cfg)?;
    let sale_address = resolve_sale_address(write_client.clone(), cfg.factory, target).await?;
    let receipt = send_and_wait(build(SaleContract::new(sale_address, write_client))).await?;
    print_receipt(&receipt);
    Ok(())
}

async fn resolve_sale_address<M: Middleware + 'static>(
    client: Arc<M>,
    factory: Address,
    target: &SaleTarget,
) -> Result<Address> {
    if let Some(address) = target.sale {
        return Ok(address);
    }
    if let Some(project_id) = target.project_id {
        return Ok(read_factory_project(client, factory, project_id)
            .await?
            .sale);
    }
    bail!("set --sale or --project-id")
}

fn parse_token_amount(amount: &str, decimals: u8, raw: bool) -> Result<U256> {
    if raw {
        parse_amount_raw(amount)
    } else {
        parse_amount_units(amount, decimals)
    }
}
