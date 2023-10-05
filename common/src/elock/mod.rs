use std::{env, str::FromStr};
use web3::transports::WebSocket;
use web3::types::{Address, BlockNumber, U256, U64};
use web3::Web3;

pub mod transfer;

const COUNTERS_INDEX: u8 = 1;
const LAST_PROCESSED_BLOCK_INDEX: u8 = 3;

pub async fn get_tx_counter(
    web3s: &Web3<WebSocket>,
    eth_address: Address,
    block_num: U64,
) -> anyhow::Result<U256> {
    let counters = web3s
        .eth()
        .storage(
            eth_address,
            U256::from(COUNTERS_INDEX),
            Some(BlockNumber::Number(block_num)),
        )
        .await?;
    let counters_str = web3::helpers::to_string(&counters)
        .replace('"', "")
        .trim_start_matches("0x")
        .to_string();
    tracing::info!("ELock counters: {counters_str}");
    let res = U256::from_str_radix(&counters_str[33..64], 16)?;
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
