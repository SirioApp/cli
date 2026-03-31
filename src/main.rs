use anyhow::{Context, Result, anyhow, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use ethers::abi::{Abi, AbiParser, Detokenize};
use ethers::contract::builders::ContractCall;
use ethers::prelude::*;
use ethers::utils::{format_units, parse_units};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

abigen!(
    AgentRaiseFactoryContract,
    r#"[
        function globalConfig() view returns (uint256,uint256,uint16,address,uint256,uint256,uint256,uint256)
        function projectCount() view returns (uint256)
        function getAgentProjects(uint256 agentId) view returns (uint256[])
        function isProjectApproved(uint256 id) view returns (bool)
        function allowedCollateral(address collateral) view returns (bool)
        function minRaiseForCollateral(address collateral) view returns (uint256)
        function maxRaiseForCollateral(address collateral) view returns (uint256)
        function createAgentRaise(uint256,string,string,string,address,address,uint256,uint256,string,string) returns (uint256)
        function approveProject(uint256 projectId)
        function revokeProject(uint256 projectId)
        function setAllowedCollateral(address collateral, bool allowed)
        function updateProjectMetadata(uint256 projectId, string description, string categories)
        function updateProjectOperationalStatus(uint256 projectId, uint8 status, string statusNote)
        function getProjectRaiseSnapshot(uint256 projectId) view returns (bool,uint256,uint256,bool,bool,bool,uint256,uint256,address)
        function getProjectCommitment(uint256 projectId, address user) view returns (uint256)
        event AgentRaiseCreated(uint256 indexed projectId,uint256 indexed agentId,string name,address indexed agent,address treasury,address sale,address agentExecutor,address collateral)
    ]"#
);

abigen!(
    SaleContract,
    r#"[
        function startTime() view returns (uint256)
        function endTime() view returns (uint256)
        function totalCommitted() view returns (uint256)
        function acceptedAmount() view returns (uint256)
        function finalized() view returns (bool)
        function failed() view returns (bool)
        function getStatus() view returns (uint256 totalCommitted, uint256 acceptedAmount, bool finalized, bool failed)
        function getClaimable(address user) view returns (uint256 payoutUsdm, uint256 refundAmt)
        function getRefundable(address user) view returns (uint256)
        function isActive() view returns (bool)
        function timeRemaining() view returns (uint256)
        function token() view returns (address)
        function commitments(address user) view returns (uint256)
        function claim()
        function refund()
        function commit(uint256 amount)
        function finalize()
        function emergencyRefund()
    ]"#
);

abigen!(
    ContractAllowlistContract,
    r#"[
        function admin() view returns (address)
        function isAllowed(address target) view returns (bool)
        function addContract(address target)
        function removeContract(address target)
        function transferAdmin(address newAdmin)
    ]"#
);

abigen!(
    Erc20Metadata,
    r#"[
        function decimals() view returns (uint8)
        function symbol() view returns (string)
        function balanceOf(address owner) view returns (uint256)
        function allowance(address owner, address spender) view returns (uint256)
        function approve(address spender, uint256 amount) returns (bool)
    ]"#
);

type SignerClient = SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>;

#[derive(Copy, Clone, Debug, ValueEnum)]
enum NetworkName {
    Testnet,
    Mainnet,
}

impl NetworkName {
    fn deployment_file(self) -> &'static str {
        match self {
            Self::Testnet => "megaeth-testnet.json",
            Self::Mainnet => "megaeth-mainnet.json",
        }
    }
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum BoolValue {
    True,
    False,
}

impl BoolValue {
    fn as_bool(self) -> bool {
        matches!(self, Self::True)
    }
}

#[derive(Parser, Debug)]
#[command(name = "backed", version, about = "Backed protocol CLI")]
struct Cli {
    #[arg(long, value_enum, default_value = "testnet", env = "BACKED_NETWORK")]
    network: NetworkName,
    #[arg(long, env = "BACKED_RPC_URL")]
    rpc_url: Option<String>,
    #[arg(long, env = "BACKED_FACTORY")]
    factory: Option<Address>,
    #[arg(long, env = "BACKED_ALLOWLIST")]
    allowlist: Option<Address>,
    #[arg(long, env = "BACKED_PRIVATE_KEY")]
    private_key: Option<String>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Network,
    Factory {
        #[command(subcommand)]
        cmd: FactoryCommand,
    },
    Sale {
        #[command(subcommand)]
        cmd: SaleCommand,
    },
    Allowlist {
        #[command(subcommand)]
        cmd: AllowlistCommand,
    },
}

#[derive(Subcommand, Debug)]
enum FactoryCommand {
    Info,
    Global,
    Collateral {
        collateral: Address,
    },
    List {
        #[arg(long, default_value_t = 0)]
        from: u64,
        #[arg(long, default_value_t = 10)]
        limit: u64,
    },
    Project {
        project_id: u64,
    },
    Snapshot {
        project_id: u64,
    },
    Commitment {
        project_id: u64,
        user: Address,
    },
    AgentProjects {
        agent_id: u64,
    },
    Create(CreateArgs),
    Approve {
        project_id: u64,
    },
    Revoke {
        project_id: u64,
    },
    UpdateMetadata(UpdateMetadataArgs),
    SetStatus(SetStatusArgs),
    SetCollateral {
        collateral: Address,
        allowed: BoolValue,
    },
    SetGlobal(SetGlobalArgs),
}

#[derive(Args, Debug)]
struct CreateArgs {
    #[arg(long)]
    agent_id: u64,
    #[arg(long)]
    name: String,
    #[arg(long)]
    description: String,
    #[arg(long, default_value = "")]
    categories: String,
    #[arg(long)]
    token_name: String,
    #[arg(long)]
    token_symbol: String,
    #[arg(long)]
    duration_minutes: u64,
    #[arg(long, default_value_t = 0)]
    launch_in_minutes: u64,
    #[arg(long)]
    agent_address: Option<Address>,
    #[arg(long)]
    collateral: Option<Address>,
}

#[derive(Args, Debug)]
struct UpdateMetadataArgs {
    #[arg(long)]
    project_id: u64,
    #[arg(long)]
    description: String,
    #[arg(long, default_value = "")]
    categories: String,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum OperationalStatus {
    Raising,
    Deploying,
    Operating,
    Paused,
    Closed,
}

impl OperationalStatus {
    fn as_u8(self) -> u8 {
        match self {
            Self::Raising => 0,
            Self::Deploying => 1,
            Self::Operating => 2,
            Self::Paused => 3,
            Self::Closed => 4,
        }
    }
}

#[derive(Args, Debug)]
struct SetStatusArgs {
    #[arg(long)]
    project_id: u64,
    #[arg(long, value_enum)]
    status: OperationalStatus,
    #[arg(long, default_value = "")]
    status_note: String,
}

#[derive(Args, Debug)]
struct SetGlobalArgs {
    #[arg(long, help = "18 decimals (e.g. 1000.5)")]
    min_raise: String,
    #[arg(long, help = "18 decimals (e.g. 100000)")]
    max_raise: String,
    #[arg(long)]
    platform_fee_bps: u16,
    #[arg(long)]
    platform_fee_recipient: Address,
    #[arg(long)]
    min_duration_minutes: u64,
    #[arg(long)]
    max_duration_minutes: u64,
    #[arg(long, default_value_t = 0)]
    min_launch_delay_minutes: u64,
    #[arg(long)]
    max_launch_delay_minutes: u64,
}

#[derive(Subcommand, Debug)]
enum SaleCommand {
    Status(SaleTarget),
    Claimable {
        #[command(flatten)]
        target: SaleTarget,
        user: Address,
    },
    Refundable {
        #[command(flatten)]
        target: SaleTarget,
        user: Address,
    },
    Commitment {
        #[command(flatten)]
        target: SaleTarget,
        user: Address,
    },
    ApproveCollateral {
        #[command(flatten)]
        target: SaleTarget,
        amount: String,
        #[arg(long, default_value_t = false)]
        raw: bool,
    },
    Commit {
        #[command(flatten)]
        target: SaleTarget,
        amount: String,
        #[arg(long, default_value_t = false)]
        raw: bool,
    },
    Finalize(SaleTarget),
    Claim(SaleTarget),
    Refund(SaleTarget),
    EmergencyRefund(SaleTarget),
}

#[derive(Args, Clone, Debug)]
struct SaleTarget {
    #[arg(long, conflicts_with = "project_id")]
    sale: Option<Address>,
    #[arg(long, conflicts_with = "sale")]
    project_id: Option<u64>,
}

#[derive(Subcommand, Debug)]
enum AllowlistCommand {
    Info,
    IsAllowed { target: Address },
    Add { target: Address },
    Remove { target: Address },
    TransferAdmin { new_admin: Address },
}

#[derive(Debug, Deserialize)]
struct DeploymentFile {
    network: String,
    #[serde(rename = "chainId")]
    chain_id: u64,
    rpc: String,
    contracts: HashMap<String, String>,
    #[serde(default)]
    external: HashMap<String, String>,
}

#[derive(Debug)]
struct RuntimeConfig {
    network: NetworkName,
    network_label: String,
    chain_id: u64,
    rpc_url: String,
    factory: Address,
    allowlist: Address,
    default_collateral: Option<Address>,
    deployment_path: PathBuf,
}

impl RuntimeConfig {
    fn resolve(cli: &Cli) -> Result<Self> {
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

fn find_repo_root(start: &Path) -> Option<PathBuf> {
    let mut cur = Some(start.to_path_buf());
    while let Some(path) = cur {
        if path.join("backend").join("deployments").is_dir() {
            return Some(path);
        }
        cur = path.parent().map(|p| p.to_path_buf());
    }
    None
}

fn provider(cfg: &RuntimeConfig) -> Result<Provider<Http>> {
    Provider::<Http>::try_from(cfg.rpc_url.as_str())
        .with_context(|| format!("invalid RPC URL: {}", cfg.rpc_url))
}

fn signer_client(cli: &Cli, cfg: &RuntimeConfig) -> Result<Arc<SignerClient>> {
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

fn parse_amount_units(value: &str, decimals: u8) -> Result<U256> {
    let parsed = parse_units(value, usize::from(decimals))
        .with_context(|| format!("invalid amount `{value}` for {decimals} decimals"))?;
    Ok(parsed.into())
}

fn parse_amount_raw(value: &str) -> Result<U256> {
    U256::from_dec_str(value).with_context(|| format!("invalid raw uint256 amount `{value}`"))
}

fn format_token_amount(value: U256, decimals: u8) -> String {
    format_units(value, usize::from(decimals)).unwrap_or_else(|_| value.to_string())
}

fn supports_color() -> bool {
    std::env::var_os("NO_COLOR").is_none()
        && std::env::var("TERM").map(|t| t != "dumb").unwrap_or(true)
}

fn paint(text: impl AsRef<str>, code: &str) -> String {
    let t = text.as_ref();
    if supports_color() {
        format!("\x1b[{code}m{t}\x1b[0m")
    } else {
        t.to_string()
    }
}

fn header(text: impl AsRef<str>) -> String {
    paint(text, "1;36")
}

fn key(text: impl AsRef<str>) -> String {
    paint(text, "1;34")
}

fn warn(text: impl AsRef<str>) -> String {
    paint(text, "1;33")
}

fn format_bps(bps: u16) -> String {
    format!("{bps} ({:.2}%)", f64::from(bps) / 100.0)
}

fn format_duration_human(seconds: U256) -> String {
    let raw = seconds.to_string();
    if seconds > U256::from(u64::MAX) {
        return format!("{raw} sec");
    }
    let mut s = seconds.as_u64();
    let days = s / 86_400;
    s %= 86_400;
    let hours = s / 3_600;
    s %= 3_600;
    let minutes = s / 60;
    let secs = s % 60;

    let mut parts: Vec<String> = Vec::new();
    if days > 0 {
        parts.push(format!("{days}d"));
    }
    if hours > 0 {
        parts.push(format!("{hours}h"));
    }
    if minutes > 0 {
        parts.push(format!("{minutes}m"));
    }
    if secs > 0 || parts.is_empty() {
        parts.push(format!("{secs}s"));
    }
    format!("{raw} sec ({})", parts.join(" "))
}

fn format_status(code: u8) -> &'static str {
    match code {
        0 => "raising",
        1 => "deploying",
        2 => "operating",
        3 => "paused",
        4 => "closed",
        _ => "unknown",
    }
}

fn format_ts(ts: U256) -> String {
    if ts > U256::from(u64::MAX) {
        return ts.to_string();
    }
    format!("{}", ts.as_u64())
}

fn minutes_to_seconds_u256(minutes: u64, label: &str) -> Result<U256> {
    let seconds = minutes
        .checked_mul(60)
        .ok_or_else(|| anyhow!("overflow on {label}"))?;
    Ok(U256::from(seconds))
}

fn unix_now() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock before unix epoch")?
        .as_secs())
}

fn print_receipt(receipt: &TransactionReceipt) {
    println!("tx_hash: {:?}", receipt.transaction_hash);
    println!("block: {}", receipt.block_number.unwrap_or_default());
    println!("status: {}", receipt.status.unwrap_or_default());
}

async fn send_and_wait<D: Detokenize>(
    call: ContractCall<SignerClient, D>,
) -> Result<TransactionReceipt> {
    let pending = call.send().await?;
    pending
        .await?
        .ok_or_else(|| anyhow!("transaction dropped before inclusion"))
}

fn sale_read_abi(signature: &str) -> Result<Abi> {
    AbiParser::default()
        .parse(&[signature])
        .context("cannot parse inline ABI")
}

#[derive(Debug, Clone)]
struct ProjectView {
    agent_id: U256,
    name: String,
    description: String,
    categories: String,
    agent: Address,
    treasury: Address,
    sale: Address,
    agent_executor: Address,
    collateral: Address,
    operational_status: u8,
    status_note: String,
    created_at: U256,
    updated_at: U256,
}

type ProjectTuple = (
    U256,
    String,
    String,
    String,
    Address,
    Address,
    Address,
    Address,
    Address,
    u8,
    String,
    U256,
    U256,
);

async fn read_factory_project<M: Middleware + 'static>(
    client: Arc<M>,
    factory: Address,
    project_id: u64,
) -> Result<ProjectView> {
    let abi = AbiParser::default()
        .parse(&[
            "function getProject(uint256 id) view returns (uint256,string,string,string,address,address,address,address,address,uint8,string,uint256,uint256)"
        ])
        .context("cannot parse getProject ABI")?;
    let contract = Contract::new(factory, abi, client);
    let tuple: ProjectTuple = contract
        .method::<_, ProjectTuple>("getProject", U256::from(project_id))
        .context("cannot build getProject call")?
        .call()
        .await
        .with_context(|| format!("getProject({project_id}) failed"))?;

    Ok(ProjectView {
        agent_id: tuple.0,
        name: tuple.1,
        description: tuple.2,
        categories: tuple.3,
        agent: tuple.4,
        treasury: tuple.5,
        sale: tuple.6,
        agent_executor: tuple.7,
        collateral: tuple.8,
        operational_status: tuple.9,
        status_note: tuple.10,
        created_at: tuple.11,
        updated_at: tuple.12,
    })
}

async fn read_sale_collateral<M: Middleware + 'static>(
    client: Arc<M>,
    sale: Address,
) -> Result<Address> {
    let abi = sale_read_abi("function COLLATERAL() view returns (address)")?;
    let c = Contract::new(sale, abi, client);
    c.method::<_, Address>("COLLATERAL", ())
        .context("cannot build COLLATERAL call")?
        .call()
        .await
        .context("COLLATERAL() call failed")
}

async fn read_sale_project_id<M: Middleware + 'static>(
    client: Arc<M>,
    sale: Address,
) -> Result<U256> {
    let abi = sale_read_abi("function PROJECT_ID() view returns (uint256)")?;
    let c = Contract::new(sale, abi, client);
    c.method::<_, U256>("PROJECT_ID", ())
        .context("cannot build PROJECT_ID call")?
        .call()
        .await
        .context("PROJECT_ID() call failed")
}

async fn send_factory_set_global_config(
    client: Arc<SignerClient>,
    factory: Address,
    config: (U256, U256, u16, Address, U256, U256, U256, U256),
) -> Result<TransactionReceipt> {
    let abi = AbiParser::default()
        .parse(&[
            "function setGlobalConfig((uint256,uint256,uint16,address,uint256,uint256,uint256,uint256) config)"
        ])
        .context("cannot parse setGlobalConfig ABI")?;
    let contract = Contract::new(factory, abi, client);
    let call = contract
        .method::<_, ()>("setGlobalConfig", (config,))
        .context("cannot build setGlobalConfig call")?;
    send_and_wait(call)
        .await
        .context("setGlobalConfig transaction failed to send")
}

async fn token_meta<M: Middleware + 'static>(
    client: Arc<M>,
    token: Address,
) -> Result<(u8, String)> {
    let erc20 = Erc20Metadata::new(token, client);
    let decimals = erc20
        .decimals()
        .call()
        .await
        .context("cannot read token decimals")?;
    let symbol = erc20
        .symbol()
        .call()
        .await
        .unwrap_or_else(|_| "TOKEN".to_string());
    Ok((decimals, symbol))
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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = RuntimeConfig::resolve(&cli)?;

    match &cli.command {
        Command::Network => {
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
        Command::Factory { cmd } => {
            let p = Arc::new(provider(&cfg)?);
            match cmd {
                FactoryCommand::Info => {
                    let factory = AgentRaiseFactoryContract::new(cfg.factory, p.clone());
                    let count = factory.project_count().call().await?;
                    let (
                        min_raise,
                        max_raise,
                        fee_bps,
                        fee_to,
                        min_dur,
                        max_dur,
                        min_delay,
                        max_delay,
                    ) = factory.global_config().call().await?;

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
                                if let Ok((decimals, symbol)) =
                                    token_meta(p.clone(), collateral).await
                                {
                                    let min_c =
                                        factory.min_raise_for_collateral(collateral).call().await?;
                                    let max_c =
                                        factory.max_raise_for_collateral(collateral).call().await?;
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
                }
                FactoryCommand::Global => {
                    let factory = AgentRaiseFactoryContract::new(cfg.factory, p.clone());
                    let (
                        min_raise,
                        max_raise,
                        fee_bps,
                        fee_to,
                        min_dur,
                        max_dur,
                        min_delay,
                        max_delay,
                    ) = factory.global_config().call().await?;
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
                }
                FactoryCommand::Collateral { collateral } => {
                    let factory = AgentRaiseFactoryContract::new(cfg.factory, p.clone());
                    let allowed = match factory.allowed_collateral(*collateral).call().await {
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
                        let (decimals, symbol) = token_meta(p.clone(), *collateral).await?;
                        let min = factory.min_raise_for_collateral(*collateral).call().await?;
                        let max = factory.max_raise_for_collateral(*collateral).call().await?;
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
                }
                FactoryCommand::List { from, limit } => {
                    let factory = AgentRaiseFactoryContract::new(cfg.factory, p.clone());
                    let count_u256 = factory.project_count().call().await?;
                    let count = count_u256.as_u64();
                    let start = (*from).min(count);
                    let end = start.saturating_add(*limit).min(count);
                    println!("projects_total: {}", count);
                    println!("range: {}..{}", start, end);

                    for id in start..end {
                        let project = read_factory_project(p.clone(), cfg.factory, id).await?;
                        let approved = factory.is_project_approved(id.into()).call().await?;
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
                }
                FactoryCommand::Project { project_id } => {
                    let factory = AgentRaiseFactoryContract::new(cfg.factory, p.clone());
                    let project = read_factory_project(p.clone(), cfg.factory, *project_id).await?;
                    let approved = factory
                        .is_project_approved((*project_id).into())
                        .call()
                        .await?;
                    println!("project_id: {}", project_id);
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
                FactoryCommand::Snapshot { project_id } => {
                    let factory = AgentRaiseFactoryContract::new(cfg.factory, p.clone());
                    let snapshot = factory
                        .get_project_raise_snapshot((*project_id).into())
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
                }
                FactoryCommand::Commitment { project_id, user } => {
                    let factory = AgentRaiseFactoryContract::new(cfg.factory, p.clone());
                    let committed = factory
                        .get_project_commitment((*project_id).into(), *user)
                        .call()
                        .await?;
                    println!("project_id: {}", project_id);
                    println!("user: {:?}", user);
                    println!("committed: {}", committed);
                }
                FactoryCommand::AgentProjects { agent_id } => {
                    let factory = AgentRaiseFactoryContract::new(cfg.factory, p.clone());
                    let ids = factory
                        .get_agent_projects((*agent_id).into())
                        .call()
                        .await?;
                    println!("agent_id: {}", agent_id);
                    println!("project_ids: {:?}", ids);
                }
                FactoryCommand::Create(args) => {
                    let client = signer_client(&cli, &cfg)?;
                    let sender = client.address();
                    let factory = AgentRaiseFactoryContract::new(cfg.factory, client.clone());

                    let collateral = args.collateral.or(cfg.default_collateral).ok_or_else(|| {
                        anyhow!(
                            "collateral not set: pass --collateral or configure USDM in deployment file"
                        )
                    })?;
                    let agent_address = args.agent_address.unwrap_or(sender);
                    let duration =
                        minutes_to_seconds_u256(args.duration_minutes, "duration_minutes")?;
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
                    print_receipt(&receipt);
                }
                FactoryCommand::Approve { project_id } => {
                    let client = signer_client(&cli, &cfg)?;
                    let factory = AgentRaiseFactoryContract::new(cfg.factory, client);
                    let call = factory.approve_project((*project_id).into());
                    let receipt = send_and_wait(call).await?;
                    print_receipt(&receipt);
                }
                FactoryCommand::Revoke { project_id } => {
                    let client = signer_client(&cli, &cfg)?;
                    let factory = AgentRaiseFactoryContract::new(cfg.factory, client);
                    let call = factory.revoke_project((*project_id).into());
                    let receipt = send_and_wait(call).await?;
                    print_receipt(&receipt);
                }
                FactoryCommand::UpdateMetadata(args) => {
                    let client = signer_client(&cli, &cfg)?;
                    let factory = AgentRaiseFactoryContract::new(cfg.factory, client);
                    let call = factory.update_project_metadata(
                        args.project_id.into(),
                        args.description.clone(),
                        args.categories.clone(),
                    );
                    let receipt = send_and_wait(call).await?;
                    print_receipt(&receipt);
                }
                FactoryCommand::SetStatus(args) => {
                    let client = signer_client(&cli, &cfg)?;
                    let factory = AgentRaiseFactoryContract::new(cfg.factory, client);
                    let call = factory.update_project_operational_status(
                        args.project_id.into(),
                        args.status.as_u8(),
                        args.status_note.clone(),
                    );
                    let receipt = send_and_wait(call).await?;
                    print_receipt(&receipt);
                }
                FactoryCommand::SetCollateral {
                    collateral,
                    allowed,
                } => {
                    let client = signer_client(&cli, &cfg)?;
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
                    print_receipt(&receipt);
                }
                FactoryCommand::SetGlobal(args) => {
                    let client = signer_client(&cli, &cfg)?;

                    let min_raise = parse_amount_units(&args.min_raise, 18)?;
                    let max_raise = parse_amount_units(&args.max_raise, 18)?;
                    let min_duration =
                        minutes_to_seconds_u256(args.min_duration_minutes, "min_duration_minutes")?;
                    let max_duration =
                        minutes_to_seconds_u256(args.max_duration_minutes, "max_duration_minutes")?;
                    let min_launch_delay = minutes_to_seconds_u256(
                        args.min_launch_delay_minutes,
                        "min_launch_delay_minutes",
                    )?;
                    let max_launch_delay = minutes_to_seconds_u256(
                        args.max_launch_delay_minutes,
                        "max_launch_delay_minutes",
                    )?;

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
                    print_receipt(&receipt);
                }
            }
        }
        Command::Sale { cmd } => {
            let p = Arc::new(provider(&cfg)?);
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
                        if let Ok(approved) = AgentRaiseFactoryContract::new(cfg.factory, p.clone())
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
                    let client = signer_client(&cli, &cfg)?;
                    let owner = client.address();
                    let sale_addr =
                        resolve_sale_address(client.clone(), cfg.factory, target).await?;
                    let collateral = read_sale_collateral(client.clone(), sale_addr).await?;
                    let erc20 = Erc20Metadata::new(collateral, client.clone());
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
                    print_receipt(&receipt);
                }
                SaleCommand::Commit {
                    target,
                    amount,
                    raw,
                } => {
                    let client = signer_client(&cli, &cfg)?;
                    let sender = client.address();
                    let sale_addr =
                        resolve_sale_address(client.clone(), cfg.factory, target).await?;
                    let collateral = read_sale_collateral(client.clone(), sale_addr).await?;
                    let sale = SaleContract::new(sale_addr, client.clone());
                    let erc20 = Erc20Metadata::new(collateral, client.clone());
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
                    print_receipt(&receipt);
                }
                SaleCommand::Finalize(target) => {
                    let client = signer_client(&cli, &cfg)?;
                    let sale_addr =
                        resolve_sale_address(client.clone(), cfg.factory, target).await?;
                    let sale = SaleContract::new(sale_addr, client);
                    let call = sale.finalize();
                    let receipt = send_and_wait(call).await?;
                    print_receipt(&receipt);
                }
                SaleCommand::Claim(target) => {
                    let client = signer_client(&cli, &cfg)?;
                    let sale_addr =
                        resolve_sale_address(client.clone(), cfg.factory, target).await?;
                    let sale = SaleContract::new(sale_addr, client);
                    let call = sale.claim();
                    let receipt = send_and_wait(call).await?;
                    print_receipt(&receipt);
                }
                SaleCommand::Refund(target) => {
                    let client = signer_client(&cli, &cfg)?;
                    let sale_addr =
                        resolve_sale_address(client.clone(), cfg.factory, target).await?;
                    let sale = SaleContract::new(sale_addr, client);
                    let call = sale.refund();
                    let receipt = send_and_wait(call).await?;
                    print_receipt(&receipt);
                }
                SaleCommand::EmergencyRefund(target) => {
                    let client = signer_client(&cli, &cfg)?;
                    let sale_addr =
                        resolve_sale_address(client.clone(), cfg.factory, target).await?;
                    let sale = SaleContract::new(sale_addr, client);
                    let call = sale.emergency_refund();
                    let receipt = send_and_wait(call).await?;
                    print_receipt(&receipt);
                }
            }
        }
        Command::Allowlist { cmd } => {
            let p = Arc::new(provider(&cfg)?);
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
                    let client = signer_client(&cli, &cfg)?;
                    let allow = ContractAllowlistContract::new(cfg.allowlist, client);
                    let call = allow.add_contract(*target);
                    let receipt = send_and_wait(call).await?;
                    print_receipt(&receipt);
                }
                AllowlistCommand::Remove { target } => {
                    let client = signer_client(&cli, &cfg)?;
                    let allow = ContractAllowlistContract::new(cfg.allowlist, client);
                    let call = allow.remove_contract(*target);
                    let receipt = send_and_wait(call).await?;
                    print_receipt(&receipt);
                }
                AllowlistCommand::TransferAdmin { new_admin } => {
                    let client = signer_client(&cli, &cfg)?;
                    let allow = ContractAllowlistContract::new(cfg.allowlist, client);
                    let call = allow.transfer_admin(*new_admin);
                    let receipt = send_and_wait(call).await?;
                    print_receipt(&receipt);
                }
            }
        }
    }

    Ok(())
}
