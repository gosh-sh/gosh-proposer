use common::eth::encoder::serialize_block;
use common::eth::FullBlock;

use common::elock::transfer::filter_and_decode_block_transactions;
use common::elock::{get_elock_address, get_tx_counter};
use common::gosh::call_function;
use common::gosh::helper::EverClient;
use common::helper::abi::CHECKER_ABI;
use serde_json::json;
use std::sync::Arc;
use web3::transports::WebSocket;
use web3::types::{H256, U64};
use web3::Web3;

pub async fn propose_blocks(
    web3s: Arc<Web3<WebSocket>>,
    client: &EverClient,
    blocks: Vec<FullBlock<H256>>,
    checker_address: &str,
    first_block_number: U64,
) -> anyhow::Result<()> {
    tracing::info!("start propose block");

    // ELock contract address
    let elock_address = get_elock_address()?;

    // Get starting tx counter
    let mut prev_tx_counter = get_tx_counter(&web3s, elock_address, first_block_number)
        .await
        .map_err(|e| anyhow::format_err!("Failed to get env ELock tx counter: {e}"))?;

    let mut all_transfers = vec![];
    let mut json_blocks = vec![];

    for block in blocks.iter().rev() {
        let block_number = block.number.ok_or(anyhow::format_err!(
            "Failed to get block number for {block:?}"
        ))?;
        let cur_tx_counter = get_tx_counter(&web3s, elock_address, block_number)
            .await
            .map_err(|e| anyhow::format_err!("Failed to get env ELock tx counter: {e}"))?;
        tracing::info!("Block number={block_number} prev tx counter={prev_tx_counter}, current counter={cur_tx_counter}");
        let mut transfers = if cur_tx_counter != prev_tx_counter {
            prev_tx_counter = cur_tx_counter;
            filter_and_decode_block_transactions(web3s.clone(), block, elock_address).await?
        } else {
            vec![]
        };
        all_transfers.append(&mut transfers);
        let hash = format!("{:?}", block.hash.unwrap());
        let data = serialize_block(block)
            .map_err(|e| anyhow::format_err!("Failed to serialize ETH block: {e}"))?;
        let data_str = data
            .iter()
            .fold(String::new(), |acc, el| format!("{acc}{:02x}", el));
        json_blocks.push(json!({"data": data_str, "hash": hash}));
    }

    tracing::info!("Send transaction to checker: {all_transfers:?}");
    let args = json!({
        "data": json_blocks,
        "transactions": all_transfers,
    });

    call_function(
        client,
        checker_address,
        CHECKER_ABI,
        None,
        "checkData",
        Some(args),
    )
    .await
    .map_err(|e| anyhow::format_err!("Failed to call GOSH function: {e}"))?;
    Ok(())
}
