use crate::elock::transfer::{Transfer, TransferPatch};
use crate::eth::events::get_events;
use crate::token_root::eth::get_root_data;
use crate::token_root::RootData;
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;
use web3::transports::WebSocket;
use web3::types::{Address, U64};
use web3::Web3;

const DEPOSIT_EVENT_SIGNATURE: &str =
    "0xf5681f9d0db1b911ac18ee83d515a1cf1051853a9eae418316a2fdf7dea427c5";
const DEPOSIT_EVENT_NAME: &str = "Deposited";

pub async fn get_deposits(
    web3s: &Web3<WebSocket>,
    elock_address: Address,
    from: U64,
    to: U64,
) -> anyhow::Result<Vec<TransferPatch>> {
    let params = web3::helpers::serialize(&json!({
        "address": elock_address,
        "fromBlock": format!("0x{:0x}", from),
        "toBlock": format!("0x{:0x}", to),
        "topics": vec![format!("{DEPOSIT_EVENT_SIGNATURE}")],
    }));
    tracing::info!(
        "Query ELock events with params: {}",
        serde_json::to_string_pretty(&params)?
    );
    let events = get_events(web3s, params).await?;
    tracing::info!("Queried events: {:?}", events);
    let mut roots_map: HashMap<String, RootData> = HashMap::new();
    let mut transfers = vec![];
    for event in events {
        assert_eq!(
            &event.name, DEPOSIT_EVENT_NAME,
            "Decoded ELock event has wrong name"
        );
        let eth_root = event.params.get("token").ok_or(anyhow::format_err!(
            "Decoded event arguments do not contain 'token' field"
        ))?;
        let root_data = match roots_map.get(eth_root) {
            Some(data) => data.to_owned(),
            None => {
                let root_address = Address::from_str(eth_root)?;
                let root_data = get_root_data(web3s, root_address).await?;
                roots_map.insert(eth_root.to_string(), root_data.clone());
                root_data
            }
        };
        let value = u128::from_str_radix(
            event
                .params
                .get("value")
                .ok_or(anyhow::format_err!(
                    "Decoded event arguments do not contain 'value' field"
                ))?
                .trim_start_matches("0x"),
            16,
        )
        .map_err(|e| anyhow::format_err!("Failed to convert event value to integer: {e}"))?;
        let pubkey = event
            .params
            .get("pubkey")
            .ok_or(anyhow::format_err!(
                "Decoded event arguments do not contain 'pubkey' field"
            ))?
            .to_owned();
        transfers.push(TransferPatch {
            data: Transfer {
                value,
                pubkey,
                hash: event.hash,
            },
            root: root_data,
        });
    }

    Ok(transfers)
}
