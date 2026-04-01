use crate::chain::client::{provider, signer_client};
use crate::chain::contracts::{
    AgentRaiseFactoryContract, read_factory_project, send_factory_set_global_config, token_meta,
};
use crate::cli::{Cli, CreateArgs, FactoryCommand};
use crate::config::RuntimeConfig;
use crate::output::{
    format_bps, format_duration_human, format_token_amount, header, key, print_project,
    print_receipt, warn,
};
use crate::util::{minutes_to_seconds_u256, parse_amount_units, send_and_wait, unix_now};
use anyhow::{Result, anyhow, bail};
use ethers::prelude::*;
use std::sync::Arc;

pub async fn run_factory(cli: &Cli, cfg: &RuntimeConfig, cmd: &FactoryCommand) -> Result<()> {
    let read_client = Arc::new(provider(cfg)?);

    match cmd {
        FactoryCommand::Info => show_info(cfg, read_client).await?,
        FactoryCommand::Global => show_global(cfg, read_client).await?,
        FactoryCommand::Collateral { collateral } => {
            show_collateral(cfg, read_client, *collateral).await?
        }
        FactoryCommand::List { from, limit } => {
            list_projects(cfg, read_client, *from, *limit).await?
        }
        FactoryCommand::Project { project_id } => {
            show_project(cfg, read_client, *project_id).await?
        }
        FactoryCommand::Snapshot { project_id } => {
            show_snapshot(cfg, read_client, *project_id).await?
        }
        FactoryCommand::Commitment { project_id, user } => {
            show_commitment(cfg, read_client, *project_id, *user).await?
        }
        FactoryCommand::AgentProjects { agent_id } => {
            show_agent_projects(cfg, read_client, *agent_id).await?
        }
        FactoryCommand::Create(args) => create_project(cli, cfg, args).await?,
        FactoryCommand::Approve { project_id } => approve_project(cli, cfg, *project_id).await?,
        FactoryCommand::Revoke { project_id } => revoke_project(cli, cfg, *project_id).await?,
        FactoryCommand::UpdateMetadata(args) => update_metadata(cli, cfg, args).await?,
        FactoryCommand::SetStatus(args) => set_status(cli, cfg, args).await?,
        FactoryCommand::SetCollateral {
            collateral,
            allowed,
        } => set_collateral(cli, cfg, *collateral, allowed.as_bool()).await?,
        FactoryCommand::SetGlobal(args) => set_global(cli, cfg, args).await?,
    }

    Ok(())
}

async fn show_info(cfg: &RuntimeConfig, read_client: Arc<Provider<Http>>) -> Result<()> {
    let factory = AgentRaiseFactoryContract::new(cfg.factory, read_client.clone());
    let count = factory.project_count().call().await?;
    let (min_raise, max_raise, fee_bps, fee_to, min_dur, max_dur, min_delay, max_delay) =
        factory.global_config().call().await?;

    println!("{} {:?}", header("Factory:"), cfg.factory);
    println!("{} {}", key("Total projects:"), count);
    print_global_config(
        min_raise, max_raise, fee_bps, fee_to, min_dur, max_dur, min_delay, max_delay,
    );

    if let Some(collateral) = cfg.default_collateral {
        match factory.allowed_collateral(collateral).call().await {
            Ok(true) => print_default_collateral(&factory, read_client, collateral).await?,
            Ok(false) => {
                println!(
                    "{} default collateral {:?} is not enabled in the factory.",
                    warn("Warning:"),
                    collateral
                );
            }
            Err(_) => {
                println!(
                    "{} this factory does not expose collateral management methods.",
                    warn("Warning:")
                );
            }
        }
    }

    Ok(())
}

async fn show_global(cfg: &RuntimeConfig, read_client: Arc<Provider<Http>>) -> Result<()> {
    let factory = AgentRaiseFactoryContract::new(cfg.factory, read_client);
    let (min_raise, max_raise, fee_bps, fee_to, min_dur, max_dur, min_delay, max_delay) =
        factory.global_config().call().await?;
    print_global_config(
        min_raise, max_raise, fee_bps, fee_to, min_dur, max_dur, min_delay, max_delay,
    );
    Ok(())
}

async fn show_collateral(
    cfg: &RuntimeConfig,
    read_client: Arc<Provider<Http>>,
    collateral: Address,
) -> Result<()> {
    let factory = AgentRaiseFactoryContract::new(cfg.factory, read_client.clone());
    let allowed = factory
        .allowed_collateral(collateral)
        .call()
        .await
        .map_err(|_| {
            anyhow!(
                "factory {:?} does not expose allowedCollateral(address)",
                cfg.factory
            )
        })?;

    println!("collateral: {:?}", collateral);
    println!("allowed: {}", allowed);

    if allowed {
        let (decimals, symbol) = token_meta(read_client, collateral).await?;
        println!("symbol: {}", symbol);
        println!(
            "min_raise: {}",
            format_token_amount(
                factory.min_raise_for_collateral(collateral).call().await?,
                decimals
            )
        );
        println!(
            "max_raise: {}",
            format_token_amount(
                factory.max_raise_for_collateral(collateral).call().await?,
                decimals
            )
        );
    }

    Ok(())
}

async fn list_projects(
    cfg: &RuntimeConfig,
    read_client: Arc<Provider<Http>>,
    from: u64,
    limit: u64,
) -> Result<()> {
    let factory = AgentRaiseFactoryContract::new(cfg.factory, read_client.clone());
    let total = factory.project_count().call().await?.as_u64();
    let start = from.min(total);
    let end = start.saturating_add(limit).min(total);

    println!("projects_total: {}", total);
    println!("range: {}..{}", start, end);

    for project_id in start..end {
        let project = read_factory_project(read_client.clone(), cfg.factory, project_id).await?;
        let approved = factory
            .is_project_approved(project_id.into())
            .call()
            .await?;
        print_project(project_id, &project, approved);
    }

    Ok(())
}

async fn show_project(
    cfg: &RuntimeConfig,
    read_client: Arc<Provider<Http>>,
    project_id: u64,
) -> Result<()> {
    let factory = AgentRaiseFactoryContract::new(cfg.factory, read_client.clone());
    let project = read_factory_project(read_client, cfg.factory, project_id).await?;
    let approved = factory
        .is_project_approved(project_id.into())
        .call()
        .await?;
    print_project(project_id, &project, approved);
    Ok(())
}

async fn show_snapshot(
    cfg: &RuntimeConfig,
    read_client: Arc<Provider<Http>>,
    project_id: u64,
) -> Result<()> {
    let snapshot = AgentRaiseFactoryContract::new(cfg.factory, read_client)
        .get_project_raise_snapshot(project_id.into())
        .call()
        .await?;

    println!("project_id: {}", project_id);
    println!("approved: {}", snapshot.0);
    println!("total_committed: {}", snapshot.1);
    println!("accepted_amount: {}", snapshot.2);
    println!("finalized: {}", snapshot.3);
    println!("failed: {}", snapshot.4);
    println!("active: {}", snapshot.5);
    println!("start_time: {}", snapshot.6);
    println!("end_time: {}", snapshot.7);
    println!("share_token: {:?}", snapshot.8);
    Ok(())
}

async fn show_commitment(
    cfg: &RuntimeConfig,
    read_client: Arc<Provider<Http>>,
    project_id: u64,
    user: Address,
) -> Result<()> {
    let committed = AgentRaiseFactoryContract::new(cfg.factory, read_client)
        .get_project_commitment(project_id.into(), user)
        .call()
        .await?;

    println!("project_id: {}", project_id);
    println!("user: {:?}", user);
    println!("committed: {}", committed);
    Ok(())
}

async fn show_agent_projects(
    cfg: &RuntimeConfig,
    read_client: Arc<Provider<Http>>,
    agent_id: u64,
) -> Result<()> {
    let ids = AgentRaiseFactoryContract::new(cfg.factory, read_client)
        .get_agent_projects(agent_id.into())
        .call()
        .await?;

    println!("agent_id: {}", agent_id);
    println!("project_ids: {:?}", ids);
    Ok(())
}

async fn create_project(cli: &Cli, cfg: &RuntimeConfig, args: &CreateArgs) -> Result<()> {
    let write_client = signer_client(cli, cfg)?;
    let sender = write_client.address();
    let collateral = args.collateral.or(cfg.default_collateral).ok_or_else(|| {
        anyhow!("collateral not set: pass --collateral or configure USDM in deployment file")
    })?;
    let launch_timestamp = unix_now()?
        .checked_add(
            args.launch_in_minutes
                .checked_mul(60)
                .ok_or_else(|| anyhow!("overflow on launch_in_minutes"))?,
        )
        .ok_or_else(|| anyhow!("overflow on launch timestamp"))?;

    let call = AgentRaiseFactoryContract::new(cfg.factory, write_client).create_agent_raise(
        args.agent_id.into(),
        args.name.clone(),
        args.description.clone(),
        args.categories.clone(),
        args.agent_address.unwrap_or(sender),
        collateral,
        minutes_to_seconds_u256(args.duration_minutes, "duration_minutes")?,
        launch_timestamp.into(),
        args.token_name.clone(),
        args.token_symbol.clone(),
    );

    if let Ok(predicted_id) = call.clone().call().await {
        println!("predicted_project_id: {}", predicted_id);
    }

    print_receipt(&send_and_wait(call).await?);
    Ok(())
}

async fn approve_project(cli: &Cli, cfg: &RuntimeConfig, project_id: u64) -> Result<()> {
    let write_client = signer_client(cli, cfg)?;
    let call = AgentRaiseFactoryContract::new(cfg.factory, write_client)
        .approve_project(project_id.into());
    print_receipt(&send_and_wait(call).await?);
    Ok(())
}

async fn revoke_project(cli: &Cli, cfg: &RuntimeConfig, project_id: u64) -> Result<()> {
    let write_client = signer_client(cli, cfg)?;
    let call =
        AgentRaiseFactoryContract::new(cfg.factory, write_client).revoke_project(project_id.into());
    print_receipt(&send_and_wait(call).await?);
    Ok(())
}

async fn update_metadata(
    cli: &Cli,
    cfg: &RuntimeConfig,
    args: &crate::cli::UpdateMetadataArgs,
) -> Result<()> {
    let write_client = signer_client(cli, cfg)?;
    let call = AgentRaiseFactoryContract::new(cfg.factory, write_client).update_project_metadata(
        args.project_id.into(),
        args.description.clone(),
        args.categories.clone(),
    );
    print_receipt(&send_and_wait(call).await?);
    Ok(())
}

async fn set_status(
    cli: &Cli,
    cfg: &RuntimeConfig,
    args: &crate::cli::SetStatusArgs,
) -> Result<()> {
    let write_client = signer_client(cli, cfg)?;
    let call = AgentRaiseFactoryContract::new(cfg.factory, write_client)
        .update_project_operational_status(
            args.project_id.into(),
            args.status.as_u8(),
            args.status_note.clone(),
        );
    print_receipt(&send_and_wait(call).await?);
    Ok(())
}

async fn set_collateral(
    cli: &Cli,
    cfg: &RuntimeConfig,
    collateral: Address,
    allowed: bool,
) -> Result<()> {
    let write_client = signer_client(cli, cfg)?;
    let factory = AgentRaiseFactoryContract::new(cfg.factory, write_client.clone());
    if factory.allowed_collateral(collateral).call().await.is_err() {
        bail!(
            "factory {:?} does not support set-collateral (allowedCollateral missing).",
            cfg.factory
        );
    }
    let call = AgentRaiseFactoryContract::new(cfg.factory, write_client)
        .set_allowed_collateral(collateral, allowed);
    print_receipt(&send_and_wait(call).await?);
    Ok(())
}

async fn set_global(
    cli: &Cli,
    cfg: &RuntimeConfig,
    args: &crate::cli::SetGlobalArgs,
) -> Result<()> {
    let write_client = signer_client(cli, cfg)?;
    let receipt = send_factory_set_global_config(
        write_client,
        cfg.factory,
        (
            parse_amount_units(&args.min_raise, 18)?,
            parse_amount_units(&args.max_raise, 18)?,
            args.platform_fee_bps,
            args.platform_fee_recipient,
            minutes_to_seconds_u256(args.min_duration_minutes, "min_duration_minutes")?,
            minutes_to_seconds_u256(args.max_duration_minutes, "max_duration_minutes")?,
            minutes_to_seconds_u256(args.min_launch_delay_minutes, "min_launch_delay_minutes")?,
            minutes_to_seconds_u256(args.max_launch_delay_minutes, "max_launch_delay_minutes")?,
        ),
    )
    .await?;

    print_receipt(&receipt);
    Ok(())
}

async fn print_default_collateral(
    factory: &AgentRaiseFactoryContract<Provider<Http>>,
    read_client: Arc<Provider<Http>>,
    collateral: Address,
) -> Result<()> {
    let (decimals, symbol) = token_meta(read_client, collateral).await?;
    let min_raise = factory.min_raise_for_collateral(collateral).call().await?;
    let max_raise = factory.max_raise_for_collateral(collateral).call().await?;

    println!(
        "{} {:?} ({symbol})",
        key("Default collateral (from deployment):"),
        collateral
    );
    println!(
        "  {} {} [raw {}]",
        key(format!("Min raise in {symbol}:")),
        format_token_amount(min_raise, decimals),
        min_raise
    );
    println!(
        "  {} {} [raw {}]",
        key(format!("Max raise in {symbol}:")),
        format_token_amount(max_raise, decimals),
        max_raise
    );
    Ok(())
}

fn print_global_config(
    min_raise: U256,
    max_raise: U256,
    fee_bps: u16,
    fee_to: Address,
    min_duration: U256,
    max_duration: U256,
    min_launch_delay: U256,
    max_launch_delay: U256,
) {
    println!("{}", header("Global config"));
    println!(
        "  {} {} [raw {}]",
        key("Min raise (18 decimals):"),
        format_token_amount(min_raise, 18),
        min_raise
    );
    println!(
        "  {} {} [raw {}]",
        key("Max raise (18 decimals):"),
        format_token_amount(max_raise, 18),
        max_raise
    );
    println!("  {} {}", key("Platform fee:"), format_bps(fee_bps));
    println!("  {} {:?}", key("Fee recipient:"), fee_to);
    println!(
        "  {} {}",
        key("Minimum sale duration:"),
        format_duration_human(min_duration)
    );
    println!(
        "  {} {}",
        key("Maximum sale duration:"),
        format_duration_human(max_duration)
    );
    println!(
        "  {} {}",
        key("Minimum launch delay:"),
        format_duration_human(min_launch_delay)
    );
    println!(
        "  {} {}",
        key("Maximum launch delay:"),
        format_duration_human(max_launch_delay)
    );
}
