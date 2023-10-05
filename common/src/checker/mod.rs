use crate::gosh::call_getter;
use crate::gosh::helper::EverClient;
use crate::helper::abi::CHECKER_ABI;
use crate::helper::deserialize_u128;
use serde::Deserialize;
use std::env;
use std::str::FromStr;
use web3::types::H256;

#[derive(Deserialize)]
struct Status {
    prevhash: String,
    #[serde(deserialize_with = "deserialize_u128")]
    #[serde(rename = "index")]
    _index: u128,
}

pub async fn get_block_from_checker(
    client: &EverClient,
    checker_address: &str,
) -> anyhow::Result<H256> {
    tracing::info!("get last ETH block from checker {checker_address}");
    let status: Status =
        call_getter(client, checker_address, CHECKER_ABI, "getStatus", None).await?;

    H256::from_str(&status.prevhash)
        .map_err(|e| anyhow::format_err!("Failed to convert prev hash: {e}"))
}

pub fn get_checker_address() -> anyhow::Result<String> {
    let address = env::var("CHECKER_ADDRESS")
        .map_err(|e| anyhow::format_err!("Failed to get env CHECKER_ADDRESS: {e}"))?;
    tracing::info!("Checker address: {address}");
    Ok(address)
}

pub fn get_root_address() -> anyhow::Result<String> {
    let root_address = env::var("ROOT_ADDRESS")
        .map_err(|e| anyhow::format_err!("Failed to get env ROOT_ADDRESS: {e}"))?;
    tracing::info!("Root address: {root_address}");
    Ok(root_address)
}
