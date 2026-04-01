use anyhow::{Context, Result, anyhow};
use ethers::abi::Detokenize;
use ethers::contract::builders::ContractCall;
use ethers::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::contracts::SignerClient;

pub fn parse_amount_units(value: &str, decimals: u8) -> Result<U256> {
    let parsed = ethers::utils::parse_units(value, usize::from(decimals))
        .with_context(|| format!("invalid amount `{value}` for {decimals} decimals"))?;
    Ok(parsed.into())
}

pub fn parse_amount_raw(value: &str) -> Result<U256> {
    U256::from_dec_str(value).with_context(|| format!("invalid raw uint256 amount `{value}`"))
}

pub fn minutes_to_seconds_u256(minutes: u64, label: &str) -> Result<U256> {
    let seconds = minutes
        .checked_mul(60)
        .ok_or_else(|| anyhow!("overflow on {label}"))?;
    Ok(U256::from(seconds))
}

pub fn unix_now() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock before unix epoch")?
        .as_secs())
}

pub async fn send_and_wait<D: Detokenize>(
    call: ContractCall<SignerClient, D>,
) -> Result<TransactionReceipt> {
    let pending = call.send().await?;
    pending
        .await?
        .ok_or_else(|| anyhow!("transaction dropped before inclusion"))
}

pub fn print_receipt(receipt: &TransactionReceipt) {
    println!("tx_hash: {:?}", receipt.transaction_hash);
    println!("block: {}", receipt.block_number.unwrap_or_default());
    println!("status: {}", receipt.status.unwrap_or_default());
}
