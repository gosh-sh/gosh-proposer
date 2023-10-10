use crate::elock::get_elock_address;
use crate::eth::create_web3_socket;
use crate::helper::abi::EVENTS_IDS;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cmp::min;
use std::collections::HashMap;
use std::io::BufReader;
use std::str::FromStr;
use web3::types::U256;
use web3::Transport;

#[derive(Deserialize, Debug)]
struct EventLog {
    #[serde(rename = "address")]
    _address: String,
    topics: Vec<String>,
    data: String,
    #[serde(rename = "blockNumber")]
    _block_number: Option<String>,
    #[serde(rename = "transactionHash")]
    _transaction_hash: Option<String>,
    #[serde(rename = "transactionIndex")]
    _transaction_index: Option<String>,
    #[serde(rename = "blockHash")]
    _block_hash: Option<String>,
    #[serde(rename = "logIndex")]
    _log_index: Option<String>,
    #[serde(rename = "removed")]
    _removed: bool,
}

#[derive(Deserialize, Debug)]
struct Input {
    name: String,
    #[serde(rename = "type")]
    param_type: String,
}

#[derive(Deserialize, Debug)]
struct Event {
    name: String,
    params: Vec<Input>,
}

#[derive(Serialize, Debug)]
struct DecodedEvent {
    name: String,
    params: HashMap<String, String>,
}

fn get_signatures_table() -> anyhow::Result<HashMap<String, Event>> {
    let reader = BufReader::new(EVENTS_IDS.as_bytes());
    serde_json::from_reader(reader)
        .map_err(|e| anyhow::format_err!("Failed to decode identifiers map {}", e))
}

pub async fn get_events() -> anyhow::Result<()> {
    let web3s = create_web3_socket().await?;
    let elock_address = get_elock_address()?;

    let params = web3::helpers::serialize(&json!({
        "address": elock_address,
        "fromBlock": "0x11646F7"
    }));
    let res = web3s
        .transport()
        .execute("eth_getLogs", vec![params])
        .await
        .map_err(|e| anyhow::format_err!("Failed to execute ETH getLogs request: {e}"))?;

    let events: Vec<EventLog> = serde_json::from_value(res)?;

    let event_ids = get_signatures_table()?;

    let mut decoded_events = vec![];

    for event in events {
        if event.topics.is_empty() {
            tracing::info!("Event does not contain topics: {event:?}");
            continue;
        }
        match event_ids.get(&event.topics[0]) {
            None => {
                tracing::info!("Event topic was not found in the events map: {event:?}");
                continue;
            }
            Some(event_scheme) => {
                tracing::info!("Found event: {}", event_scheme.name);
                // TODO: change to real decode
                let data_buf = event
                    .data
                    .replace('"', "")
                    .trim_start_matches("0x")
                    .to_string();
                let mut extra_topics = event.topics.clone();
                extra_topics.remove(0);
                let mut params = HashMap::new();
                if data_buf.is_empty() {
                    if !extra_topics.is_empty() {
                        tracing::info!("Params:");
                        for i in 0..min(extra_topics.len(), event_scheme.params.len()) {
                            params.insert(
                                event_scheme.params[i].name.clone(),
                                extra_topics[i].clone(),
                            );
                            tracing::info!("{}={}", event_scheme.params[i].name, extra_topics[i]);
                        }
                    }
                } else {
                    tracing::info!("Params:");
                    let mut data = data_buf;
                    for param in &event_scheme.params {
                        match param.param_type.as_str() {
                            "address" => {
                                let address = extra_topics[0].clone();
                                extra_topics.remove(0);
                                params.insert(
                                    param.name.clone(),
                                    format!("0x{}", address[24..64].to_string()),
                                );
                                tracing::info!(
                                    "\t{}: 0x{}",
                                    param.name,
                                    address[24..64].to_string()
                                );
                            }
                            "uint256" => {
                                let u_param = data[0..64].to_string();
                                data = data[64..].to_string();
                                let u_param = U256::from_str(&u_param)?;
                                params.insert(param.name.clone(), format!("{}", u_param));
                                tracing::info!("\t{}: {}", param.name, u_param);
                            }
                            "string" => {
                                // skip first word
                                let _prefix = data[0..64].to_string();
                                data = data[64..].to_string();
                                let len = data[0..64].to_string();
                                data = data[64..].to_string();
                                let len = U256::from_str(&len)?.as_usize() * 2;
                                let string_data = data[0..len].to_string();
                                let string_decoded = hex::decode(string_data).map_err(|e| {
                                    anyhow::format_err!("Failed to decode string as hex: {e}")
                                })?;
                                let string = String::from_utf8(string_decoded).map_err(|e| {
                                    anyhow::format_err!("Failed to decode string as hex: {e}")
                                })?;
                                tracing::info!("\t{}: {}", param.name, string);
                                params.insert(param.name.clone(), string);
                            }
                            t => {
                                tracing::info!("Unsupported event param type: {t}");
                            }
                        }
                    }
                }
                decoded_events.push(DecodedEvent {
                    name: event_scheme.name.clone(),
                    params,
                })
            }
        }
    }

    tracing::info!("Decoded events:");
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::to_value(decoded_events)?)?
    );

    Ok(())
}
