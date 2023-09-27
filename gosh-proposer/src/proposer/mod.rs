use crate::proposer::propose::propose_blocks;
use common::eth::read_block;
use common::gosh::call_getter;
use common::gosh::helper::{create_client, EverClient};
use common::helper::deserialize_u128;
use serde::Deserialize;
use std::env;
use std::str::FromStr;
use std::sync::Arc;

use web3::transports::WebSocket;
use web3::types::{BlockId, BlockNumber, H256};
use web3::Web3;

mod propose;

#[derive(Deserialize)]
struct Status {
    prevhash: String,
    #[serde(deserialize_with = "deserialize_u128")]
    #[serde(rename = "index")]
    _index: u128,
}

const CHECKER_ABI_PATH: &str = "contracts/l2/checker.abi.json";
const DEFAULT_MAX_BLOCK_IN_ONE_CHUNK: u64 = 20;

pub async fn get_block_from_checker(client: &EverClient) -> anyhow::Result<H256> {
    tracing::info!("get block from checker");
    let checker_address = env::var("CHECKER_ADDRESS")?;
    let value = call_getter(
        client,
        &checker_address,
        CHECKER_ABI_PATH,
        "getStatus",
        None,
    )
    .await?;
    tracing::info!("getter res: {value}");
    let status: Status = serde_json::from_value(value)?;
    Ok(H256::from_str(&status.prevhash)?)
}

pub async fn propose_eth_blocks() -> anyhow::Result<()> {
    // Load variables from .env
    let websocket = WebSocket::new(&env::var("ETH_NETWORK")?).await?;
    let web3s = Web3::new(websocket);

    let client = create_client()?;
    // Oldest saved block hash
    let first_block_hash = get_block_from_checker(&client).await?;
    let first_block_number = read_block(&web3s, BlockId::Hash(first_block_hash))
        .await?
        .number
        .ok_or(anyhow::format_err!(
            "Failed to read Eth block with hash from GOSH checker: {}",
            web3::helpers::to_string(&first_block_hash)
        ))?;

    // Start from the latest block
    let mut block_id = BlockId::Number(BlockNumber::Latest);

    let last_block_number = read_block(&web3s, block_id)
        .await?
        .number
        .ok_or(anyhow::format_err!("Failed to read latest Eth block"))?;

    let mut block_diff = (last_block_number - first_block_number).as_u64();
    tracing::info!(
        "Number of blocks to latest: {}",
        web3::helpers::to_string(&block_diff)
    );

    let max_blocks = env::var("MAX_BLOCK_IN_ONE_CHUNK")
        .ok()
        .and_then(|s| u64::from_str(&s).ok())
        .unwrap_or(DEFAULT_MAX_BLOCK_IN_ONE_CHUNK);

    if block_diff > max_blocks {
        block_id = BlockId::Number(BlockNumber::Number(first_block_number + max_blocks));
        tracing::info!("Difference in block numbers is too high, send till the block {block_id:?}");
        block_diff = max_blocks;
    }

    let mut blocks = vec![];

    for _ in 0..block_diff {
        // Read block
        let next_block = read_block(&web3s, block_id).await?;

        // Get hash of the previous block
        block_id = BlockId::Hash(next_block.parent_hash);
        blocks.push(next_block);
    }

    // Check that we reached the last saved block break the loop
    assert_eq!(
        blocks.last().unwrap().parent_hash,
        first_block_hash,
        "Wrong last queried block"
    );

    let web3s = Arc::new(web3s);
    propose_blocks(web3s, &client, blocks).await?;

    Ok(())
}
