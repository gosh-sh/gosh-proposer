use std::{env, str::FromStr};
use std::collections::HashMap;
use web3::contract::{Contract, Options};
use web3::transports::WebSocket;
use web3::types::{Address, BlockNumber, H256, U256, U64};
use web3::Web3;
use crate::token_root::eth::get_root_data;
use crate::token_root::RootData;

pub mod deposit;
pub mod transfer;

pub const TOTAL_SUPPLY_INDEX: u8 = 0;
pub const COUNTERS_INDEX: u8 = 1;
const LAST_PROCESSED_BLOCK_INDEX: u8 = 3;

pub async fn get_storage(
    web3s: &Web3<WebSocket>,
    eth_address: Address,
    block_num: U64,
    index: u8,
) -> anyhow::Result<H256> {
    web3s
        .eth()
        .storage(
            eth_address,
            U256::from(index),
            Some(BlockNumber::Number(block_num)),
        )
        .await
        .map_err(|e| anyhow::format_err!("Failed to get ELock storage: {e}"))
}

pub async fn get_tx_counter(
    web3s: &Web3<WebSocket>,
    eth_address: Address,
    block_num: U64,
) -> anyhow::Result<U256> {
    let counters = get_storage(web3s, eth_address, block_num, COUNTERS_INDEX).await?;
    let counters_str = web3::helpers::to_string(&counters)
        .replace('"', "")
        .trim_start_matches("0x")
        .to_string();
    tracing::info!("ELock counters: {counters_str}");
    let res = U256::from_str_radix(&counters_str[32..64], 16)?;
    Ok(res)
}

pub fn get_elock_address() -> anyhow::Result<Address> {
    let eth_contract_address = env::var("ETH_CONTRACT_ADDRESS")
        .map_err(|e| anyhow::format_err!("Failed to get env ETH_CONTRACT_ADDRESS: {e}"))?
        .to_lowercase();
    tracing::info!("ELock address: {eth_contract_address}");
    Address::from_str(&eth_contract_address)
        .map_err(|e| anyhow::format_err!("Failed to convert eth address: {e}"))
}

pub async fn get_last_gosh_block_id(
    elock_address: Address,
    web3s: &Web3<WebSocket>,
) -> anyhow::Result<String> {
    let last_gosh_block = web3s
        .eth()
        .storage(elock_address, U256::from(LAST_PROCESSED_BLOCK_INDEX), None)
        .await
        .map_err(|e| anyhow::format_err!("Failed to get ETH contract storage value: {e}"))?;

    let res = web3::helpers::to_string(&last_gosh_block)
        .replace('"', "")
        .trim_start_matches("0x")
        .to_string();
    tracing::info!("last gosh block from ELock: {res}");
    Ok(res)
}

pub async fn get_total_supplies(
    web3s: &Web3<WebSocket>,
    elock_contract: &Contract<WebSocket>,
) -> anyhow::Result<HashMap<RootData, u128>> {
    let mut res = HashMap::new();
    let token_roots: Vec<Address> = elock_contract
        .query("getTokenRoots", (), None, Options::default(), None)
        .await
        .map_err(|e| anyhow::format_err!("Failed to call ELock getter getTokenRoots: {e}"))?;

    for root in token_roots {
        let root_data = get_root_data(
            web3s,
            root,
        ).await?;
        let value: U256 = elock_contract.query(
            "getTotalSupply",
            root,
            None,
            Options::default(),
            None,
        ).await
            .map_err(|e| anyhow::format_err!("Failed to call ELock getter getTotalSupply: {e}"))?;
        res.insert(root_data, value.as_u128());
    }
    Ok(res)
}