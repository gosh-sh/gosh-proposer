use crate::eth::helper::wei_to_eth;
use std::collections::BTreeMap;
use std::env;
use web3::helpers as w3h;
use web3::types::{Transaction, H160, U64};

const INPUT_CHUNK_SIZE: usize = 64; // Number of bytes in one function argument
const ADDRESS_PREFIX_SIZE: usize = 24; // Number of leading zeros in address argument
const DEFAULT_DENOMINATOR: f64 = 100.0;

#[derive(Debug)]
pub struct Transfer {
    from: String,
    to: String,
    value: f64,
}

pub fn decode_transfer(
    tx: Transaction,
    code_sig_lookup: &BTreeMap<String, Vec<String>>,
) -> anyhow::Result<Transfer> {
    // Token name
    let eth_token_name = env::var("ETH_TOKEN_NAME")?;

    // Decode function call
    let input_str: String = w3h::to_string(&tx.input);
    if input_str.len() < 12 {
        anyhow::bail!("Transaction body is too short");
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
        }
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
        }
        None => {
            tracing::trace!("Failed to decode transfer destination");
            String::from("Unknown")
        }
    };
    let token_value = u64::from_str_radix(&token_value, 16)? as f64 / DEFAULT_DENOMINATOR;

    tracing::info!("Transfer destination: {destination_address}, amount: {token_value}");

    // Decode address fields
    let from_addr = tx.from.unwrap_or(H160::zero());
    let to_addr = tx.to.unwrap_or(H160::zero());

    // Decode eth value
    let eth_value = wei_to_eth(tx.value);

    tracing::info!(
        "[{}] ({} -> {}) from {}, to {}, value {}, gas {}, gas price {}",
        tx.transaction_index.unwrap_or(U64::from(0)),
        &eth_token_name,
        &func_signature,
        w3h::to_string(&from_addr),
        w3h::to_string(&to_addr),
        eth_value,
        tx.gas,
        tx.gas_price.unwrap(),
    );

    let source_address = w3h::to_string(&from_addr)
        .trim()
        .trim_end_matches('"')
        .trim_start_matches('"')
        .to_string();
    Ok(Transfer {
        from: source_address,
        to: destination_address,
        value: token_value,
    })
}
