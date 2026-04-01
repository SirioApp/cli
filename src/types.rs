use ethers::prelude::{Address, U256};

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
