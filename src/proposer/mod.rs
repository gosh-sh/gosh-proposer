use crate::eth::read_block;
use crate::proposer::propose::propose_blocks;
use std::env;
use web3::transports::WebSocket;
use web3::types::{BlockId, BlockNumber, U64};
use web3::Web3;

mod propose;

pub async fn propose_eth_blocks() -> anyhow::Result<()> {
    // Load variables from .env
    let websocket = WebSocket::new(&env::var("ETH_NETWORK")?).await?;
    let web3s = Web3::new(websocket);

    // Oldest saved block num
    // TODO: change to query from GLOCK contract
    let eth_end_block = U64::from_str_radix(&env::var("ETH_END_BLOCK")?, 10)?;

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
        if next_block.number.unwrap() == eth_end_block {
            tracing::info!("Reached end block.");
            break;
        }

        // Get hash of the previous block
        block_id = BlockId::Hash(next_block.parent_hash);
        blocks.push(next_block);
    }

    propose_blocks(&web3s, blocks).await?;

    Ok(())
}
