use std::env;
use chrono::{DateTime, NaiveDateTime, Utc};
use web3::contract::{Contract, Options};
use web3::transports::WebSocket;
use web3::types::{Block, BlockId, BlockNumber, H160, H256, TransactionId, U64};
use web3::Web3;
use web3::helpers as w3h;
use crate::eth::helper::{get_signatures_table, wei_to_eth};

mod helper;


const INPUT_CHUNK_SIZE: usize = 64; // Number of bytes in one function argument
const ADDRESS_PREFIX_SIZE: usize = 24; // Number of leading zeros in address argument
const DEFAULT_DENOMINATOR: u128 = 100;

pub async fn read_eth_blocks() -> anyhow::Result<()> {
    // Load variables from .env
    // Token contract address
    let eth_contract_address = env::var("ETH_CONTRACT_ADDRESS").unwrap().to_lowercase();

    // Token name
    let eth_token_name = env::var("ETH_TOKEN_NAME").unwrap();

    // Oldest saved block num
    let eth_end_block = U64::from_str_radix(&env::var("ETH_STARTING_BLOCK").unwrap(), 10).unwrap();

    // Lookup table of contract methods
    let code_sig_lookup = get_signatures_table()?;

    let websocket = WebSocket::new(&env::var("ETH_NETWORK").unwrap())
        .await
        .unwrap();
    let web3s = Web3::new(websocket);

    // Start from the latest block
    // let mut block_id = BlockId::Number(BlockNumber::Latest);
    let mut block_id = BlockId::Number(BlockNumber::Number(U64::from(4319836)));

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
                let dest = w3h::to_string(&address).trim().trim_end_matches('"').trim_start_matches('"').to_string().to_lowercase();
                tracing::trace!("Txn destination address: {dest}");
                if  dest != eth_contract_address {
                    tracing::trace!("Wrong destination address, skip it. `{}` != `{eth_contract_address}`", dest);
                    continue;
                }
            } else {
                tracing::trace!("No destination address, skip it.");
                continue;
            }

            // Check that contract contains code and that token name is equal to the specified
            // TODO: think of necessity of this checks. mb remove
            let token_name = if false {
                let smart_contract_addr = match tx.to {
                    Some(addr) => match web3s.eth().code(addr, None).await {
                        Ok(code) => {
                            if code == web3::types::Bytes::from([]) {
                                tracing::trace!("Empty code, skipping.");
                                continue;
                            } else {
                                addr
                            }
                        }
                        _ => {
                            tracing::trace!("Unable to retrieve code, skipping.");
                            continue;
                        }
                    },
                    _ => {
                        tracing::trace!("Destination address is not a valid address, skipping.");
                        continue;
                    }
                };


                let smart_contract = match Contract::from_json(
                    web3s.eth(),
                    smart_contract_addr,
                    include_bytes!("../../resources/elock.abi.json"),
                ) {
                    Ok(contract) => contract,
                    _ => {
                        tracing::trace!("Failed to init contract, skipping.");
                        continue;
                    }
                };

                let token_name: String = match smart_contract
                    .query("name", (), None, Options::default(), None)
                    .await
                {
                    Ok(result) => result,
                    _ => {
                        tracing::trace!("Could not get token name, skipping.");
                        continue;
                    }
                };

                if token_name != eth_token_name {
                    tracing::trace!("Wrong token name, skip it. {token_name} != {eth_token_name}");
                    continue;
                }
                token_name
            } else {
                eth_token_name.clone()
            };

            // Decode function call
            let input_str: String = w3h::to_string(&tx.input);
            if input_str.len() < 12 {
                continue;
            }
            tracing::trace!("{} input_str: {input_str}", w3h::to_string(&tx.hash));
            let func_code = input_str[3..11].to_string();
            let func_signature: String = match code_sig_lookup.get(&func_code) {
                Some(func_sig) => format!("{:?}", func_sig),
                _ => {
                    tracing::trace!("Function not found.");
                    "[unknown]".to_string()
                }
            };

            let chunks = input_str[11..]
                .chars()
                .collect::<Vec<char>>()
                .chunks(INPUT_CHUNK_SIZE)
                .map(|c| c.iter().collect::<String>())
                .collect::<Vec<String>>();

            let destination_address = match chunks.get(0) {
                Some(chars) => {
                    if chars.len() == INPUT_CHUNK_SIZE {
                        let mut address = chars.clone();
                        address.drain(0..ADDRESS_PREFIX_SIZE);
                        format!("0x{address}")
                    } else {
                        String::from("Unknown")
                    }
                },
                None => {
                    tracing::trace!("Failed to decode transfer destination");
                    String::from("Unknown")
                }
            };

            let token_value = match chunks.get(1) {
                Some(chars) => {
                    if chars.len() == INPUT_CHUNK_SIZE {
                        chars.clone()
                    } else {
                        String::from("Unknown")
                    }
                },
                None => {
                    tracing::trace!("Failed to decode transfer destination");
                    String::from("Unknown")
                }
            };
            let token_value = u128::from_str_radix(&token_value, 16)? / DEFAULT_DENOMINATOR;

            tracing::info!("Transfer destination: {destination_address}, amount: {token_value}");

            // Decode address fields
            let from_addr = tx.from.unwrap_or(H160::zero());
            let to_addr = tx.to.unwrap_or(H160::zero());

            // Decode eth value
            let eth_value = wei_to_eth(tx.value);

            tracing::info!(
                "[{}] ({} -> {}) from {}, to {}, value {}, gas {}, gas price {}",
                tx.transaction_index.unwrap_or(U64::from(0)),
                &token_name,
                &func_signature,
                w3h::to_string(&from_addr),
                w3h::to_string(&to_addr),
                eth_value,
                tx.gas,
                tx.gas_price.unwrap(),
            );
        }
    }
    Ok(())
}

// Read Ethereum block with specified block id
async fn read_block(web3s: &Web3<WebSocket>, block_id: BlockId) -> anyhow::Result<Block<H256>> {
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
        block.base_fee_per_gas.unwrap(),
        block.difficulty,
        block.total_difficulty.unwrap()
    );
    Ok(block)
}