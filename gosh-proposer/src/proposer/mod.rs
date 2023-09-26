use crate::proposer::propose::propose_blocks;
use common::eth::read_block;
use std::env;
use web3::transports::WebSocket;
use web3::types::{BlockId, BlockNumber, H256, U64};
use web3::Web3;
use common::gosh::call_getter;
use common::gosh::helper::{create_client, EverClient};
use common::helper::deserialize_u128;
use std::str::FromStr;
use serde::Deserialize;

mod propose;

#[derive(Deserialize)]
struct Status {
    prevhash: String,
    #[serde(deserialize_with = "deserialize_u128")]
    #[serde(rename = "index")]
    _index: u128,
}

const CHECKER_ABI_PATH: &str = "contracts/l2/checker.abi.json";

pub async fn get_block_from_checker(
    client: &EverClient
) -> anyhow::Result<H256> {
    tracing::info!("get block from checker");
    let checker_address = env::var("CHECKER_ADDRESS")?;
    let value = call_getter(
        client,
        &checker_address,
        CHECKER_ABI_PATH,
        "getStatus",
        None,
    ).await?;
    tracing::info!("getter res: {value}");
    let status: Status = serde_json::from_value(value)?;
    Ok(H256::from_str(&status.prevhash)?)
}

pub async fn propose_eth_blocks() -> anyhow::Result<()> {
    // Load variables from .env
    let websocket = WebSocket::new(&env::var("ETH_NETWORK")?).await?;
    let web3s = Web3::new(websocket);

    let client = create_client()?;
    // Oldest saved block num
    // TODO: change to query from GLOCK contract
    // let eth_end_block = U64::from_str_radix(&env::var("ETH_END_BLOCK")?, 10)?;
    let eth_end_block = get_block_from_checker(&client).await?;


    // Start from the latest block if nothing was specified in the env
    let mut block_id = match U64::from_str_radix(&env::var("ETH_STARTING_BLOCK")?, 10) {
        Ok(val) => BlockId::Number(BlockNumber::Number(val)),
        Err(_) => BlockId::Number(BlockNumber::Latest),
    };

    let mut blocks = vec![];
    loop {
        // Read block
        let next_block = read_block(&web3s, block_id).await?;

        // If we reached the last saved block break the loop
        if next_block.hash.unwrap() == eth_end_block {
            tracing::info!("Reached end block.");
            break;
        }

        // Get hash of the previous block
        block_id = BlockId::Hash(next_block.parent_hash);
        blocks.push(next_block);
    }

    propose_blocks(&web3s, &client, blocks).await?;

    Ok(())
}
