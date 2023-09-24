use std::env;
use std::str::FromStr;

use common::eth::block::FullBlock;
use common::eth::helper::get_signatures_table;
use common::eth::read_block as eth_read_block;
use common::eth::transfer::{filter_and_decode_block_transactions};
use web3::transports::WebSocket;
use web3::types::{BlockId, H256};
use web3::Web3;
use crate::gosh::proposal::Proposal;

pub async fn validate_proposal(
    web3s: &Web3<WebSocket>,
    proposal: Proposal,
) -> anyhow::Result<()> {
    tracing::info!("Validate proposal: {proposal:?}");
    let from_block = BlockId::Hash(H256::from_str(&proposal.details.hash)?);
    let till_block = BlockId::Hash(H256::from_str(&proposal.details.new_hash)?);
    let mut verifying_transfers = proposal.details.transactions;

    let from_block_num = {
        let FullBlock {number, ..} = eth_read_block(web3s, from_block).await
            .map_err(|e| anyhow::format_err!("Failed to fetch block with proposal hash({from_block:?}): {e}"))?;
        match number {
            Some(num) => num,
            None => anyhow::bail!("Failed to fetch block with proposal hash")
        }
    };

    let till_block_num = {
        let FullBlock {number, ..} = eth_read_block(web3s, till_block).await
            .map_err(|e| anyhow::format_err!("Failed to fetch block with proposal new_hash({till_block:?}): {e}"))?;
        match number {
            Some(num) => num,
            None => anyhow::bail!("Failed to fetch block with proposal new_hash")
        }
    };

    if from_block_num >= till_block_num {
        tracing::info!("Wrong block chain: {from_block_num} >= {till_block_num}");
        anyhow::bail!("Wrong block chain: {from_block_num} >= {till_block_num}");
    }

    let eth_contract_address = env::var("ETH_CONTRACT_ADDRESS")?.to_lowercase();
    let code_sig_lookup = get_signatures_table()?;
    let mut block_id = till_block;
    loop {
        let block = eth_read_block(web3s, block_id).await?;

        if block.number.unwrap() == from_block_num {
            tracing::info!("Reached end block.");
            break;
        }

        let transfers = filter_and_decode_block_transactions(
            web3s,
            &block,
            &eth_contract_address,
            &code_sig_lookup,
        )
            .await?;
        tracing::info!("block transfers: {transfers:?}");
        // Find this block transfers in the list from proposal and check that all block transfers present in proposal
        let (found_transfers, rest_transfers) = verifying_transfers.into_iter().partition(|trans| transfers.contains(trans));
        tracing::info!("found transfers: {transfers:?}");
        if transfers != found_transfers {
            tracing::info!("Not all block transfers were found in proposal.");
            anyhow::bail!("Not all block transfers were found in proposal.");
        }
        verifying_transfers = rest_transfers;

        block_id = BlockId::Hash(block.parent_hash);
    }
    Ok(())
}
