use crate::eth::helper::wei_to_eth;
use std::collections::BTreeMap;
use std::env;

use crate::eth::block::FullBlock;
use serde::Serialize;
use web3::transports::WebSocket;
use web3::types::{Transaction, TransactionId, H256, U64};
use web3::{helpers as w3h, Web3};

const INPUT_CHUNK_SIZE: usize = 64; // Number of bytes in one function argument
const ADDRESS_PREFIX_SIZE: usize = 24; // Number of leading zeros in address argument
const DEFAULT_DENOMINATOR: f64 = 100.0;

#[derive(Debug, Serialize)]
pub struct Transfer {
    pubkey: String,
    value: u128,
    hash: String,
}

fn decode_transfer(
    tx: Transaction,
    code_sig_lookup: &BTreeMap<String, Vec<String>>,
) -> anyhow::Result<Transfer> {
    // Decode function call
    tracing::trace!("Decode transaction: {}", w3h::to_string(&tx.hash));
    let input_str: String = w3h::to_string(&tx.input);
    if input_str.len() < 75 {
        tracing::trace!("Transaction body is too short");
        anyhow::bail!("Transaction body is too short");
    }
    tracing::trace!("input_str: {input_str}");
    let func_code = input_str[3..11].to_string();
    let func_signature: String = match code_sig_lookup.get(&func_code) {
        Some(func_sig) => format!("{:?}", func_sig.get(0).ok_or(anyhow::format_err!("Failed to decode function signature"))?),
        _ => {
            tracing::trace!("Function not found.");
            anyhow::bail!("Function not found.");
        }
    };

    let function_name = env::var("ETH_FUNCTION_NAME")?;
    if function_name != func_signature.replace('"', "") {
        tracing::trace!("Wrong function name: {function_name} != {func_signature}");
        anyhow::bail!("Wrong function name: {function_name} != {func_signature}");
    }

    let owner_pubkey = input_str[11..75].to_string();
    let eth_value = wei_to_eth(tx.value);
    tracing::trace!("Transfer owner: {owner_pubkey}, amount: {eth_value}");

    tracing::info!(
        "[{}] ({} -> {}) value {}, gas {}, gas price {}",
        tx.transaction_index.unwrap_or(U64::from(0)),
        "ETH",
        &func_signature,
        eth_value,
        tx.gas,
        tx.gas_price.unwrap(),
    );

    let tx_value = w3h::to_string(&tx.value).replace('"', "").trim_start_matches("0x").to_string();

    let tx_hash = w3h::to_string(&tx.hash).replace('"', "");
    let value = u128::from_str_radix(&tx_value, 16);
    let value = value?;
    let res = Transfer {
        hash: tx_hash,
        pubkey: owner_pubkey,
        value,
    };

    tracing::info!("Valid transfer: {:?}", res);

    Ok(res)
}

pub async fn filter_and_decode_block_transactions(
    web3s: &Web3<WebSocket>,
    block: &FullBlock<H256>,
    eth_contract_address: &str,
    code_sig_lookup: &BTreeMap<String, Vec<String>>,
) -> anyhow::Result<Vec<Transfer>> {
    let mut res = vec![];
    // Parse block transactions
    for transaction_hash in &block.transactions {
        // Load transaction
        tracing::info!("tx: {}", w3h::to_string(transaction_hash));
        let tx = match web3s
            .eth()
            .transaction(TransactionId::Hash(transaction_hash.to_owned()))
            .await
        {
            Ok(Some(tx)) => tx,
            _ => {
                tracing::info!("Failed to fetch transaction: {transaction_hash}");
                continue;
            }
        };

        // Check that transaction destination is equal to the specified address
        if let Some(address) = tx.to {
            let dest = w3h::to_string(&address)
                .trim()
                .trim_end_matches('"')
                .trim_start_matches('"')
                .to_lowercase();
            tracing::info!("Txn destination address: {dest}");
            if dest != eth_contract_address {
                tracing::info!(
                    "Wrong destination address, skip it. `{}` != `{eth_contract_address}`",
                    dest
                );
                continue;
            }
        } else {
            tracing::info!("No destination address, skip it.");
            continue;
        }

        match decode_transfer(tx, code_sig_lookup) {
            Ok(transfer) => res.push(transfer),
            Err(_) => {}
        }
    }
    tracing::info!("block {} transfers: {:?}", w3h::to_string(&block.hash), res);
    Ok(res)
}
