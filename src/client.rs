use crate::cli::Cli;
use crate::config::RuntimeConfig;
use crate::contracts::SignerClient;
use anyhow::{Context, Result, anyhow};
use ethers::prelude::*;
use std::sync::Arc;

pub fn provider(cfg: &RuntimeConfig) -> Result<Provider<Http>> {
    Provider::<Http>::try_from(cfg.rpc_url.as_str())
        .with_context(|| format!("invalid RPC URL: {}", cfg.rpc_url))
}

pub fn signer_client(cli: &Cli, cfg: &RuntimeConfig) -> Result<Arc<SignerClient>> {
    let pk = cli
        .private_key
        .clone()
        .or_else(|| std::env::var("BACKED_PRIVATE_KEY").ok())
        .ok_or_else(|| anyhow!("write command: set --private-key or BACKED_PRIVATE_KEY"))?;

    let wallet = pk
        .parse::<LocalWallet>()
        .context("invalid private key format")?
        .with_chain_id(cfg.chain_id);

    let p = provider(cfg)?;
    Ok(Arc::new(SignerMiddleware::new(p, wallet)))
}
