use crate::proposer::propose::propose_blocks;
use common::eth::{create_web3_socket, read_block};
use common::gosh::helper::create_client;
use std::env;
use std::str::FromStr;
use std::sync::Arc;

use common::checker::{get_block_from_checker, get_checker_address};
use web3::types::{BlockId, BlockNumber};

mod propose;

const DEFAULT_MAX_BLOCK_IN_ONE_CHUNK: u64 = 20;

pub async fn propose_eth_blocks() -> anyhow::Result<()> {
    // Create client for ETH
    let web3s = create_web3_socket().await?;

    // Create client for GOSH
    let client = create_client()?;

    // Get checker address
    let checker_address = get_checker_address()?;

    // Get oldest saved block hash from GOSH checker
    let first_block_hash = get_block_from_checker(&client, &checker_address).await?;
    let first_block_number = read_block(&web3s, BlockId::Hash(first_block_hash))
        .await?
        .number
        .ok_or(anyhow::format_err!(
            "Failed to read Eth block with hash from GOSH checker: {}",
            web3::helpers::to_string(&first_block_hash)
        ))?;

    tracing::info!("Saved block number: {}", first_block_number.as_u64());

    // Get the latest GOSH block
    let mut block_id = BlockId::Number(BlockNumber::Finalized);
    let last_block_number = read_block(&web3s, block_id)
        .await?
        .number
        .ok_or(anyhow::format_err!("Failed to read latest Eth block"))?;
    tracing::info!("Last block number: {}", last_block_number.as_u64());

    // exit if the latest ETH block is already set
    if last_block_number <= first_block_number {
        anyhow::bail!("Saved block in GOSH is newer than queried finalized block. {last_block_number} <= {first_block_number}");
    }

    let mut block_diff = (last_block_number - first_block_number).as_u64();
    tracing::info!(
        "Number of blocks to latest: {}",
        web3::helpers::to_string(&block_diff)
    );

    // Get maximum block amount for one message
    let max_blocks = env::var("MAX_BLOCK_IN_ONE_CHUNK")
        .ok()
        .and_then(|s| u64::from_str(&s).ok())
        .unwrap_or(DEFAULT_MAX_BLOCK_IN_ONE_CHUNK);

    // If current distance to the latest block is too great, send a maximal batch
    if block_diff > max_blocks {
        block_id = BlockId::Number(BlockNumber::Number(first_block_number + max_blocks));
        tracing::info!("Difference in block numbers is too high, send till the block {block_id:?}");
        block_diff = max_blocks;
    }

    // Query blocks
    let mut blocks = vec![];
    for _ in 0..block_diff {
        // Read block
        let next_block = read_block(&web3s, block_id).await?;

        // Get hash of the previous block
        block_id = BlockId::Hash(next_block.parent_hash);
        blocks.push(next_block);
    }

    // Check that we reached the last saved block
    assert_eq!(
        blocks.last().unwrap().parent_hash,
        first_block_hash,
        "Wrong last queried block"
    );

    // get transfers for queried blocks and propose them
    let web3s = Arc::new(web3s);
    propose_blocks(web3s, &client, blocks, &checker_address, first_block_number).await?;

    Ok(())
}
