use crate::eth::helper::{get_signatures_table, wei_to_eth};
use std::collections::BTreeMap;
use std::env;
use std::str::FromStr;
use std::sync::Arc;

use crate::eth::block::FullBlock;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;
use web3::ethabi::Address;
use web3::transports::WebSocket;
use web3::types::{Transaction, TransactionId, H256, U64};
use web3::{helpers as w3h, Web3};

use crate::helper::deserialize_u128;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Transfer {
    pub pubkey: String,
    #[serde(deserialize_with = "deserialize_u128")]
    pub value: u128,
    pub hash: String,
}

pub fn decode_transfer(
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
        Some(func_sig) => format!(
            "{:?}",
            func_sig
                .get(0)
                .ok_or(anyhow::format_err!("Failed to decode function signature"))?
        ),
        _ => {
            anyhow::bail!("Function not found.");
        }
    };

    let function_name = env::var("ETH_FUNCTION_NAME")?;
    if function_name != func_signature.replace('"', "") {
        anyhow::bail!("Wrong function name: {function_name} != {func_signature}");
    }

    let owner_pubkey = input_str[11..75].to_string();
    let eth_value = wei_to_eth(tx.value);
    tracing::trace!("Transfer owner: 0x{owner_pubkey}, amount: {eth_value}");

    tracing::info!(
        "[{}] ({} -> {}) value {}, gas {}, gas price {}",
        tx.transaction_index.unwrap_or(U64::from(0)),
        "ETH",
        &func_signature,
        eth_value,
        tx.gas,
        tx.gas_price.unwrap(),
    );

    let tx_value = w3h::to_string(&tx.value)
        .replace('"', "")
        .trim_start_matches("0x")
        .to_string();

    let tx_hash = w3h::to_string(&tx.hash).replace('"', "");
    let value = u128::from_str_radix(&tx_value, 16);
    let value = value?;
    let res = Transfer {
        hash: tx_hash,
        pubkey: format!("0x{owner_pubkey}"),
        value,
    };

    tracing::info!("Valid transfer: {:?}", res);

    Ok(res)
}

pub async fn filter_and_decode_block_transactions(
    web3s: Arc<Web3<WebSocket>>,
    block: &FullBlock<H256>,
    eth_contract_address: &str,
) -> anyhow::Result<Vec<Transfer>> {
    // Parse block transactions
    tracing::info!("start search of transfers");
    let mut parallel: JoinSet<anyhow::Result<Transfer>> = JoinSet::new();
    for transaction_hash in &block.transactions {
        // Load transaction
        // tracing::info!("tx: {}", w3h::to_string(transaction_hash));
        let transaction_hash = *transaction_hash;
        let web3s_clone = web3s.clone();
        let eth_contract_address_clone = Address::from_str(eth_contract_address)?;
        parallel.spawn(async move {
            let tx = match web3s_clone
                .eth()
                .transaction(TransactionId::Hash(transaction_hash.to_owned()))
                .await
            {
                Ok(Some(tx)) => tx,
                _ => {
                    anyhow::bail!("Failed to fetch transaction: {transaction_hash}");
                }
            };

            // Check that transaction destination is equal to the specified address
            if let Some(address) = tx.to {
                if address != eth_contract_address_clone {
                    anyhow::bail!(
                    "Wrong destination address, skip it. `{address}` != `{eth_contract_address_clone}`",
                );
                }
            } else {
                anyhow::bail!("No destination address, skip it.");
            }
            let code_sig_lookup = get_signatures_table()?;
            decode_transfer(tx, &code_sig_lookup)
        });
    }

    let mut transfers = vec![];
    while let Some(res) = parallel.join_next().await {
        let val = res?;
        if let Ok(trans) = val {
            transfers.push(trans);
        }
    }
    tracing::info!(
        "block {} transfers: {:?}",
        w3h::to_string(&block.hash),
        transfers
    );
    Ok(transfers)
}
