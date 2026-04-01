use clap::{Args, Parser, Subcommand, ValueEnum};
use ethers::prelude::Address;

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum NetworkName {
    Testnet,
    Mainnet,
}

impl NetworkName {
    pub fn deployment_file(self) -> &'static str {
        match self {
            Self::Testnet => "megaeth-testnet.json",
            Self::Mainnet => "megaeth-mainnet.json",
        }
    }
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum BoolValue {
    True,
    False,
}

impl BoolValue {
    pub fn as_bool(self) -> bool {
        matches!(self, Self::True)
    }
}

#[derive(Parser, Debug)]
#[command(name = "backed", version, about = "Backed protocol CLI")]
pub struct Cli {
    #[arg(long, value_enum, default_value = "testnet", env = "BACKED_NETWORK")]
    pub network: NetworkName,
    #[arg(long, env = "BACKED_RPC_URL")]
    pub rpc_url: Option<String>,
    #[arg(long, env = "BACKED_FACTORY")]
    pub factory: Option<Address>,
    #[arg(long, env = "BACKED_ALLOWLIST")]
    pub allowlist: Option<Address>,
    #[arg(long, env = "BACKED_PRIVATE_KEY")]
    pub private_key: Option<String>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
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
pub enum FactoryCommand {
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

#[derive(Subcommand, Debug)]
pub enum SaleCommand {
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

#[derive(Subcommand, Debug)]
pub enum AllowlistCommand {
    Info,
    IsAllowed { target: Address },
    Add { target: Address },
    Remove { target: Address },
    TransferAdmin { new_admin: Address },
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    #[arg(long)]
    pub agent_id: u64,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub description: String,
    #[arg(long, default_value = "")]
    pub categories: String,
    #[arg(long)]
    pub token_name: String,
    #[arg(long)]
    pub token_symbol: String,
    #[arg(long)]
    pub duration_minutes: u64,
    #[arg(long, default_value_t = 0)]
    pub launch_in_minutes: u64,
    #[arg(long)]
    pub agent_address: Option<Address>,
    #[arg(long)]
    pub collateral: Option<Address>,
}

#[derive(Args, Debug)]
pub struct UpdateMetadataArgs {
    #[arg(long)]
    pub project_id: u64,
    #[arg(long)]
    pub description: String,
    #[arg(long, default_value = "")]
    pub categories: String,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum OperationalStatus {
    Raising,
    Deploying,
    Operating,
    Paused,
    Closed,
}

impl OperationalStatus {
    pub fn as_u8(self) -> u8 {
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
pub struct SetStatusArgs {
    #[arg(long)]
    pub project_id: u64,
    #[arg(long, value_enum)]
    pub status: OperationalStatus,
    #[arg(long, default_value = "")]
    pub status_note: String,
}

#[derive(Args, Debug)]
pub struct SetGlobalArgs {
    #[arg(long, help = "18 decimals (e.g. 1000.5)")]
    pub min_raise: String,
    #[arg(long, help = "18 decimals (e.g. 100000)")]
    pub max_raise: String,
    #[arg(long)]
    pub platform_fee_bps: u16,
    #[arg(long)]
    pub platform_fee_recipient: Address,
    #[arg(long)]
    pub min_duration_minutes: u64,
    #[arg(long)]
    pub max_duration_minutes: u64,
    #[arg(long, default_value_t = 0)]
    pub min_launch_delay_minutes: u64,
    #[arg(long)]
    pub max_launch_delay_minutes: u64,
}

#[derive(Args, Clone, Debug)]
pub struct SaleTarget {
    #[arg(long, conflicts_with = "project_id")]
    pub sale: Option<Address>,
    #[arg(long, conflicts_with = "sale")]
    pub project_id: Option<u64>,
}
