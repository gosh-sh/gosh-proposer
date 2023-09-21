use crate::eth::helper::get_signatures_table;
use crate::eth::transfer::decode_transfer;
use chrono::{DateTime, NaiveDateTime, Utc};
use std::env;
use web3::{helpers as w3h, Transport};
use web3::transports::WebSocket;
use web3::types::{Block, BlockId, BlockNumber, TransactionId, H256, U256, U64};
use web3::Web3;
use crate::eth::block::FullBlock;

pub mod helper;
pub mod proof;
mod transfer;
pub mod block;

pub async fn read_eth_blocks() -> anyhow::Result<()> {
    // Load variables from .env
    // Token contract address
    let eth_contract_address = env::var("ETH_CONTRACT_ADDRESS")?.to_lowercase();

    // Oldest saved block num
    let eth_end_block = U64::from_str_radix(&env::var("ETH_END_BLOCK")?, 10)?;

    // Lookup table of contract methods
    let code_sig_lookup = get_signatures_table()?;

    let websocket = WebSocket::new(&env::var("ETH_NETWORK")?).await?;
    let web3s = Web3::new(websocket);

    // Start from the latest block if nothing was specified in the env
    let mut block_id = match U64::from_str_radix(&env::var("ETH_STARTING_BLOCK")?, 10) {
        Ok(val) => BlockId::Number(BlockNumber::Number(U64::from(val))),
        Err(_) => BlockId::Number(BlockNumber::Latest),
    };

    let mut transfers = vec![];

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

        // Parse block transactions
        for transaction_hash in next_block.transactions {
            // Load transaction
            let tx = match web3s
                .eth()
                .transaction(TransactionId::Hash(transaction_hash))
                .await
            {
                Ok(Some(tx)) => tx,
                _ => {
                    tracing::trace!("Failed to fetch transaction: {transaction_hash}");
                    continue;
                }
            };

            // Check that transaction destination is equal to the specified address
            if let Some(address) = tx.to {
                let dest = w3h::to_string(&address)
                    .trim()
                    .trim_end_matches('"')
                    .trim_start_matches('"')
                    .to_string()
                    .to_lowercase();
                tracing::trace!("Txn destination address: {dest}");
                if dest != eth_contract_address {
                    tracing::trace!(
                        "Wrong destination address, skip it. `{}` != `{eth_contract_address}`",
                        dest
                    );
                    continue;
                }
            } else {
                tracing::trace!("No destination address, skip it.");
                continue;
            }

            match decode_transfer(tx, &code_sig_lookup) {
                Ok(transfer) => transfers.push(transfer),
                Err(_) => {}
            }
        }
    }

    tracing::info!("List of transfers: {transfers:?}");

    Ok(())
}

// Read Ethereum block with specified block id
pub async fn read_block(web3s: &Web3<WebSocket>, block_id: BlockId) -> anyhow::Result<FullBlock<H256>> {
    tracing::info!("Reading block: {block_id:?}");
    let include_txs = w3h::serialize(&false);
    let block = match block_id {
        BlockId::Hash(hash) => {
            let hash = w3h::serialize(&hash);
            web3s.transport().execute("eth_getBlockByHash", vec![hash, include_txs])
        }
        BlockId::Number(num) => {
            let num = w3h::serialize(&num);
            web3s.transport().execute("eth_getBlockByNumber", vec![num, include_txs])
        }
    }.await?;

    tracing::info!("{}", serde_json::to_string_pretty(&block)?);
    Ok(serde_json::from_value(block)?)
}

mod test {
    use super::proof::serialize_block;
    use super::read_block;
    use crate::helper::tracing::init_default_tracing;
    use std::env;
    use web3::transports::WebSocket;
    use web3::types::{BlockId, BlockNumber, U64};
    use web3::Web3;

    #[tokio::test]
    pub async fn test_hash() -> anyhow::Result<()> {
        dotenv::dotenv().ok();
        init_default_tracing();
        let block_id = BlockId::Number(BlockNumber::Number(
            U64::from_str_radix("400000", 10).unwrap(),
        ));
        let websocket = WebSocket::new(&env::var("ETH_NETWORK")?).await?;
        let web3s = Web3::new(websocket);
        let block = read_block(&web3s, block_id).await?;
        serialize_block(block)?;
        Ok(())
    }
}
