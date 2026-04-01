use crate::cli::{Cli, FactoryCommand};
use crate::client::signer_client;
use crate::config::RuntimeConfig;
use crate::contracts::{
    AgentRaiseFactoryContract, ProjectView, read_factory_project, send_factory_set_global_config,
    token_meta,
};
use crate::formatters::{
    format_bps, format_duration_human, format_status, format_token_amount, format_ts, header, key,
    warn,
};
use crate::util::{minutes_to_seconds_u256, parse_amount_units, send_and_wait, unix_now};
use anyhow::{Result, anyhow, bail};
use ethers::prelude::*;
use std::sync::Arc;

pub async fn run_factory(cli: &Cli, cfg: &RuntimeConfig, cmd: &FactoryCommand) -> Result<()> {
    let p = Arc::new(crate::client::provider(cfg)?);
    match cmd {
        FactoryCommand::Info => show_info(cfg, p.clone()).await?,
        FactoryCommand::Global => show_global(cfg, p.clone()).await?,
        FactoryCommand::Collateral { collateral } => {
            show_collateral(cfg, p.clone(), *collateral).await?
        }
        FactoryCommand::List { from, limit } => {
            list_projects(cfg, p.clone(), *from, *limit).await?
        }
        FactoryCommand::Project { project_id } => show_project(cfg, p.clone(), *project_id).await?,
        FactoryCommand::Snapshot { project_id } => {
            show_snapshot(cfg, p.clone(), *project_id).await?
        }
        FactoryCommand::Commitment { project_id, user } => {
            show_commitment(cfg, p.clone(), *project_id, *user).await?
        }
        FactoryCommand::AgentProjects { agent_id } => {
            show_agent_projects(cfg, p.clone(), *agent_id).await?
        }
        FactoryCommand::Create(args) => create_project(cli, cfg, args).await?,
        FactoryCommand::Approve { project_id } => {
            let client = signer_client(cli, cfg)?;
            let factory = AgentRaiseFactoryContract::new(cfg.factory, client);
            let call = factory.approve_project((*project_id).into());
            let receipt = send_and_wait(call).await?;
            crate::util::print_receipt(&receipt);
        }
        FactoryCommand::Revoke { project_id } => {
            let client = signer_client(cli, cfg)?;
            let factory = AgentRaiseFactoryContract::new(cfg.factory, client);
            let call = factory.revoke_project((*project_id).into());
            let receipt = send_and_wait(call).await?;
            crate::util::print_receipt(&receipt);
        }
        FactoryCommand::UpdateMetadata(args) => {
            let client = signer_client(cli, cfg)?;
            let factory = AgentRaiseFactoryContract::new(cfg.factory, client);
            let call = factory.update_project_metadata(
                args.project_id.into(),
                args.description.clone(),
                args.categories.clone(),
            );
            let receipt = send_and_wait(call).await?;
            crate::util::print_receipt(&receipt);
        }
        FactoryCommand::SetStatus(args) => {
            let client = signer_client(cli, cfg)?;
            let factory = AgentRaiseFactoryContract::new(cfg.factory, client);
            let call = factory.update_project_operational_status(
                args.project_id.into(),
                args.status.as_u8(),
                args.status_note.clone(),
            );
            let receipt = send_and_wait(call).await?;
            crate::util::print_receipt(&receipt);
        }
        FactoryCommand::SetCollateral {
            collateral,
            allowed,
        } => {
            let client = signer_client(cli, cfg)?;
            let factory = AgentRaiseFactoryContract::new(cfg.factory, client);
            if factory
                .allowed_collateral(*collateral)
                .call()
                .await
                .is_err()
            {
                bail!(
                    "factory {:?} does not support set-collateral (allowedCollateral missing). Use a latest AgentRaiseFactory address.",
                    cfg.factory
                );
            }
            let call = factory.set_allowed_collateral(*collateral, allowed.as_bool());
            let receipt = send_and_wait(call).await?;
            crate::util::print_receipt(&receipt);
        }
        FactoryCommand::SetGlobal(args) => {
            let client = signer_client(cli, cfg)?;
            let min_raise = parse_amount_units(&args.min_raise, 18)?;
            let max_raise = parse_amount_units(&args.max_raise, 18)?;
            let min_duration =
                minutes_to_seconds_u256(args.min_duration_minutes, "min_duration_minutes")?;
            let max_duration =
                minutes_to_seconds_u256(args.max_duration_minutes, "max_duration_minutes")?;
            let min_launch_delay =
                minutes_to_seconds_u256(args.min_launch_delay_minutes, "min_launch_delay_minutes")?;
            let max_launch_delay =
                minutes_to_seconds_u256(args.max_launch_delay_minutes, "max_launch_delay_minutes")?;

            let receipt = send_factory_set_global_config(
                client,
                cfg.factory,
                (
                    min_raise,
                    max_raise,
                    args.platform_fee_bps,
                    args.platform_fee_recipient,
                    min_duration,
                    max_duration,
                    min_launch_delay,
                    max_launch_delay,
                ),
            )
            .await?;
            crate::util::print_receipt(&receipt);
        }
    }
    Ok(())
}

async fn show_info(cfg: &RuntimeConfig, client: Arc<Provider<Http>>) -> Result<()> {
    let factory = AgentRaiseFactoryContract::new(cfg.factory, client.clone());
    let count = factory.project_count().call().await?;
    let (min_raise, max_raise, fee_bps, fee_to, min_dur, max_dur, min_delay, max_delay) =
        factory.global_config().call().await?;

    println!("{} {:?}", header("Factory:"), cfg.factory);
    println!("{} {}", key("Total projects:"), count);
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
        format_duration_human(min_dur)
    );
    println!(
        "  {} {}",
        key("Maximum sale duration:"),
        format_duration_human(max_dur)
    );
    println!(
        "  {} {}",
        key("Minimum launch delay:"),
        format_duration_human(min_delay)
    );
    println!(
        "  {} {}",
        key("Maximum launch delay:"),
        format_duration_human(max_delay)
    );

    if let Some(collateral) = cfg.default_collateral {
        match factory.allowed_collateral(collateral).call().await {
            Ok(true) => {
                if let Ok((decimals, symbol)) = token_meta(client.clone(), collateral).await {
                    let min_c = factory.min_raise_for_collateral(collateral).call().await?;
                    let max_c = factory.max_raise_for_collateral(collateral).call().await?;
                    println!(
                        "{} {:?} ({symbol})",
                        key("Default collateral (from deployment):"),
                        collateral
                    );
                    println!(
                        "  {} {} [raw {}]",
                        key(format!("Min raise in {symbol}:")),
                        format_token_amount(min_c, decimals),
                        min_c
                    );
                    println!(
                        "  {} {} [raw {}]",
                        key(format!("Max raise in {symbol}:")),
                        format_token_amount(max_c, decimals),
                        max_c
                    );
                }
            }
            Ok(false) => {
                println!(
                    "{} default collateral {:?} is NOT enabled in the factory.",
                    warn("Warning:"),
                    collateral,
                );
                println!(
                    "  Enable it with: backed --private-key <ADMIN_PK> factory set-collateral {:?} true",
                    collateral,
                );
            }
            Err(_) => {
                println!(
                    "{} this factory does not expose allowedCollateral/setAllowedCollateral (legacy ABI).",
                    warn("Warning:")
                );
                println!("  Current factory: {:?}", cfg.factory);
                println!(
                    "  Action: deploy/update to latest AgentRaiseFactory and set --factory to the new address."
                );
            }
        }
    }
    Ok(())
}

async fn show_global(cfg: &RuntimeConfig, client: Arc<Provider<Http>>) -> Result<()> {
    let factory = AgentRaiseFactoryContract::new(cfg.factory, client.clone());
    let (min_raise, max_raise, fee_bps, fee_to, min_dur, max_dur, min_delay, max_delay) =
        factory.global_config().call().await?;
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
        format_duration_human(min_dur)
    );
    println!(
        "  {} {}",
        key("Maximum sale duration:"),
        format_duration_human(max_dur)
    );
    println!(
        "  {} {}",
        key("Minimum launch delay:"),
        format_duration_human(min_delay)
    );
    println!(
        "  {} {}",
        key("Maximum launch delay:"),
        format_duration_human(max_delay)
    );
    Ok(())
}

async fn show_collateral(
    cfg: &RuntimeConfig,
    client: Arc<Provider<Http>>,
    collateral: Address,
) -> Result<()> {
    let factory = AgentRaiseFactoryContract::new(cfg.factory, client.clone());
    let allowed = match factory.allowed_collateral(collateral).call().await {
        Ok(v) => v,
        Err(_) => {
            bail!(
                "factory {:?} does not expose allowedCollateral(address). Likely legacy deployment.",
                cfg.factory
            );
        }
    };
    println!("collateral: {:?}", collateral);
    println!("allowed: {}", allowed);
    if allowed {
        let (decimals, symbol) = token_meta(client.clone(), collateral).await?;
        let min = factory.min_raise_for_collateral(collateral).call().await?;
        let max = factory.max_raise_for_collateral(collateral).call().await?;
        println!("symbol: {}", symbol);
        println!(
            "min_raise: {} ({})",
            format_token_amount(min, decimals),
            min
        );
        println!(
            "max_raise: {} ({})",
            format_token_amount(max, decimals),
            max
        );
    }
    Ok(())
}

async fn list_projects(
    cfg: &RuntimeConfig,
    client: Arc<Provider<Http>>,
    from: u64,
    limit: u64,
) -> Result<()> {
    let factory = AgentRaiseFactoryContract::new(cfg.factory, client.clone());
    let count_u256 = factory.project_count().call().await?;
    let count = count_u256.as_u64();
    let start = from.min(count);
    let end = start.saturating_add(limit).min(count);
    println!("projects_total: {}", count);
    println!("range: {}..{}", start, end);

    for id in start..end {
        let project = read_factory_project(client.clone(), cfg.factory, id).await?;
        let approved = factory.is_project_approved(id.into()).call().await?;
        print_project(id, &project, approved);
    }
    Ok(())
}

async fn show_project(
    cfg: &RuntimeConfig,
    client: Arc<Provider<Http>>,
    project_id: u64,
) -> Result<()> {
    let factory = AgentRaiseFactoryContract::new(cfg.factory, client.clone());
    let project = read_factory_project(client.clone(), cfg.factory, project_id).await?;
    let approved = factory
        .is_project_approved(project_id.into())
        .call()
        .await?;
    print_project(project_id, &project, approved);
    Ok(())
}

async fn show_snapshot(
    cfg: &RuntimeConfig,
    client: Arc<Provider<Http>>,
    project_id: u64,
) -> Result<()> {
    let factory = AgentRaiseFactoryContract::new(cfg.factory, client.clone());
    let s = factory
        .get_project_raise_snapshot(project_id.into())
        .call()
        .await?;
    println!("project_id: {}", project_id);
    println!("approved: {}", s.0);
    println!("total_committed: {}", s.1);
    println!("accepted_amount: {}", s.2);
    println!("finalized: {}", s.3);
    println!("failed: {}", s.4);
    println!("active: {}", s.5);
    println!("start_time: {}", s.6);
    println!("end_time: {}", s.7);
    println!("share_token: {:?}", s.8);
    Ok(())
}

async fn show_commitment(
    cfg: &RuntimeConfig,
    client: Arc<Provider<Http>>,
    project_id: u64,
    user: Address,
) -> Result<()> {
    let factory = AgentRaiseFactoryContract::new(cfg.factory, client.clone());
    let committed = factory
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
    client: Arc<Provider<Http>>,
    agent_id: u64,
) -> Result<()> {
    let factory = AgentRaiseFactoryContract::new(cfg.factory, client);
    let ids = factory.get_agent_projects(agent_id.into()).call().await?;
    println!("agent_id: {}", agent_id);
    println!("project_ids: {:?}", ids);
    Ok(())
}

async fn create_project(
    cli: &Cli,
    cfg: &RuntimeConfig,
    args: &crate::cli::CreateArgs,
) -> Result<()> {
    let client = signer_client(cli, cfg)?;
    let sender = client.address();
    let factory = AgentRaiseFactoryContract::new(cfg.factory, client.clone());

    let collateral = args.collateral.or(cfg.default_collateral).ok_or_else(|| {
        anyhow!("collateral not set: pass --collateral or configure USDM in deployment file")
    })?;
    let agent_address = args.agent_address.unwrap_or(sender);
    let duration = minutes_to_seconds_u256(args.duration_minutes, "duration_minutes")?;
    let launch_delay_seconds = args
        .launch_in_minutes
        .checked_mul(60)
        .ok_or_else(|| anyhow!("overflow on launch_in_minutes"))?;
    let launch_ts = unix_now()?
        .checked_add(launch_delay_seconds)
        .ok_or_else(|| anyhow!("overflow on launch timestamp"))?;

    let call = factory.create_agent_raise(
        args.agent_id.into(),
        args.name.clone(),
        args.description.clone(),
        args.categories.clone(),
        agent_address,
        collateral,
        duration,
        launch_ts.into(),
        args.token_name.clone(),
        args.token_symbol.clone(),
    );

    if let Ok(predicted_id) = call.clone().call().await {
        println!("predicted_project_id: {}", predicted_id);
    }

    let receipt = send_and_wait(call).await?;
    crate::util::print_receipt(&receipt);
    Ok(())
}

fn print_project(id: u64, project: &ProjectView, approved: bool) {
    println!("---");
    println!("project_id: {}", id);
    println!("name: {}", project.name);
    println!("description: {}", project.description);
    if !project.categories.is_empty() {
        println!("categories: {}", project.categories);
    }
    println!("agent_id: {}", project.agent_id);
    println!("agent: {:?}", project.agent);
    println!("treasury: {:?}", project.treasury);
    println!("sale: {:?}", project.sale);
    println!("agent_executor: {:?}", project.agent_executor);
    println!("collateral: {:?}", project.collateral);
    println!(
        "operational_status: {} ({})",
        project.operational_status,
        format_status(project.operational_status)
    );
    if !project.status_note.is_empty() {
        println!("status_note: {}", project.status_note);
    }
    println!("approved: {}", approved);
    println!("created_at: {}", format_ts(project.created_at));
    println!("updated_at: {}", format_ts(project.updated_at));
}
