use std::str::FromStr;

use crate::deposit::proposal::Proposal;
use common::elock::deposit::get_deposits;
use common::elock::{get_elock_address, get_tx_counter};
use common::eth::read_block as eth_read_block;
use common::eth::FullBlock;
use web3::transports::WebSocket;
use web3::types::{BlockId, H256};
use web3::Web3;

pub async fn validate_proposal(web3s: &Web3<WebSocket>, proposal: Proposal) -> anyhow::Result<()> {
    tracing::info!("Validate proposal: {proposal:?}");
    let from_block = BlockId::Hash(
        H256::from_str(&proposal.details.hash)
            .map_err(|e| anyhow::format_err!("Failed to convert proposal from block: {e}"))?,
    );
    let till_block = BlockId::Hash(
        H256::from_str(&proposal.details.new_hash)
            .map_err(|e| anyhow::format_err!("Failed to convert proposal from block: {e}"))?,
    );
    let verifying_transfers = proposal.details.transactions;
    let elock_address = get_elock_address()?;

    // Get block numbers for block range from proposal and query tx counters on this blocks
    let from_block_num = {
        let FullBlock { number, .. } = eth_read_block(web3s, from_block).await.map_err(|e| {
            anyhow::format_err!("Failed to fetch block with proposal hash({from_block:?}): {e}")
        })?;
        match number {
            Some(num) => num,
            None => anyhow::bail!("Failed to fetch block with proposal hash"),
        }
    };
    let from_block_num = from_block_num + 1;
    let start_tx_counter = get_tx_counter(web3s, elock_address, from_block_num)
        .await
        .map_err(|e| anyhow::format_err!("Failed to get env ELock tx counter: {e}"))?;

    let till_block_num = {
        let FullBlock { number, .. } = eth_read_block(web3s, till_block).await.map_err(|e| {
            anyhow::format_err!("Failed to fetch block with proposal new_hash({till_block:?}): {e}")
        })?;
        match number {
            Some(num) => num,
            None => anyhow::bail!("Failed to fetch block with proposal new_hash"),
        }
    };
    let end_tx_counter = get_tx_counter(web3s, elock_address, till_block_num)
        .await
        .map_err(|e| anyhow::format_err!("Failed to get env ELock tx counter: {e}"))?;

    if from_block_num >= till_block_num {
        anyhow::bail!("Wrong chain of blocks: {from_block_num} >= {till_block_num}");
    }

    tracing::info!("end_tx_counter={end_tx_counter} start_tx_counter={start_tx_counter} ");
    let tx_cnt = (end_tx_counter - start_tx_counter).as_usize();
    if verifying_transfers.len() != tx_cnt {
        anyhow::bail!("Number of transfers in proposal is not equal to tx counter change");
    }

    // Get real deposits and compare them to transfers from proposal
    let actual_deposits =
        get_deposits(web3s, elock_address, from_block_num, till_block_num).await?;
    if actual_deposits != verifying_transfers {
        anyhow::bail!("Actual transfers do not match proposed: {actual_deposits:?} != {verifying_transfers:?}");
    }

    Ok(())
}
