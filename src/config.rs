use crate::cli::NetworkName;
use anyhow::{Context, Result, anyhow};
use ethers::prelude::Address;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct DeploymentFile {
    pub network: String,
    #[serde(rename = "chainId")]
    pub chain_id: u64,
    pub rpc: String,
    pub contracts: HashMap<String, String>,
    #[serde(default)]
    pub external: HashMap<String, String>,
}

#[derive(Debug)]
pub struct RuntimeConfig {
    pub network: NetworkName,
    pub network_label: String,
    pub chain_id: u64,
    pub rpc_url: String,
    pub factory: Address,
    pub allowlist: Address,
    pub default_collateral: Option<Address>,
    pub deployment_path: PathBuf,
}

impl RuntimeConfig {
    pub fn resolve(cli: &crate::cli::Cli) -> Result<Self> {
        let cwd = std::env::current_dir().context("cannot read current directory")?;
        let root = find_repo_root(&cwd)
            .ok_or_else(|| anyhow!("cannot locate repo root containing backend/deployments"))?;

        let deployment_path = root
            .join("backend")
            .join("deployments")
            .join(cli.network.deployment_file());

        let raw = fs::read_to_string(&deployment_path)
            .with_context(|| format!("cannot read {}", deployment_path.display()))?;
        let deployment: DeploymentFile = serde_json::from_str(&raw)
            .with_context(|| format!("invalid JSON in {}", deployment_path.display()))?;

        let factory_from_file = parse_map_addr(&deployment.contracts, "AgentRaiseFactory")?;
        let allowlist_from_file = parse_map_addr(&deployment.contracts, "ContractAllowlist")?;
        let default_collateral = deployment
            .external
            .get("USDM")
            .map(|v| v.parse::<Address>())
            .transpose()
            .context("invalid USDM address in deployment file")?;

        Ok(Self {
            network: cli.network,
            network_label: deployment.network,
            chain_id: deployment.chain_id,
            rpc_url: cli.rpc_url.clone().unwrap_or(deployment.rpc),
            factory: cli.factory.unwrap_or(factory_from_file),
            allowlist: cli.allowlist.unwrap_or(allowlist_from_file),
            default_collateral,
            deployment_path,
        })
    }
}

fn parse_map_addr(map: &HashMap<String, String>, key: &str) -> Result<Address> {
    let raw = map
        .get(key)
        .ok_or_else(|| anyhow!("missing key `{key}` in deployment file"))?;
    raw.parse::<Address>()
        .with_context(|| format!("invalid address for key `{key}`: {raw}"))
}

pub fn find_repo_root(start: &Path) -> Option<PathBuf> {
    let mut cur = Some(start.to_path_buf());
    while let Some(path) = cur {
        if path.join("backend").join("deployments").is_dir() {
            return Some(path);
        }
        cur = path.parent().map(|p| p.to_path_buf());
    }
    None
}
