use std::env;
use std::str::FromStr;

use crate::gosh::proposal::Proposal;
use common::eth::block::FullBlock;
use common::eth::helper::get_signatures_table;
use common::eth::read_block as eth_read_block;
use common::eth::transfer::decode_transfer;
use web3::transports::WebSocket;
use web3::types::{Address, BlockId, BlockNumber, TransactionId, H256, U256, U64};
use web3::Web3;

const COUNTERS_INDEX: u8 = 1;

async fn get_tx_counter(
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
    tracing::info!("counters: {counters_str}");
    let res = U256::from_str_radix(&counters_str[33..64], 16)?;
    Ok(res)
}

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
    let eth_contract_address = env::var("ETH_CONTRACT_ADDRESS")
        .map_err(|e| anyhow::format_err!("Failed to get env ETH_CONTRACT_ADDRESS: {e}"))?
        .to_lowercase();
    let eth_address = Address::from_str(&eth_contract_address)
        .map_err(|e| anyhow::format_err!("Failed to convert eth address: {e}"))?;

    let from_block_num = {
        let FullBlock { number, .. } = eth_read_block(web3s, from_block).await.map_err(|e| {
            anyhow::format_err!("Failed to fetch block with proposal hash({from_block:?}): {e}")
        })?;
        match number {
            Some(num) => num,
            None => anyhow::bail!("Failed to fetch block with proposal hash"),
        }
    };
    let start_tx_counter = get_tx_counter(web3s, eth_address, from_block_num)
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
    let end_tx_counter = get_tx_counter(web3s, eth_address, till_block_num)
        .await
        .map_err(|e| anyhow::format_err!("Failed to get env ELock tx counter: {e}"))?;

    if from_block_num >= till_block_num {
        anyhow::bail!("Wrong block chain: {from_block_num} >= {till_block_num}");
    }

    tracing::info!("end_tx_counter={end_tx_counter} start_tx_counter={start_tx_counter} ");
    let tx_cnt = (end_tx_counter - start_tx_counter).as_usize();
    if verifying_transfers.len() != tx_cnt {
        anyhow::bail!("Number of transfers in prpposal is not equal to tx counter change");
    }

    let code_sig_lookup = get_signatures_table()
        .map_err(|e| anyhow::format_err!("Failed to get signatures table: {e}"))?;

    for transfer in verifying_transfers {
        let tx = match web3s
            .eth()
            .transaction(TransactionId::Hash(H256::from_str(&transfer.hash)?))
            .await
        {
            Ok(Some(tx)) => tx,
            _ => {
                anyhow::bail!("Failed to fetch transaction: {}", transfer.hash);
            }
        };
        let actual_transfer = decode_transfer(tx, &code_sig_lookup)
            .map_err(|e| anyhow::format_err!("Failed to decode transfer: {e}"))?;
        if transfer != actual_transfer {
            tracing::info!("{:?} != {:?}", transfer, actual_transfer);
            anyhow::bail!("Fetched transaction is not equal to the one in proposal.")
        }
    }

    Ok(())
}
