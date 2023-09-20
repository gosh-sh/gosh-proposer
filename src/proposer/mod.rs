use crate::eth::read_block;
use crate::proposer::checker::check_blocks;
use std::env;
use web3::transports::WebSocket;
use web3::types::{BlockId, BlockNumber, U64};
use web3::Web3;

mod checker;

pub async fn check_eth_blocks() -> anyhow::Result<()> {
    // Load variables from .env
    // Oldest saved block num
    let eth_end_block = U64::from_str_radix(&env::var("ETH_END_BLOCK")?, 10)?;

    let websocket = WebSocket::new(&env::var("ETH_NETWORK")?).await?;
    let web3s = Web3::new(websocket);

    // Start from the latest block if nothing was specified in the env
    let mut block_id = match U64::from_str_radix(&env::var("ETH_STARTING_BLOCK")?, 10) {
        Ok(val) => BlockId::Number(BlockNumber::Number(U64::from(val))),
        Err(_) => BlockId::Number(BlockNumber::Latest),
    };
    let mut blocks = vec![];
    loop {
        // Read block
        let next_block = read_block(&web3s, block_id).await?;

        // If we reached the last saved block break the loop
        if next_block.number.unwrap() == eth_end_block {
            println!("Reached end block.");
            break;
        }

        // Get hash of the previous block
        block_id = BlockId::Hash(next_block.parent_hash);
        blocks.push(next_block);
    }

    check_blocks(blocks).await?;

    Ok(())
}
