use std::str::FromStr;
use web3::transports::WebSocket;
use web3::types::{Address, U256};
use web3::Web3;

// uint256 public lastProcessedBlock; // 0x4
const LAST_PROCESSED_BLOCK_INDEX: u8 = 4;

pub async fn get_last_gosh_block_id(
    elock_address: &str,
    web3s: &Web3<WebSocket>,
) -> anyhow::Result<String> {
    let address = Address::from_str(elock_address)?;

    let last_gosh_block = web3s
        .eth()
        .storage(address, U256::from(LAST_PROCESSED_BLOCK_INDEX), None)
        .await?;

    let res = web3::helpers::to_string(&last_gosh_block)
        .replace('"', "")
        .trim_start_matches("0x")
        .to_string();
    tracing::info!("last gosh block: {res}");
    Ok(res)
}
