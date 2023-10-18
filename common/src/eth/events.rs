use crate::elock::get_elock_address;
use crate::eth::create_web3_socket;
use crate::helper::abi::EVENTS_IDS;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::io::BufReader;
use std::str::FromStr;
use web3::transports::WebSocket;
use web3::types::{H256, U256};
use web3::{Transport, Web3};

#[derive(Deserialize, Debug)]
struct EventLog {
    #[serde(rename = "address")]
    _address: String,
    topics: Vec<String>,
    data: String,
    #[serde(rename = "blockNumber")]
    _block_number: Option<String>,
    #[serde(rename = "transactionHash")]
    transaction_hash: Option<String>,
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
    indexed: bool,
}

#[derive(Deserialize, Debug)]
struct Event {
    name: String,
    params: Vec<Input>,
    #[serde(rename = "anonymous")]
    _anonymous: bool,
}

#[derive(Serialize, Debug)]
pub struct DecodedEvent {
    pub name: String,
    pub params: HashMap<String, String>,
    pub hash: String,
}

fn get_events_signatures_table() -> anyhow::Result<HashMap<String, Event>> {
    let reader = BufReader::new(EVENTS_IDS.as_bytes());
    serde_json::from_reader(reader)
        .map_err(|e| anyhow::format_err!("Failed to decode identifiers map {}", e))
}

pub async fn get_all_events() -> anyhow::Result<()> {
    // create ETH client
    let web3s = create_web3_socket().await?;

    // Load ELock address
    let elock_address = get_elock_address()?;

    // Setup event filter
    let params = web3::helpers::serialize(&json!({
        "address": elock_address,
        "fromBlock": "0x0"
    }));

    // Get events
    let events = get_events(&web3s, params).await?;

    tracing::info!("Decoded events:");
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::to_value(events)?)?
    );

    Ok(())
}

// possible params for query:
// fromBlock - string
// The block number as a string in hexadecimal format or tags. The supported tag values include
// earliest for the earliest/genesis block, latest for the latest mined block, pending for the
// pending state/transactions, safe for the most recent secure block, and finalized for the most
// recent secure block accepted by more than 2/3 of validators. safe and finalized are only
// supported on Ethereum, Gnosis, Arbitrum, Arbitrum Nova, and Avalanche C-chain
//
// toBlock - string
// The block number as a string in hexadecimal format or tags. The supported tag values include
// earliest for the earliest/genesis block, latest for the latest mined block, pending for the
// pending state/transactions, safe for the most recent secure block, and finalized for the most
// recent secure block accepted by more than 2/3 of validators. safe and finalized are only
// supported on Ethereum, Gnosis, Arbitrum, Arbitrum Nova, and Avalanche C-chain
//
// address - string
// The contract address or a list of addresses from which logs should originate
//
// topics - string
// An array of DATA topics and also, the topics are order-dependent. Visit this official page to
// learn more about topics
//
// blockHash - string
// With the addition of EIP-234, blockHash is a new filter option that restricts the logs returned
// to the block number referenced in the blockHash. Using the blockHash field is equivalent to
// setting the fromBlock and toBlock to the block number the blockHash references. If blockHash
// is present in the filter criteria, neither fromBlock nor toBlock is allowed
pub async fn get_events(
    web3s: &Web3<WebSocket>,
    params: serde_json::Value,
) -> anyhow::Result<Vec<DecodedEvent>> {
    // Execute query
    let res = web3s
        .transport()
        .execute("eth_getLogs", vec![params])
        .await
        .map_err(|e| anyhow::format_err!("Failed to execute ETH getLogs request: {e}"))?;

    // Deserialize result
    let events: Vec<EventLog> = serde_json::from_value(res)?;

    // Load ELock events mapping
    let event_ids = get_events_signatures_table()?;

    let mut decoded_events = vec![];
    for event in events {
        if event.topics.is_empty() {
            // If event is anonymous, skip it
            tracing::info!("Event does not contain topics: {event:?}");
            continue;
        }
        match event_ids.get(&event.topics[0]) {
            None => {
                // If event is anonymous or failed to decode it, skip it
                tracing::info!("Event topic was not found in the events map: {event:?}");
                continue;
            }
            Some(event_scheme) => {
                tracing::info!("Found event: {}", event_scheme.name);
                // Decode event data to bytes
                let data_buf = event
                    .data
                    .replace('"', "")
                    .trim_start_matches("0x")
                    .to_string();

                let mut extra_topics = event.topics.clone();
                // Remove function signature from topics
                extra_topics.remove(0);

                // Decode arguments
                let mut params = HashMap::new();
                let mut indexed_cnt = 0;
                for (index, input) in event_scheme.params.iter().enumerate() {
                    if input.indexed {
                        indexed_cnt += 1;
                    }
                    let value =
                        decode_argument(input, index - indexed_cnt, &mut extra_topics, &data_buf)?;
                    params.insert(input.name.clone(), value);
                }
                tracing::info!("Decoded event args: {params:?}");
                decoded_events.push(DecodedEvent {
                    name: event_scheme.name.clone(),
                    params,
                    hash: event.transaction_hash.unwrap(),
                })
            }
        }
    }

    Ok(decoded_events)
}

fn decode_argument(
    param: &Input,
    param_index: usize,
    topics: &mut Vec<String>,
    data: &str,
) -> anyhow::Result<String> {
    // If param is indexed it is taken from topics, otherwise from data
    let trimmed_data = match param.indexed {
        true => {
            let arg = topics.remove(0);
            arg.replace('"', "").trim_start_matches("0x").to_string()
        }
        false => data[64 * param_index..64 * (param_index + 1)].to_string(),
    };
    match param.param_type.as_str() {
        "address" => Ok(trimmed_data[24..64].to_string()),
        "uint256" => {
            let u_param = H256::from_str(&trimmed_data)?;
            let str_param = web3::helpers::to_string(&u_param).replace('"', "");

            Ok(format!("{str_param}"))
        }
        "string" => {
            // in case of string or other array type first word describes offset in data where the
            // arg is stored
            let mut offset = U256::from_str(&trimmed_data)?.as_usize() * 2;
            let len = data[offset..offset + 64].to_string();
            let len = U256::from_str(&len)?.as_usize() * 2;
            // increase offset to the length field
            offset += 64;
            let string_data = data[offset..offset + len].to_string();
            let string_decoded = hex::decode(string_data)
                .map_err(|e| anyhow::format_err!("Failed to decode string as hex: {e}"))?;
            let string = String::from_utf8(string_decoded)
                .map_err(|e| anyhow::format_err!("Failed to decode string as hex: {e}"))?;
            Ok(string)
        }
        type_name => {
            anyhow::bail!("Unsupported event argument type: {type_name}");
        }
    }
}
