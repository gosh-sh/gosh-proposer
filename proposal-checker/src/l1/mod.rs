use std::str::FromStr;

use common::eth::block::FullBlock;
use common::eth::helper::get_signatures_table;
use common::eth::read_block as eth_read_block;
use common::eth::transfer::{decode_transfer, TransferProposal};
use web3::transports::WebSocket;
use web3::types::{BlockId, BlockNumber, H256, TransactionId};
use web3::Web3;

pub async fn validate_transaction(
    web3s: &Web3<WebSocket>,
    proposal: &TransferProposal,
) -> anyhow::Result<bool> {
    let from_block = proposal.from_block;
    let till_block = proposal.till_block;
    let verifying_xfer = &proposal.xfer;
    let from_block_num = match from_block {
        BlockId::Number(BlockNumber::Number(num)) => num,
        BlockId::Number(_) => anyhow::bail!("Can't get block number"),
        BlockId::Hash(_) => {
            let FullBlock {number, ..} = eth_read_block(web3s, from_block).await?;
            match number {
                Some(num) => num,
                None => anyhow::bail!("Can't get block number")
            }
        }
    };
    let till_block_num = match till_block {
        BlockId::Number(BlockNumber::Number(num)) => num,
        BlockId::Number(_) => anyhow::bail!("Can't get block number"),
        BlockId::Hash(_) => {
            let FullBlock {number, ..} = eth_read_block(web3s, till_block).await?;
            match number {
                Some(num) => num,
                None => anyhow::bail!("Can't get block number")
            }
        }
    };

    let txn_hash = H256::from_str(&verifying_xfer.hash)?;
    let txn = web3s.eth()
        .transaction(TransactionId::Hash(txn_hash))
        .await?
        .unwrap();

    let txn_owning_block = txn.block_number.unwrap();
    if txn_owning_block < from_block_num || txn_owning_block > till_block_num {
        return Ok(false)
    }

    let elock_fn_signatures = get_signatures_table()?;

    let transfer = match decode_transfer(txn, &elock_fn_signatures) {
        Ok(transfer) => Some(transfer),
        Err(_) => None
    };

    match transfer {
        Some(xfer) => Ok(*verifying_xfer == xfer),
        None => Ok(false)
    }
}
