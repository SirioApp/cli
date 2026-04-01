use crate::util::send_and_wait;
use anyhow::{Context, Result};
use ethers::abi::{Abi, AbiParser};
use ethers::prelude::*;
use std::sync::Arc;

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

pub type SignerClient = SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>;

#[derive(Debug, Clone)]
pub struct ProjectView {
    pub agent_id: U256,
    pub name: String,
    pub description: String,
    pub categories: String,
    pub agent: Address,
    pub treasury: Address,
    pub sale: Address,
    pub agent_executor: Address,
    pub collateral: Address,
    pub operational_status: u8,
    pub status_note: String,
    pub created_at: U256,
    pub updated_at: U256,
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

pub async fn read_factory_project<M: Middleware + 'static>(
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

pub async fn read_sale_collateral<M: Middleware + 'static>(
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

pub async fn read_sale_project_id<M: Middleware + 'static>(
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

pub async fn token_meta<M: Middleware + 'static>(
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

pub async fn send_factory_set_global_config(
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

fn sale_read_abi(signature: &str) -> Result<Abi> {
    AbiParser::default()
        .parse(&[signature])
        .context("cannot parse inline ABI")
}
