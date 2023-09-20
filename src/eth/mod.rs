use crate::eth::helper::get_signatures_table;
use crate::eth::transfer::decode_transfer;
use chrono::{DateTime, NaiveDateTime, Utc};
use std::env;
use web3::helpers as w3h;
use web3::transports::WebSocket;
use web3::types::{Block, BlockId, BlockNumber, TransactionId, H256, U256, U64};
use web3::Web3;

pub mod helper;
pub mod proof;
mod transfer;

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
pub async fn read_block(web3s: &Web3<WebSocket>, block_id: BlockId) -> anyhow::Result<Block<H256>> {
    let block = web3s
        .eth()
        .block(block_id)
        .await
        .and_then(|val| Ok(val.unwrap()))?;

    let timestamp = block.timestamp.as_u64() as i64;
    let naive = NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
    let utc_dt: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);

    tracing::info!(
        "[{}] block num {}, block hash {}, parent {}, transactions: {}, gas used {}, gas limit {}, base fee {}, difficulty {}, total difficulty {}",
        utc_dt.format("%Y-%m-%d %H:%M:%S"),
        block.number.unwrap(),
        block.hash.unwrap(),
        block.parent_hash,
        block.transactions.len(),
        block.gas_used,
        block.gas_limit,
        block.base_fee_per_gas.unwrap_or(U256::from(0)),
        block.difficulty,
        block.total_difficulty.unwrap_or(U256::from(0))
    );
    Ok(block)
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
